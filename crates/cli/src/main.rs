use clap::{Parser, ValueEnum};
use core::{Minifier, MinifyError, MinifyLevel};
use css::CSSPlugin;
use html::HTMLPlugin;
use js::JavaScriptPlugin;
use log::{debug, error, info};
use std::fs;
use std::path::PathBuf;

/// omni-minify: A format-aware CLI tool for minifying various file types
#[derive(Parser)]
#[command(name = "omni-minify")]
#[command(about = "A format-aware CLI tool for minifying various file types")]
#[command(version = "0.1.0")]
pub struct Args {
	/// Input file to minify
	pub input: PathBuf,

	/// Use safe minification mode (default)
	#[arg(long, conflicts_with = "aggressive")]
	pub safe: bool,

	/// Use aggressive minification mode
	#[arg(long, conflicts_with = "safe")]
	pub aggressive: bool,

	/// Display detailed minification statistics
	#[arg(long)]
	pub stats: bool,

	/// Override format detection and use specified format
	#[arg(long, value_enum)]
	pub format: Option<FormatOverride>,

	/// Output file path (default: input.min.ext)
	#[arg(long, short)]
	pub output: Option<PathBuf>,
}

/// Supported format overrides for manual format selection
#[derive(Clone, ValueEnum, Debug)]
pub enum FormatOverride {
	/// JavaScript/TypeScript
	Js,
	/// CSS
	Css,
	/// HTML
	Html,
	/// GLB (3D models)
	Glb,
	/// Rust
	Rust,
}

impl Args {
	/// Get the minification level based on flags
	pub fn get_minify_level(&self) -> MinifyLevel {
		if self.aggressive {
			MinifyLevel::Aggressive
		} else {
			// Default to Safe mode (even when neither flag is specified)
			MinifyLevel::Safe
		}
	}

	/// Get the output file path based on input and --output flag
	pub fn get_output_path(&self) -> PathBuf {
		if let Some(ref output) = self.output {
			// Use explicit output path if provided
			output.clone()
		} else {
			// Generate default output path: input.min.ext
			self.generate_default_output_path()
		}
	}

	/// Generate default output path by inserting .min before the extension
	fn generate_default_output_path(&self) -> PathBuf {
		let input_path = &self.input;

		if let Some(extension) = input_path.extension() {
			// Has extension: file.ext -> file.min.ext
			let stem = input_path.file_stem().unwrap_or_default();
			let parent = input_path
				.parent()
				.unwrap_or_else(|| std::path::Path::new("."));

			let new_filename = format!(
				"{}.min.{}",
				stem.to_string_lossy(),
				extension.to_string_lossy()
			);

			parent.join(new_filename)
		} else {
			// No extension: file -> file.min
			let filename = input_path.file_name().unwrap_or_default();
			let parent = input_path
				.parent()
				.unwrap_or_else(|| std::path::Path::new("."));

			let new_filename = format!("{}.min", filename.to_string_lossy());
			parent.join(new_filename)
		}
	}
}

/// Plugin registry that manages all available minification plugins
pub struct PluginRegistry {
	plugins: Vec<Box<dyn Minifier>>,
}

impl Default for PluginRegistry {
	fn default() -> Self {
		Self::new()
	}
}

impl PluginRegistry {
	/// Create a new plugin registry
	pub fn new() -> Self {
		debug!("Initializing plugin registry");

		Self {
			plugins: Vec::new(),
		}
	}

	/// Register a new plugin with the registry
	pub fn register(&mut self, plugin: Box<dyn Minifier>) {
		debug!("Registering plugin: {}", plugin.name());
		self.plugins.push(plugin);
	}

	/// Find the first plugin that can handle the given input
	/// Implements deterministic plugin selection (first match wins)
	pub fn find_plugin(&self, input_bytes: &[u8]) -> Option<&dyn Minifier> {
		debug!("Searching for plugin to handle {} bytes", input_bytes.len());

		// Handle empty input gracefully
		if input_bytes.is_empty() {
			debug!("Empty input provided, no plugin can handle this");
			return None;
		}

		for (index, plugin) in self.plugins.iter().enumerate() {
			debug!("Checking plugin {}: {}", index, plugin.name());
			if plugin.detect(input_bytes) {
				debug!(
					"Plugin '{}' can handle this input (first match wins)",
					plugin.name()
				);
				return Some(plugin.as_ref());
			}
		}

		debug!("No plugin found for input");
		None
	}

	/// Find plugin by format override
	/// Provides explicit format selection bypassing auto-detection
	pub fn find_plugin_by_format(&self, format: FormatOverride) -> Option<&dyn Minifier> {
		let format_name = match format {
			FormatOverride::Js => "javascript",
			FormatOverride::Css => "css",
			FormatOverride::Html => "html",
			FormatOverride::Glb => "glb",
			FormatOverride::Rust => "rust",
		};

		debug!("Looking for plugin with name: {}", format_name);

		let plugin = self
			.plugins
			.iter()
			.find(|plugin| plugin.name() == format_name)
			.map(|plugin| plugin.as_ref());

		if let Some(p) = plugin {
			debug!("Found plugin '{}' for format override", p.name());
		} else {
			debug!("No plugin found for format '{}'", format_name);
		}

		plugin
	}

	/// Get all registered plugin names
	pub fn get_plugin_names(&self) -> Vec<&str> {
		self.plugins.iter().map(|plugin| plugin.name()).collect()
	}

	/// Get the number of registered plugins
	pub fn len(&self) -> usize {
		self.plugins.len()
	}

	/// Check if the registry is empty
	pub fn is_empty(&self) -> bool {
		self.plugins.is_empty()
	}
}

/// Main application logic
pub struct App {
	registry: PluginRegistry,
}

impl Default for App {
	fn default() -> Self {
		Self::new()
	}
}

impl App {
	/// Create a new application instance
	pub fn new() -> Self {
		debug!("Creating new App instance");

		let mut app = Self {
			registry: PluginRegistry::new(),
		};

		// Register JavaScript plugin
		app.registry.register(Box::new(JavaScriptPlugin::new()));

		// Register CSS plugin
		app.registry.register(Box::new(CSSPlugin::new()));

		// Register HTML plugin
		app.registry.register(Box::new(HTMLPlugin::new()));

		debug!("App initialized with {} plugins", app.registry.len());
		app
	}

	/// Process a file according to the provided arguments
	pub fn process_file(&mut self, args: &Args) -> Result<(), MinifyError> {
		debug!("Processing file: {:?}", args.input);

		// Read input file
		let input_bytes = self.read_file(&args.input)?;
		debug!("Read {} bytes from input file", input_bytes.len());

		// Find appropriate plugin
		let plugin = if let Some(format_override) = &args.format {
			debug!("Using format override: {:?}", format_override);
			self.registry
				.find_plugin_by_format(format_override.clone())
				.ok_or_else(|| {
					debug!(
						"Plugin not found for format override: {:?}",
						format_override
					);
					MinifyError::UnsupportedFormat
				})?
		} else {
			debug!("Auto-detecting format");
			self.registry.find_plugin(&input_bytes).ok_or_else(|| {
				debug!("No plugin could detect the file format");
				MinifyError::UnsupportedFormat
			})?
		};

		info!("Selected plugin: {}", plugin.name());

		// Get minification level
		let level = args.get_minify_level();
		debug!("Using minification level: {:?}", level);

		// Perform minification
		let output_bytes = plugin.minify(&input_bytes, level)?;
		debug!(
			"Minification complete: {} -> {} bytes",
			input_bytes.len(),
			output_bytes.len()
		);

		// Get output path
		let output_path = args.get_output_path();
		debug!("Output path: {:?}", output_path);

		// Write output to new file (preserves original)
		self.write_file(&output_path, &output_bytes)?;
		info!(
			"File minified successfully: {} -> {}",
			args.input.display(),
			output_path.display()
		);

		// Display statistics if requested
		if args.stats {
			self.display_stats(
				plugin,
				input_bytes.len(),
				output_bytes.len(),
				&args.input,
				&output_path,
			);
		}

		Ok(())
	}

	/// Read file contents as bytes
	fn read_file(&self, path: &PathBuf) -> Result<Vec<u8>, MinifyError> {
		debug!("Reading file: {:?}", path);

		if !path.exists() {
			return Err(MinifyError::IoError(std::io::Error::new(
				std::io::ErrorKind::NotFound,
				format!("File not found: {}", path.display()),
			)));
		}

		let bytes = fs::read(path)?;
		debug!("Successfully read {} bytes", bytes.len());
		Ok(bytes)
	}

	/// Write bytes to file
	fn write_file(&self, path: &PathBuf, bytes: &[u8]) -> Result<(), MinifyError> {
		debug!("Writing {} bytes to file: {:?}", bytes.len(), path);
		fs::write(path, bytes)?;
		debug!("File written successfully");
		Ok(())
	}

	/// Display minification statistics
	fn display_stats(
		&self,
		plugin: &dyn Minifier,
		before_bytes: usize,
		after_bytes: usize,
		input_path: &PathBuf,
		output_path: &PathBuf,
	) {
		println!("\n=== Minification Statistics ===");
		println!("Plugin: {}", plugin.name());
		println!("Input: {}", input_path.display());
		println!("Output: {}", output_path.display());
		println!("Before: {} bytes", before_bytes);
		println!("After: {} bytes", after_bytes);

		let saved = before_bytes.saturating_sub(after_bytes);
		let percentage = if before_bytes > 0 {
			(saved as f64 / before_bytes as f64) * 100.0
		} else {
			0.0
		};

		println!("Saved: {} bytes ({:.1}%)", saved, percentage);

		// Display plugin-specific stats if available
		if let Some(stats) = plugin.stats()
			&& let Some(extra) = &stats.extra
		{
			println!("Details: {}", extra);
		}
		println!("===============================");
	}
}

fn main() {
	// Initialize logging
	env_logger::init();

	// Parse command line arguments
	let args = Args::parse();
	debug!(
		"Parsed arguments: input={:?}, level={:?}, stats={}, format={:?}, output={:?}",
		args.input,
		args.get_minify_level(),
		args.stats,
		args.format,
		args.output
	);

	// Create and run application
	let mut app = App::new();

	match app.process_file(&args) {
		Ok(()) => {
			debug!("Application completed successfully");
		}
		Err(e) => {
			error!("Application failed: {}", e);
			eprintln!("Error: {}", e);

			// Provide helpful error messages
			match e {
				MinifyError::UnsupportedFormat => {
					eprintln!("No plugin available to handle this file format.");
					if app.registry.is_empty() {
						eprintln!(
							"No plugins are currently registered. Plugins will be available once implemented."
						);
					} else {
						eprintln!(
							"Available plugins: {}",
							app.registry.get_plugin_names().join(", ")
						);
						eprintln!("Try using --format flag to override auto-detection.");
					}
				}
				MinifyError::IoError(_) => {
					eprintln!(
						"Check that the file exists and you have permission to read/write it."
					);
				}
				_ => {}
			}

			std::process::exit(1);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use quickcheck::{TestResult, quickcheck};
	use std::io::Write;
	use tempfile::NamedTempFile;

	/**
	 * Feature: omni-minify-cli, Property 1: CLI Flag Processing
	 * Validates: Requirements 1.2, 1.3, 1.4
	 */
	#[test]
	fn property_cli_flag_processing() {
		fn prop(safe_flag: bool, aggressive_flag: bool) -> TestResult {
			// Skip invalid combinations (both flags set)
			if safe_flag && aggressive_flag {
				return TestResult::discard();
			}

			debug!(
				"Testing CLI flag processing: safe={}, aggressive={}",
				safe_flag, aggressive_flag
			);

			let args = Args {
				input: PathBuf::from("test.js"),
				safe: safe_flag,
				aggressive: aggressive_flag,
				stats: false,
				format: None,
				output: None,
			};

			let level = args.get_minify_level();
			debug!("Resulting minify level: {:?}", level);

			// Property: Flag processing should follow the correct precedence rules
			let expected_level = if aggressive_flag {
				MinifyLevel::Aggressive
			} else {
				// Default to Safe mode when no flags or only safe flag is set
				MinifyLevel::Safe
			};

			debug!(
				"Expected level: {:?}, actual level: {:?}",
				expected_level, level
			);
			TestResult::from_bool(level == expected_level)
		}

		quickcheck(prop as fn(bool, bool) -> TestResult);
	}

	/**
	 * Feature: omni-minify-cli, Property 2: File Input Processing
	 * Validates: Requirements 1.1
	 */
	#[test]
	fn property_file_input_processing() {
		fn prop(file_content: Vec<u8>) -> TestResult {
			// Skip very large files to keep test reasonable
			if file_content.len() > 10000 {
				return TestResult::discard();
			}

			debug!(
				"Testing file input processing with {} bytes",
				file_content.len()
			);

			// Create a temporary file with the test content
			let mut temp_file = match NamedTempFile::new() {
				Ok(file) => file,
				Err(_) => return TestResult::discard(),
			};

			if temp_file.write_all(&file_content).is_err() {
				return TestResult::discard();
			}

			let app = App::new();
			let path = temp_file.path().to_path_buf();

			// Property: The CLI should successfully read any valid file path
			match app.read_file(&path) {
				Ok(read_content) => {
					debug!("File read successfully: {} bytes", read_content.len());
					// Property: Read content should match original content
					TestResult::from_bool(read_content == file_content)
				}
				Err(e) => {
					debug!("File read failed: {}", e);
					// For property testing, we expect file operations to succeed
					// with valid temporary files
					TestResult::failed()
				}
			}
		}

		quickcheck(prop as fn(Vec<u8>) -> TestResult);
	}

	#[test]
	fn test_args_minify_level() {
		// Test default (safe mode)
		let args = Args {
			input: PathBuf::from("test.js"),
			safe: false,
			aggressive: false,
			stats: false,
			format: None,
			output: None,
		};
		assert_eq!(args.get_minify_level(), MinifyLevel::Safe);

		// Test explicit safe mode
		let args = Args {
			input: PathBuf::from("test.js"),
			safe: true,
			aggressive: false,
			stats: false,
			format: None,
			output: None,
		};
		assert_eq!(args.get_minify_level(), MinifyLevel::Safe);

		// Test aggressive mode
		let args = Args {
			input: PathBuf::from("test.js"),
			safe: false,
			aggressive: true,
			stats: false,
			format: None,
			output: None,
		};
		assert_eq!(args.get_minify_level(), MinifyLevel::Aggressive);
	}

	#[test]
	fn test_output_path_generation() {
		// Test default output path generation
		let args = Args {
			input: PathBuf::from("script.js"),
			safe: false,
			aggressive: false,
			stats: false,
			format: None,
			output: None,
		};
		assert_eq!(args.get_output_path(), PathBuf::from("script.min.js"));

		// Test with custom output path
		let args = Args {
			input: PathBuf::from("script.js"),
			safe: false,
			aggressive: false,
			stats: false,
			format: None,
			output: Some(PathBuf::from("custom-output.js")),
		};
		assert_eq!(args.get_output_path(), PathBuf::from("custom-output.js"));

		// Test with no extension
		let args = Args {
			input: PathBuf::from("script"),
			safe: false,
			aggressive: false,
			stats: false,
			format: None,
			output: None,
		};
		assert_eq!(args.get_output_path(), PathBuf::from("script.min"));

		// Test with path and extension
		let args = Args {
			input: PathBuf::from("src/styles.css"),
			safe: false,
			aggressive: false,
			stats: false,
			format: None,
			output: None,
		};
		assert_eq!(args.get_output_path(), PathBuf::from("src/styles.min.css"));
	}

	#[test]
	fn test_plugin_registry_basic_operations() {
		let mut registry = PluginRegistry::new();
		assert!(registry.is_empty());
		assert_eq!(registry.len(), 0);

		// Mock plugin for testing
		struct TestPlugin;
		impl Minifier for TestPlugin {
			fn name(&self) -> &str {
				"test"
			}
			fn detect(&self, _: &[u8]) -> bool {
				true
			}
			fn minify(&self, input: &[u8], _: MinifyLevel) -> Result<Vec<u8>, MinifyError> {
				Ok(input.to_vec())
			}
			fn stats(&self) -> Option<core::MinifyStats> {
				None
			}
		}

		registry.register(Box::new(TestPlugin));
		assert!(!registry.is_empty());
		assert_eq!(registry.len(), 1);
		assert_eq!(registry.get_plugin_names(), vec!["test"]);
	}

	/**
	 * Feature: omni-minify-cli, Property 4: Plugin Detection Consistency
	 * Validates: Requirements 3.1, 3.2
	 */
	#[test]
	fn property_plugin_detection_consistency() {
		fn prop(input_data: Vec<u8>) -> TestResult {
			// Skip very large inputs to keep test reasonable
			if input_data.len() > 1000 {
				return TestResult::discard();
			}

			debug!(
				"Testing plugin detection consistency with {} bytes",
				input_data.len()
			);

			// Create registry with multiple test plugins
			let mut registry = PluginRegistry::new();

			// Mock plugins for testing detection consistency
			struct AlwaysDetectPlugin(String);
			impl Minifier for AlwaysDetectPlugin {
				fn name(&self) -> &str {
					&self.0
				}
				fn detect(&self, _: &[u8]) -> bool {
					true
				}
				fn minify(&self, input: &[u8], _: MinifyLevel) -> Result<Vec<u8>, MinifyError> {
					Ok(input.to_vec())
				}
				fn stats(&self) -> Option<core::MinifyStats> {
					None
				}
			}

			struct NeverDetectPlugin(String);
			impl Minifier for NeverDetectPlugin {
				fn name(&self) -> &str {
					&self.0
				}
				fn detect(&self, _: &[u8]) -> bool {
					false
				}
				fn minify(&self, input: &[u8], _: MinifyLevel) -> Result<Vec<u8>, MinifyError> {
					Ok(input.to_vec())
				}
				fn stats(&self) -> Option<core::MinifyStats> {
					None
				}
			}

			struct ConditionalDetectPlugin(String);
			impl Minifier for ConditionalDetectPlugin {
				fn name(&self) -> &str {
					&self.0
				}
				fn detect(&self, input_bytes: &[u8]) -> bool {
					// Detect based on content - look for specific patterns
					!input_bytes.is_empty() && input_bytes[0] == b'{'
				}
				fn minify(&self, input: &[u8], _: MinifyLevel) -> Result<Vec<u8>, MinifyError> {
					Ok(input.to_vec())
				}
				fn stats(&self) -> Option<core::MinifyStats> {
					None
				}
			}

			// Register plugins in specific order (first match wins)
			registry.register(Box::new(ConditionalDetectPlugin("conditional".to_string())));
			registry.register(Box::new(AlwaysDetectPlugin("always".to_string())));
			registry.register(Box::new(NeverDetectPlugin("never".to_string())));

			debug!("Registry has {} plugins", registry.len());

			// Property 1: Detection should be consistent across multiple calls
			let first_result = registry.find_plugin(&input_data);
			let second_result = registry.find_plugin(&input_data);
			let third_result = registry.find_plugin(&input_data);

			let consistency_check = match (first_result, second_result, third_result) {
				(Some(first), Some(second), Some(third)) => {
					let names_match =
						first.name() == second.name() && second.name() == third.name();
					debug!(
						"All calls found plugin '{}', consistent: {}",
						first.name(),
						names_match
					);
					names_match
				}
				(None, None, None) => {
					debug!("All calls found no plugin - consistent");
					true
				}
				_ => {
					debug!("Inconsistent detection results");
					false
				}
			};

			if !consistency_check {
				return TestResult::failed();
			}

			// Property 2: First matching plugin should always be selected (deterministic)
			if let Some(plugin) = first_result {
				let expected_plugin = if !input_data.is_empty() && input_data[0] == b'{' {
					"conditional" // Should match first
				} else {
					"always" // Should match second (always detects)
				};

				debug!(
					"Expected plugin: {}, actual plugin: {}",
					expected_plugin,
					plugin.name()
				);
				TestResult::from_bool(plugin.name() == expected_plugin)
			} else {
				// No plugin detected - this should only happen if input is empty or doesn't match any
				let should_detect = !input_data.is_empty();
				debug!("No plugin detected, should_detect: {}", should_detect);
				TestResult::from_bool(!should_detect)
			}
		}

		quickcheck(prop as fn(Vec<u8>) -> TestResult);
	}

	#[test]
	fn test_plugin_detection_edge_cases() {
		let mut registry = PluginRegistry::new();

		// Test empty input
		assert!(registry.find_plugin(&[]).is_none());

		// Test with no plugins registered
		assert!(registry.find_plugin(b"some content").is_none());

		// Test format override with no matching plugin
		assert!(registry.find_plugin_by_format(FormatOverride::Js).is_none());

		// Add a plugin and test
		struct TestPlugin;
		impl Minifier for TestPlugin {
			fn name(&self) -> &str {
				"test"
			}
			fn detect(&self, input: &[u8]) -> bool {
				!input.is_empty() && input[0] == b't'
			}
			fn minify(&self, input: &[u8], _: MinifyLevel) -> Result<Vec<u8>, MinifyError> {
				Ok(input.to_vec())
			}
			fn stats(&self) -> Option<core::MinifyStats> {
				None
			}
		}

		registry.register(Box::new(TestPlugin));

		// Test detection with matching content
		assert!(registry.find_plugin(b"test content").is_some());

		// Test detection with non-matching content
		assert!(registry.find_plugin(b"other content").is_none());
	}

	#[test]
	fn test_format_override_functionality() {
		let mut registry = PluginRegistry::new();

		// Mock plugins with specific names
		struct JsPlugin;
		impl Minifier for JsPlugin {
			fn name(&self) -> &str {
				"javascript"
			}
			fn detect(&self, _: &[u8]) -> bool {
				false
			} // Doesn't matter for override
			fn minify(&self, input: &[u8], _: MinifyLevel) -> Result<Vec<u8>, MinifyError> {
				Ok(input.to_vec())
			}
			fn stats(&self) -> Option<core::MinifyStats> {
				None
			}
		}

		struct CssPlugin;
		impl Minifier for CssPlugin {
			fn name(&self) -> &str {
				"css"
			}
			fn detect(&self, _: &[u8]) -> bool {
				false
			}
			fn minify(&self, input: &[u8], _: MinifyLevel) -> Result<Vec<u8>, MinifyError> {
				Ok(input.to_vec())
			}
			fn stats(&self) -> Option<core::MinifyStats> {
				None
			}
		}

		registry.register(Box::new(JsPlugin));
		registry.register(Box::new(CssPlugin));

		// Test format overrides
		let js_plugin = registry.find_plugin_by_format(FormatOverride::Js);
		assert!(js_plugin.is_some());
		assert_eq!(js_plugin.unwrap().name(), "javascript");

		let css_plugin = registry.find_plugin_by_format(FormatOverride::Css);
		assert!(css_plugin.is_some());
		assert_eq!(css_plugin.unwrap().name(), "css");

		// Test non-existent format
		let html_plugin = registry.find_plugin_by_format(FormatOverride::Html);
		assert!(html_plugin.is_none());
	}

	#[test]
	fn test_app_file_operations() {
		let app = App::new();

		// Test reading non-existent file
		let result = app.read_file(&PathBuf::from("non_existent_file.txt"));
		assert!(result.is_err());

		// Test reading and writing with temporary file
		let mut temp_file = NamedTempFile::new().unwrap();
		let test_content = b"test content";
		temp_file.write_all(test_content).unwrap();

		let path = temp_file.path().to_path_buf();
		let read_result = app.read_file(&path).unwrap();
		assert_eq!(read_result, test_content);

		// Test writing to different path (new behavior)
		let output_path = path.with_extension("min.txt");
		let new_content = b"new content";
		app.write_file(&output_path, new_content).unwrap();
		let read_again = app.read_file(&output_path).unwrap();
		assert_eq!(read_again, new_content);

		// Verify original file is unchanged
		let original_content = app.read_file(&path).unwrap();
		assert_eq!(original_content, test_content);
	}
}
