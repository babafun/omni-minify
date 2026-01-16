use clap::{Parser, ValueEnum};
use core::{file_safety, Minifier, MinifyError, MinifyLevel, ExitCode};
use css::CSSPlugin;
use glb::GlbPlugin;
use html::HTMLPlugin;
use js::JavaScriptPlugin;
use log::{debug, error, info, warn};
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
	/// C
	C,
	/// C++
	Cpp,
	/// Java
	Java,
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
			FormatOverride::C => "c",
			FormatOverride::Cpp => "cpp",
			FormatOverride::Java => "java",
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

		// Register binary format plugins first (more specific detection)
		// Register GLB plugin
		app.registry.register(Box::new(GlbPlugin::new()));

		// Register text-based plugins (less specific detection)
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

		// Get output path early for validation
		let output_path = args.get_output_path();
		debug!("Output path: {:?}", output_path);

		// Validate file permissions before processing
		file_safety::validate_file_permissions(&args.input, &output_path)?;

		// Read input file safely
		let input_bytes = file_safety::safe_read_file(&args.input)?;
		debug!("Read {} bytes from input file", input_bytes.len());

		// Create backup manager for safety
		let mut backup = file_safety::FileBackup::new(&args.input);

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

		// Perform minification with error handling
		let output_bytes = match plugin.minify(&input_bytes, level) {
			Ok(bytes) => {
				debug!(
					"Minification complete: {} -> {} bytes",
					input_bytes.len(),
					bytes.len()
				);
				bytes
			}
			Err(e) => {
				error!("Minification failed with plugin '{}': {}", plugin.name(), e);
				
				// Restore from backup if needed (though input file shouldn't be modified)
				if let Err(restore_err) = backup.restore_from_backup() {
					warn!("Failed to restore from backup: {}", restore_err);
				}
				
				return Err(MinifyError::PluginError(format!(
					"Plugin '{}' failed: {}", plugin.name(), e
				)));
			}
		};

		// Write output to new file safely (preserves original)
		match file_safety::safe_write_file(&output_path, &output_bytes) {
			Ok(()) => {
				info!(
					"File minified successfully: {} -> {}",
					args.input.display(),
					output_path.display()
				);
			}
			Err(e) => {
				error!("Failed to write output file: {}", e);
				
				// Clean up any partial output file
				if output_path.exists() {
					if let Err(cleanup_err) = fs::remove_file(&output_path) {
						warn!("Failed to cleanup partial output file: {}", cleanup_err);
					}
				}
				
				return Err(e);
			}
		}

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
		
		// Get plugin-specific stats if available
		if let Some(stats) = plugin.stats() {
			// Use the comprehensive report from MinifyStats
			println!("{}", stats.generate_report());
		} else {
			// Fallback to basic statistics if plugin doesn't provide detailed stats
			println!("Before: {} bytes", before_bytes);
			println!("After: {} bytes", after_bytes);
			
			let saved = before_bytes.saturating_sub(after_bytes);
			let percentage = if before_bytes > 0 {
				(saved as f64 / before_bytes as f64) * 100.0
			} else {
				0.0
			};
			
			println!("Saved: {} bytes ({:.1}%)", saved, percentage);
		}
		
		println!("===============================");
	}

	/// Display detailed error information with helpful suggestions
	fn display_error_help(&self, error: &MinifyError) {
		match error {
			MinifyError::UnsupportedFormat => {
				eprintln!("No plugin available to handle this file format.");
				if self.registry.is_empty() {
					eprintln!("No plugins are currently registered. Plugins will be available once implemented.");
				} else {
					eprintln!("Available plugins: {}", self.registry.get_plugin_names().join(", "));
					eprintln!("Try using --format flag to override auto-detection.");
				}
			}
			MinifyError::FileNotFound(_) => {
				eprintln!("Check that the input file exists and the path is correct.");
			}
			MinifyError::PermissionError(_) => {
				eprintln!("Check file and directory permissions.");
				eprintln!("Ensure you have read access to the input file and write access to the output directory.");
			}
			MinifyError::MemoryLimitExceeded(_) => {
				eprintln!("The file is too large to process safely.");
				eprintln!("Maximum supported file size is {} MB.", file_safety::MAX_FILE_SIZE / (1024 * 1024));
				eprintln!("Consider splitting the file or processing it in smaller chunks.");
			}
			MinifyError::ParseError(_) => {
				eprintln!("The file contains syntax errors or is corrupted.");
				eprintln!("Verify the file is valid and try again.");
			}
			MinifyError::PluginError(_) => {
				eprintln!("The minification plugin encountered an error.");
				eprintln!("This may indicate a bug in the plugin or unsupported syntax.");
			}
			MinifyError::BackupError(_) => {
				eprintln!("Failed to create or manage file backup.");
				eprintln!("Check available disk space and file permissions.");
			}
			MinifyError::OutputExists(_) => {
				eprintln!("Output file already exists.");
				eprintln!("Use --output flag to specify a different output path.");
			}
			MinifyError::IoError(_) => {
				eprintln!("A file system error occurred.");
				eprintln!("Check file permissions, disk space, and file system integrity.");
			}
		}
	}
}

fn main() {
	// Initialize logging
	env_logger::init();

	// Parse command line arguments
	let args = match Args::try_parse() {
		Ok(args) => args,
		Err(e) => {
			// Handle clap errors (invalid arguments)
			eprintln!("Error: {}", e);
			std::process::exit(ExitCode::InvalidArguments as i32);
		}
	};

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
			std::process::exit(ExitCode::Success as i32);
		}
		Err(e) => {
			error!("Application failed: {}", e);
			eprintln!("Error: {}", e);

			// Display helpful error information
			app.display_error_help(&e);

			// Exit with appropriate error code
			let exit_code = ExitCode::from(e);
			debug!("Exiting with code: {:?}", exit_code);
			std::process::exit(exit_code as i32);
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

			let path = temp_file.path().to_path_buf();

			// Property: The CLI should successfully read any valid file path
			match file_safety::safe_read_file(&path) {
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
	fn test_file_safety_operations() {
		// Test reading non-existent file
		let result = file_safety::safe_read_file(&PathBuf::from("non_existent_file.txt"));
		assert!(result.is_err());
		match result.unwrap_err() {
			MinifyError::FileNotFound(_) => {}, // Expected
			_ => panic!("Expected FileNotFound error"),
		}

		// Test reading and writing with temporary file
		let mut temp_file = NamedTempFile::new().unwrap();
		let test_content = b"test content";
		temp_file.write_all(test_content).unwrap();

		let path = temp_file.path().to_path_buf();
		let read_result = file_safety::safe_read_file(&path).unwrap();
		assert_eq!(read_result, test_content);

		// Test writing to different path (new behavior)
		let output_path = path.with_extension("min.txt");
		let new_content = b"new content";
		file_safety::safe_write_file(&output_path, new_content).unwrap();
		let read_again = file_safety::safe_read_file(&output_path).unwrap();
		assert_eq!(read_again, new_content);

		// Verify original file is unchanged
		let original_content = file_safety::safe_read_file(&path).unwrap();
		assert_eq!(original_content, test_content);
	}

	#[test]
	fn test_file_backup_operations() {
		// Create a temporary file
		let mut temp_file = NamedTempFile::new().unwrap();
		let original_content = b"original content";
		temp_file.write_all(original_content).unwrap();
		let path = temp_file.path().to_path_buf();

		// Test backup creation and restoration
		let mut backup = file_safety::FileBackup::new(&path);
		
		// Create backup
		backup.create_backup().unwrap();
		
		// Modify original file
		let modified_content = b"modified content";
		file_safety::safe_write_file(&path, modified_content).unwrap();
		
		// Verify file was modified
		let current_content = file_safety::safe_read_file(&path).unwrap();
		assert_eq!(current_content, modified_content);
		
		// Restore from backup
		backup.restore_from_backup().unwrap();
		
		// Verify restoration
		let restored_content = file_safety::safe_read_file(&path).unwrap();
		assert_eq!(restored_content, original_content);
		
		// Cleanup
		backup.cleanup_backup().unwrap();
	}

	#[test]
	fn test_file_permission_validation() {
		// Test with temporary file (should succeed)
		let mut temp_file = NamedTempFile::new().unwrap();
		temp_file.write_all(b"test").unwrap();
		let input_path = temp_file.path().to_path_buf();
		let output_path = input_path.with_extension("min.txt");
		
		let result = file_safety::validate_file_permissions(&input_path, &output_path);
		assert!(result.is_ok());
		
		// Test with non-existent input file
		let non_existent = PathBuf::from("non_existent_file.txt");
		let result = file_safety::validate_file_permissions(&non_existent, &output_path);
		assert!(result.is_err());
		match result.unwrap_err() {
			MinifyError::FileNotFound(_) => {}, // Expected
			_ => panic!("Expected FileNotFound error"),
		}
	}

	/**
	 * Feature: omni-minify-cli, Property 9: File Safety on Failure
	 * Validates: Requirements 10.3
	 */
	#[test]
	fn property_file_safety_on_failure() {
		fn prop(file_content: Vec<u8>) -> TestResult {
			use log::debug;
			
			// Skip very large files to keep test reasonable
			if file_content.len() > 10000 {
				return TestResult::discard();
			}

			debug!("Testing file safety on failure with {} bytes", file_content.len());

			// Create a temporary input file with the test content
			let mut temp_input = match NamedTempFile::new() {
				Ok(file) => file,
				Err(_) => return TestResult::discard(),
			};

			if temp_input.write_all(&file_content).is_err() {
				return TestResult::discard();
			}

			let input_path = temp_input.path().to_path_buf();
			let output_path = input_path.with_extension("min.tmp");

			debug!("Input path: {:?}, Output path: {:?}", input_path, output_path);

			// Read original file content for comparison
			let original_content = match file_safety::safe_read_file(&input_path) {
				Ok(content) => content,
				Err(_) => return TestResult::discard(),
			};

			debug!("Original content: {} bytes", original_content.len());

			// Create a mock plugin that always fails minification
			struct FailingPlugin;
			impl Minifier for FailingPlugin {
				fn name(&self) -> &str {
					"failing_plugin"
				}
				fn detect(&self, _: &[u8]) -> bool {
					true // Always detect to ensure it gets used
				}
				fn minify(&self, _: &[u8], _: MinifyLevel) -> Result<Vec<u8>, MinifyError> {
					// Always fail with a parse error
					Err(MinifyError::ParseError("Intentional failure for testing".to_string()))
				}
				fn stats(&self) -> Option<core::MinifyStats> {
					None
				}
			}

			// Create a registry with only the failing plugin
			let mut registry = PluginRegistry::new();
			registry.register(Box::new(FailingPlugin));

			// Create app with the failing plugin registry
			let mut app = App {
				registry,
			};

			// Create args that would trigger minification
			let args = Args {
				input: input_path.clone(),
				safe: true,
				aggressive: false,
				stats: false,
				format: None,
				output: Some(output_path.clone()),
			};

			debug!("Attempting minification that should fail");

			// Attempt to process the file (should fail)
			let result = app.process_file(&args);

			debug!("Minification result: {:?}", result.is_err());

			// Property 1: Minification should fail (we expect this)
			if result.is_ok() {
				debug!("Expected minification to fail, but it succeeded");
				return TestResult::failed();
			}

			// Property 2: Original file should remain unchanged
			match file_safety::safe_read_file(&input_path) {
				Ok(current_content) => {
					if current_content != original_content {
						debug!("Original file was modified! Original: {} bytes, Current: {} bytes", 
							   original_content.len(), current_content.len());
						return TestResult::failed();
					}
					debug!("Original file preserved correctly");
				}
				Err(e) => {
					debug!("Failed to read original file after failure: {}", e);
					return TestResult::failed();
				}
			}

			// Property 3: No output file should be created
			if output_path.exists() {
				debug!("Output file was created despite failure: {:?}", output_path);
				return TestResult::failed();
			}

			debug!("File safety on failure validated successfully");
			TestResult::passed()
		}

		quickcheck(prop as fn(Vec<u8>) -> TestResult);
	}

	#[test]
	fn test_exit_code_mapping() {
		// Test error to exit code conversion
		assert_eq!(ExitCode::from(MinifyError::FileNotFound("test".to_string())), ExitCode::FileNotFound);
		assert_eq!(ExitCode::from(MinifyError::PermissionError("test".to_string())), ExitCode::PermissionDenied);
		assert_eq!(ExitCode::from(MinifyError::UnsupportedFormat), ExitCode::UnsupportedFormat);
		assert_eq!(ExitCode::from(MinifyError::ParseError("test".to_string())), ExitCode::ParseError);
		assert_eq!(ExitCode::from(MinifyError::PluginError("test".to_string())), ExitCode::PluginError);
		assert_eq!(ExitCode::from(MinifyError::MemoryLimitExceeded("test".to_string())), ExitCode::MemoryLimitExceeded);
	}
}
