use std::fmt;
use std::fs;

/// Minification optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinifyLevel {
	/// Guaranteed semantic preservation - only removes whitespace and comments
	Safe,
	/// Best-effort optimization with potential renaming and aggressive optimizations
	Aggressive,
}

/// Statistics about minification results
#[derive(Debug, Clone, PartialEq)]
pub struct MinifyStats {
	/// Original file size in bytes
	pub before_bytes: usize,
	/// Minified file size in bytes
	pub after_bytes: usize,
	/// Format-specific metrics (e.g., "lines: 100 -> 50, identifiers renamed: 25")
	pub extra: Option<String>,
	/// Line count statistics (before, after)
	pub lines: Option<(usize, usize)>,
	/// Format-specific counts (e.g., triangles for 3D models, selectors for CSS)
	pub format_metrics: Option<FormatMetrics>,
}

/// Format-specific metrics for different file types
#[derive(Debug, Clone, PartialEq)]
pub enum FormatMetrics {
	/// JavaScript/TypeScript metrics
	JavaScript {
		identifiers_renamed: usize,
		functions_inlined: usize,
		dead_code_removed: usize,
	},
	/// CSS metrics
	Css {
		selectors_merged: usize,
		rules_removed: usize,
		properties_collapsed: usize,
	},
	/// HTML metrics
	Html {
		tags_removed: usize,
		attributes_collapsed: usize,
		comments_removed: usize,
	},
	/// 3D model metrics
	Glb {
		triangles_before: usize,
		triangles_after: usize,
		vertices_before: usize,
		vertices_after: usize,
	},
	/// Rust code metrics
	Rust {
		imports_removed: usize,
		identifiers_renamed: usize,
		expressions_collapsed: usize,
	},
	/// C code metrics
	C {
		macros_inlined: usize,
		debug_statements_removed: usize,
		preprocessor_directives_collapsed: usize,
	},
	/// C++ code metrics
	Cpp {
		templates_inlined: usize,
		includes_optimized: usize,
		namespaces_collapsed: usize,
	},
	/// Java code metrics
	Java {
		imports_optimized: usize,
		debug_code_removed: usize,
		constants_inlined: usize,
	},
}

impl MinifyStats {
	/// Create new stats with before and after byte counts
	pub fn new(before_bytes: usize, after_bytes: usize) -> Self {
		Self {
			before_bytes,
			after_bytes,
			extra: None,
			lines: None,
			format_metrics: None,
		}
	}

	/// Add format-specific extra information
	pub fn with_extra(mut self, extra: String) -> Self {
		self.extra = Some(extra);
		self
	}

	/// Add line count statistics
	pub fn with_lines(mut self, before_lines: usize, after_lines: usize) -> Self {
		self.lines = Some((before_lines, after_lines));
		self
	}

	/// Add format-specific metrics
	pub fn with_format_metrics(mut self, metrics: FormatMetrics) -> Self {
		self.format_metrics = Some(metrics);
		self
	}

	/// Calculate percentage reduction
	pub fn reduction_percentage(&self) -> f64 {
		if self.before_bytes == 0 {
			0.0
		} else {
			((self.before_bytes - self.after_bytes) as f64 / self.before_bytes as f64) * 100.0
		}
	}

	/// Calculate bytes saved
	pub fn bytes_saved(&self) -> usize {
		self.before_bytes.saturating_sub(self.after_bytes)
	}

	/// Calculate line reduction percentage
	pub fn line_reduction_percentage(&self) -> Option<f64> {
		self.lines.map(|(before, after)| {
			if before == 0 {
				0.0
			} else {
				((before - after) as f64 / before as f64) * 100.0
			}
		})
	}

	/// Get lines saved count
	pub fn lines_saved(&self) -> Option<usize> {
		self.lines.map(|(before, after)| before.saturating_sub(after))
	}

	/// Generate a comprehensive statistics report
	pub fn generate_report(&self) -> String {
		let mut report = Vec::new();

		// Basic size statistics
		report.push(format!("Size: {} → {} bytes", self.before_bytes, self.after_bytes));
		report.push(format!("Saved: {} bytes ({:.1}%)", self.bytes_saved(), self.reduction_percentage()));

		// Line statistics if available
		if let Some((before_lines, after_lines)) = self.lines {
			let line_percentage = self.line_reduction_percentage().unwrap_or(0.0);
			report.push(format!("Lines: {} → {} ({:.1}% reduction)", before_lines, after_lines, line_percentage));
		}

		// Format-specific metrics
		if let Some(ref metrics) = self.format_metrics {
			match metrics {
				FormatMetrics::JavaScript { identifiers_renamed, functions_inlined, dead_code_removed } => {
					if *identifiers_renamed > 0 {
						report.push(format!("Identifiers renamed: {}", identifiers_renamed));
					}
					if *functions_inlined > 0 {
						report.push(format!("Functions inlined: {}", functions_inlined));
					}
					if *dead_code_removed > 0 {
						report.push(format!("Dead code blocks removed: {}", dead_code_removed));
					}
				}
				FormatMetrics::Css { selectors_merged, rules_removed, properties_collapsed } => {
					if *selectors_merged > 0 {
						report.push(format!("Selectors merged: {}", selectors_merged));
					}
					if *rules_removed > 0 {
						report.push(format!("Rules removed: {}", rules_removed));
					}
					if *properties_collapsed > 0 {
						report.push(format!("Properties collapsed: {}", properties_collapsed));
					}
				}
				FormatMetrics::Html { tags_removed, attributes_collapsed, comments_removed } => {
					if *tags_removed > 0 {
						report.push(format!("Tags removed: {}", tags_removed));
					}
					if *attributes_collapsed > 0 {
						report.push(format!("Attributes collapsed: {}", attributes_collapsed));
					}
					if *comments_removed > 0 {
						report.push(format!("Comments removed: {}", comments_removed));
					}
				}
				FormatMetrics::Glb { triangles_before, triangles_after, vertices_before, vertices_after } => {
					let triangle_reduction = ((triangles_before - triangles_after) as f64 / *triangles_before as f64) * 100.0;
					let vertex_reduction = ((vertices_before - vertices_after) as f64 / *vertices_before as f64) * 100.0;
					report.push(format!("Triangles: {} → {} ({:.1}% reduction)", triangles_before, triangles_after, triangle_reduction));
					report.push(format!("Vertices: {} → {} ({:.1}% reduction)", vertices_before, vertices_after, vertex_reduction));
				}
				FormatMetrics::Rust { imports_removed, identifiers_renamed, expressions_collapsed } => {
					if *imports_removed > 0 {
						report.push(format!("Imports removed: {}", imports_removed));
					}
					if *identifiers_renamed > 0 {
						report.push(format!("Identifiers renamed: {}", identifiers_renamed));
					}
					if *expressions_collapsed > 0 {
						report.push(format!("Expressions collapsed: {}", expressions_collapsed));
					}
				}
				FormatMetrics::C { macros_inlined, debug_statements_removed, preprocessor_directives_collapsed } => {
					if *macros_inlined > 0 {
						report.push(format!("Macros inlined: {}", macros_inlined));
					}
					if *debug_statements_removed > 0 {
						report.push(format!("Debug statements removed: {}", debug_statements_removed));
					}
					if *preprocessor_directives_collapsed > 0 {
						report.push(format!("Preprocessor directives collapsed: {}", preprocessor_directives_collapsed));
					}
				}
				FormatMetrics::Cpp { templates_inlined, includes_optimized, namespaces_collapsed } => {
					if *templates_inlined > 0 {
						report.push(format!("Templates inlined: {}", templates_inlined));
					}
					if *includes_optimized > 0 {
						report.push(format!("Includes optimized: {}", includes_optimized));
					}
					if *namespaces_collapsed > 0 {
						report.push(format!("Namespaces collapsed: {}", namespaces_collapsed));
					}
				}
				FormatMetrics::Java { imports_optimized, debug_code_removed, constants_inlined } => {
					if *imports_optimized > 0 {
						report.push(format!("Imports optimized: {}", imports_optimized));
					}
					if *debug_code_removed > 0 {
						report.push(format!("Debug code removed: {}", debug_code_removed));
					}
					if *constants_inlined > 0 {
						report.push(format!("Constants inlined: {}", constants_inlined));
					}
				}
			}
		}

		// Include extra information if available
		if let Some(ref extra) = self.extra {
			report.push(extra.clone());
		}

		report.join(", ")
	}
}

/// Errors that can occur during minification
///
#[derive(Debug)]
pub enum MinifyError {
	/// Error parsing the input format
	ParseError(String),
	/// I/O error during file operations
	IoError(std::io::Error),
	/// Format is not supported by any plugin
	UnsupportedFormat,
	/// Plugin-specific error
	PluginError(String),
	/// File permission error
	PermissionError(String),
	/// File not found error
	FileNotFound(String),
	/// Output file already exists and would be overwritten
	OutputExists(String),
	/// Memory limit exceeded for large files
	MemoryLimitExceeded(String),
	/// File backup/restore operation failed
	BackupError(String),
}

/// Exit codes for different error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
	/// Success
	Success = 0,
	/// General error
	GeneralError = 1,
	/// File not found
	FileNotFound = 2,
	/// Permission denied
	PermissionDenied = 3,
	/// Unsupported format
	UnsupportedFormat = 4,
	/// Parse error
	ParseError = 5,
	/// Plugin error
	PluginError = 6,
	/// Memory limit exceeded
	MemoryLimitExceeded = 7,
	/// Invalid arguments
	InvalidArguments = 8,
}

impl From<MinifyError> for ExitCode {
	fn from(error: MinifyError) -> Self {
		match error {
			MinifyError::FileNotFound(_) => ExitCode::FileNotFound,
			MinifyError::PermissionError(_) => ExitCode::PermissionDenied,
			MinifyError::UnsupportedFormat => ExitCode::UnsupportedFormat,
			MinifyError::ParseError(_) => ExitCode::ParseError,
			MinifyError::PluginError(_) => ExitCode::PluginError,
			MinifyError::MemoryLimitExceeded(_) => ExitCode::MemoryLimitExceeded,
			MinifyError::IoError(_) => ExitCode::GeneralError,
			MinifyError::OutputExists(_) => ExitCode::GeneralError,
			MinifyError::BackupError(_) => ExitCode::GeneralError,
		}
	}
}

impl fmt::Display for MinifyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			MinifyError::ParseError(msg) => write!(f, "Parse error: {}", msg),
			MinifyError::IoError(err) => write!(f, "I/O error: {}", err),
			MinifyError::UnsupportedFormat => write!(f, "Unsupported file format"),
			MinifyError::PluginError(msg) => write!(f, "Plugin error: {}", msg),
			MinifyError::PermissionError(msg) => write!(f, "Permission error: {}", msg),
			MinifyError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
			MinifyError::OutputExists(msg) => write!(f, "Output file exists: {}", msg),
			MinifyError::MemoryLimitExceeded(msg) => write!(f, "Memory limit exceeded: {}", msg),
			MinifyError::BackupError(msg) => write!(f, "Backup operation failed: {}", msg),
		}
	}
}

impl std::error::Error for MinifyError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			MinifyError::IoError(err) => Some(err),
			_ => None,
		}
	}
}

impl From<std::io::Error> for MinifyError {
	fn from(err: std::io::Error) -> Self {
		match err.kind() {
			std::io::ErrorKind::NotFound => MinifyError::FileNotFound(err.to_string()),
			std::io::ErrorKind::PermissionDenied => MinifyError::PermissionError(err.to_string()),
			_ => MinifyError::IoError(err),
		}
	}
}

/// Core trait that all minification plugins must implement
pub trait Minifier {
	/// Get the name of this minifier plugin
	fn name(&self) -> &str;

	/// Detect if this plugin can handle the given input bytes
	/// Returns true if the plugin can process this format
	fn detect(&self, input_bytes: &[u8]) -> bool;

	/// Minify the input bytes using the specified optimization level
	/// Returns the minified bytes or an error
	fn minify(&self, input_bytes: &[u8], level: MinifyLevel) -> Result<Vec<u8>, MinifyError>;

	/// Get statistics from the last minification operation
	/// Returns None if no minification has been performed yet
	fn stats(&self) -> Option<MinifyStats>;
}

/// File safety utilities for safe file operations
pub mod file_safety {
	use super::*;
	use std::fs::{File, OpenOptions};
	use std::io::{Read, Write};
	use std::path::{Path, PathBuf};
	use log::{debug, warn};

	/// Maximum file size to process (100MB)
	pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

	/// File backup manager for safe operations
	pub struct FileBackup {
		original_path: PathBuf,
		backup_path: Option<PathBuf>,
		backup_created: bool,
	}

	impl FileBackup {
		/// Create a new file backup manager
		pub fn new(original_path: &Path) -> Self {
			Self {
				original_path: original_path.to_path_buf(),
				backup_path: None,
				backup_created: false,
			}
		}

		/// Create a backup of the original file
		pub fn create_backup(&mut self) -> Result<(), MinifyError> {
			if !self.original_path.exists() {
				return Err(MinifyError::FileNotFound(format!(
					"Original file does not exist: {}",
					self.original_path.display()
				)));
			}

			let backup_path = self.generate_backup_path();
			debug!("Creating backup: {} -> {}", 
				   self.original_path.display(), backup_path.display());

			fs::copy(&self.original_path, &backup_path)
				.map_err(|e| MinifyError::BackupError(format!(
					"Failed to create backup: {}", e
				)))?;

			self.backup_path = Some(backup_path);
			self.backup_created = true;
			debug!("Backup created successfully");
			Ok(())
		}

		/// Restore from backup if it exists
		pub fn restore_from_backup(&mut self) -> Result<(), MinifyError> {
			if let Some(ref backup_path) = self.backup_path {
				if backup_path.exists() {
					debug!("Restoring from backup: {} -> {}", 
						   backup_path.display(), self.original_path.display());
					
					fs::copy(backup_path, &self.original_path)
						.map_err(|e| MinifyError::BackupError(format!(
							"Failed to restore from backup: {}", e
						)))?;
					
					debug!("Restored from backup successfully");
				}
			}
			Ok(())
		}

		/// Clean up backup file
		pub fn cleanup_backup(&mut self) -> Result<(), MinifyError> {
			if let Some(ref backup_path) = self.backup_path {
				if backup_path.exists() {
					debug!("Cleaning up backup: {}", backup_path.display());
					fs::remove_file(backup_path)
						.map_err(|e| MinifyError::BackupError(format!(
							"Failed to cleanup backup: {}", e
						)))?;
					debug!("Backup cleaned up successfully");
				}
			}
			self.backup_path = None;
			self.backup_created = false;
			Ok(())
		}

		/// Generate backup file path
		fn generate_backup_path(&self) -> PathBuf {
			let mut backup_path = self.original_path.clone();
			
			// Add .backup extension
			if let Some(extension) = backup_path.extension() {
				let new_extension = format!("{}.backup", extension.to_string_lossy());
				backup_path.set_extension(new_extension);
			} else {
				backup_path.set_extension("backup");
			}

			// If backup already exists, add timestamp
			if backup_path.exists() {
				let timestamp = std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)
					.unwrap_or_default()
					.as_secs();
				
				let stem = backup_path.file_stem().unwrap_or_default();
				let extension = backup_path.extension().unwrap_or_default();
				let new_name = format!("{}.{}.{}", 
					stem.to_string_lossy(), timestamp, extension.to_string_lossy());
				backup_path.set_file_name(new_name);
			}

			backup_path
		}
	}

	impl Drop for FileBackup {
		fn drop(&mut self) {
			if self.backup_created {
				if let Err(e) = self.cleanup_backup() {
					warn!("Failed to cleanup backup during drop: {}", e);
				}
			}
		}
	}

	/// Safely read a file with size and permission checks
	pub fn safe_read_file(path: &Path) -> Result<Vec<u8>, MinifyError> {
		debug!("Safe reading file: {}", path.display());

		// Check if file exists
		if !path.exists() {
			return Err(MinifyError::FileNotFound(format!(
				"File does not exist: {}", path.display()
			)));
		}

		// Check if it's a file (not a directory)
		if !path.is_file() {
			return Err(MinifyError::IoError(std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				format!("Path is not a file: {}", path.display())
			)));
		}

		// Check file size
		let metadata = fs::metadata(path)?;
		let file_size = metadata.len();
		
		debug!("File size: {} bytes", file_size);
		
		if file_size > MAX_FILE_SIZE {
			return Err(MinifyError::MemoryLimitExceeded(format!(
				"File too large: {} bytes (max: {} bytes)", 
				file_size, MAX_FILE_SIZE
			)));
		}

		// Check read permissions
		let mut file = File::open(path)?;
		
		// Read file contents
		let mut contents = Vec::with_capacity(file_size as usize);
		file.read_to_end(&mut contents)?;
		
		debug!("Successfully read {} bytes", contents.len());
		Ok(contents)
	}

	/// Safely write a file with permission checks and atomic operations
	pub fn safe_write_file(path: &Path, contents: &[u8]) -> Result<(), MinifyError> {
		debug!("Safe writing {} bytes to: {}", contents.len(), path.display());

		// Check if parent directory exists and is writable
		let parent = match path.parent() {
			Some(parent) if !parent.as_os_str().is_empty() => parent,
			_ => std::path::Path::new("."),
		};
		
		if !parent.exists() {
			return Err(MinifyError::PermissionError(format!(
				"Parent directory does not exist: {}", parent.display()
			)));
		}

		// Test write permissions by creating a temporary file
		let temp_path = parent.join(".omni_minify_test");
		match File::create(&temp_path) {
			Ok(_) => {
				// Clean up test file
				let _ = fs::remove_file(&temp_path);
			}
			Err(e) => {
				return Err(MinifyError::PermissionError(format!(
					"Cannot write to directory {}: {}", parent.display(), e
				)));
			}
		}

		// Check if output file already exists (for safety)
		if path.exists() {
			warn!("Output file already exists, will be overwritten: {}", path.display());
		}

		// Write file atomically using a temporary file
		let temp_path = path.with_extension("tmp");
		
		{
			let mut file = OpenOptions::new()
				.write(true)
				.create(true)
				.truncate(true)
				.open(&temp_path)?;
			
			file.write_all(contents)?;
			file.sync_all()?; // Ensure data is written to disk
		}

		// Atomically move temporary file to final location
		fs::rename(&temp_path, path)?;
		
		debug!("Successfully wrote file: {}", path.display());
		Ok(())
	}

	/// Validate file permissions before processing
	pub fn validate_file_permissions(input_path: &Path, output_path: &Path) -> Result<(), MinifyError> {
		debug!("Validating file permissions: input={}, output={}", 
			   input_path.display(), output_path.display());

		// Check input file exists and is readable
		if !input_path.exists() {
			return Err(MinifyError::FileNotFound(format!(
				"Input file does not exist: {}", input_path.display()
			)));
		}

		if !input_path.is_file() {
			return Err(MinifyError::IoError(std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				format!("Input path is not a file: {}", input_path.display())
			)));
		}

		// Test read access
		File::open(input_path).map_err(|e| {
			MinifyError::PermissionError(format!(
				"Cannot read input file {}: {}", input_path.display(), e
			))
		})?;

		// Check output directory is writable
		let output_parent = match output_path.parent() {
			Some(parent) if !parent.as_os_str().is_empty() => parent,
			_ => std::path::Path::new("."),
		};
		
		debug!("Output parent directory: '{}'", output_parent.display());
		
		if !output_parent.exists() {
			return Err(MinifyError::PermissionError(format!(
				"Output directory does not exist: {}", output_parent.display()
			)));
		}

		// Test write permissions
		let test_file = output_parent.join(".omni_minify_write_test");
		match File::create(&test_file) {
			Ok(_) => {
				let _ = fs::remove_file(&test_file);
			}
			Err(e) => {
				return Err(MinifyError::PermissionError(format!(
					"Cannot write to output directory {}: {}", 
					output_parent.display(), e
				)));
			}
		}

		debug!("File permissions validated successfully");
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use quickcheck::{TestResult, quickcheck};

	#[test]
	fn minify_stats_calculation() {
		let stats = MinifyStats::new(1000, 750);
		assert_eq!(stats.reduction_percentage(), 25.0);
		assert_eq!(stats.bytes_saved(), 250);
	}

	#[test]
	fn minify_stats_with_extra() {
		let stats = MinifyStats::new(1000, 750).with_extra("lines: 100 -> 75".to_string());
		assert_eq!(stats.extra, Some("lines: 100 -> 75".to_string()));
	}

	#[test]
	fn minify_stats_with_lines() {
		let stats = MinifyStats::new(1000, 750).with_lines(100, 75);
		assert_eq!(stats.lines, Some((100, 75)));
		assert_eq!(stats.line_reduction_percentage(), Some(25.0));
		assert_eq!(stats.lines_saved(), Some(25));
	}

	#[test]
	fn minify_stats_with_format_metrics() {
		let metrics = FormatMetrics::JavaScript {
			identifiers_renamed: 10,
			functions_inlined: 3,
			dead_code_removed: 2,
		};
		let stats = MinifyStats::new(1000, 750).with_format_metrics(metrics);
		
		let report = stats.generate_report();
		assert!(report.contains("Size: 1000 → 750 bytes"));
		assert!(report.contains("Saved: 250 bytes (25.0%)"));
		assert!(report.contains("Identifiers renamed: 10"));
		assert!(report.contains("Functions inlined: 3"));
		assert!(report.contains("Dead code blocks removed: 2"));
	}

	#[test]
	fn minify_stats_comprehensive_report() {
		let metrics = FormatMetrics::Css {
			selectors_merged: 5,
			rules_removed: 8,
			properties_collapsed: 12,
		};
		let stats = MinifyStats::new(2000, 1500)
			.with_lines(150, 120)
			.with_format_metrics(metrics)
			.with_extra("Custom optimization applied".to_string());
		
		let report = stats.generate_report();
		assert!(report.contains("Size: 2000 → 1500 bytes"));
		assert!(report.contains("Saved: 500 bytes (25.0%)"));
		assert!(report.contains("Lines: 150 → 120 (20.0% reduction)"));
		assert!(report.contains("Selectors merged: 5"));
		assert!(report.contains("Rules removed: 8"));
		assert!(report.contains("Properties collapsed: 12"));
		assert!(report.contains("Custom optimization applied"));
	}

	#[test]
	fn minify_stats_glb_metrics() {
		let metrics = FormatMetrics::Glb {
			triangles_before: 1000,
			triangles_after: 800,
			vertices_before: 500,
			vertices_after: 400,
		};
		let stats = MinifyStats::new(50000, 40000).with_format_metrics(metrics);
		
		let report = stats.generate_report();
		assert!(report.contains("Triangles: 1000 → 800 (20.0% reduction)"));
		assert!(report.contains("Vertices: 500 → 400 (20.0% reduction)"));
	}

	#[test]
	fn minify_error_display() {
		let error = MinifyError::ParseError("Invalid syntax".to_string());
		assert_eq!(error.to_string(), "Parse error: Invalid syntax");
	}

	#[test]
	fn minify_level_equality() {
		assert_eq!(MinifyLevel::Safe, MinifyLevel::Safe);
		assert_ne!(MinifyLevel::Safe, MinifyLevel::Aggressive);
	}

	// Mock plugin for testing the Minifier trait
	struct MockPlugin {
		name: String,
		can_detect: bool,
		stats: Option<MinifyStats>,
	}

	impl MockPlugin {
		fn new(name: String, can_detect: bool) -> Self {
			Self {
				name,
				can_detect,
				stats: None,
			}
		}
	}

	impl Minifier for MockPlugin {
		fn name(&self) -> &str {
			&self.name
		}

		fn detect(&self, _input_bytes: &[u8]) -> bool {
			self.can_detect
		}

		fn minify(&self, input_bytes: &[u8], _level: MinifyLevel) -> Result<Vec<u8>, MinifyError> {
			// Simple mock minification: just return input with stats
			let output = input_bytes.to_vec();
			Ok(output)
		}

		fn stats(&self) -> Option<MinifyStats> {
			self.stats.clone()
		}
	}

	// Plugin registry for testing
	struct PluginRegistry {
		plugins: Vec<Box<dyn Minifier>>,
	}

	impl PluginRegistry {
		fn new() -> Self {
			Self {
				plugins: Vec::new(),
			}
		}

		fn register(&mut self, plugin: Box<dyn Minifier>) {
			self.plugins.push(plugin);
		}

		fn find_plugin(&self, input_bytes: &[u8]) -> Option<&dyn Minifier> {
			self.plugins
				.iter()
				.find(|plugin| plugin.detect(input_bytes))
				.map(|plugin| plugin.as_ref())
		}

		fn get_plugin_names(&self) -> Vec<&str> {
			self.plugins.iter().map(|plugin| plugin.name()).collect()
		}
	}

	/**
	 * Feature: omni-minify-cli, Property 3: Plugin Registry Completeness
	 * Validates: Requirements 2.1
	 */
	#[test]
	fn property_plugin_registry_completeness() {
		fn prop(plugin_names: Vec<String>) -> TestResult {
			// Skip empty or very large inputs to keep test reasonable
			if plugin_names.is_empty() || plugin_names.len() > 10 {
				return TestResult::discard();
			}

			// Skip inputs with empty names
			if plugin_names.iter().any(|name| name.is_empty()) {
				return TestResult::discard();
			}

			let mut registry = PluginRegistry::new();

			// Register all plugins
			for name in &plugin_names {
				let plugin = Box::new(MockPlugin::new(name.clone(), true));
				registry.register(plugin);
			}

			// Verify all plugins are registered and accessible
			let registered_names = registry.get_plugin_names();

			// Property: All registered plugins should be findable by name
			for expected_name in &plugin_names {
				if !registered_names.contains(&expected_name.as_str()) {
					return TestResult::failed();
				}
			}

			// Property: Registry should contain exactly the plugins we registered
			if registered_names.len() != plugin_names.len() {
				return TestResult::failed();
			}

			TestResult::passed()
		}

		quickcheck(prop as fn(Vec<String>) -> TestResult);
	}

	/**
	 * Feature: omni-minify-cli, Property 8: Statistics Accuracy
	 * Validates: Requirements 9.1
	 */
	#[test]
	fn property_statistics_accuracy() {
		fn prop(before_bytes: usize, after_bytes: usize) -> TestResult {
			use log::debug;
			
			// Skip unreasonable inputs to keep test meaningful
			if before_bytes > 1_000_000 || after_bytes > 1_000_000 {
				return TestResult::discard();
			}

			debug!("Testing statistics accuracy: {} -> {} bytes", before_bytes, after_bytes);

			let stats = MinifyStats::new(before_bytes, after_bytes);

			// Property 1: Byte counts should match exactly what was provided
			if stats.before_bytes != before_bytes || stats.after_bytes != after_bytes {
				debug!("Byte count mismatch: expected ({}, {}), got ({}, {})", 
					   before_bytes, after_bytes, stats.before_bytes, stats.after_bytes);
				return TestResult::failed();
			}

			// Property 2: Bytes saved calculation should be accurate
			let expected_saved = before_bytes.saturating_sub(after_bytes);
			let actual_saved = stats.bytes_saved();
			if actual_saved != expected_saved {
				debug!("Bytes saved mismatch: expected {}, got {}", expected_saved, actual_saved);
				return TestResult::failed();
			}

			// Property 3: Percentage calculation should be mathematically correct
			let expected_percentage = if before_bytes == 0 {
				0.0  // By definition, when before_bytes is 0, percentage is 0
			} else {
				((before_bytes - after_bytes) as f64 / before_bytes as f64) * 100.0
			};
			let actual_percentage = stats.reduction_percentage();
			
			// Use epsilon comparison for floating point
			let epsilon = 0.0001;
			if (actual_percentage - expected_percentage).abs() > epsilon {
				debug!("Percentage mismatch: expected {:.4}%, got {:.4}%", 
					   expected_percentage, actual_percentage);
				return TestResult::failed();
			}

			// Property 4: When before_bytes > 0, percentage should be in valid mathematical range
			if before_bytes > 0 {
				// Percentage can be negative (size increase) or positive (size decrease)
				// But should be mathematically consistent
				let manual_calc = ((before_bytes as i64 - after_bytes as i64) as f64 / before_bytes as f64) * 100.0;
				if (actual_percentage - manual_calc).abs() > epsilon {
					debug!("Mathematical inconsistency: manual calc {:.4}%, actual {:.4}%", 
						   manual_calc, actual_percentage);
					return TestResult::failed();
				}
			}

			debug!("Statistics validation passed: {:.1}% reduction, {} bytes saved", 
				   actual_percentage, actual_saved);
			TestResult::passed()
		}

		quickcheck(prop as fn(usize, usize) -> TestResult);
	}

	/**
	 * Feature: omni-minify-cli, Property 3: Plugin Registry Completeness (Detection)
	 * Validates: Requirements 2.1
	 */
	#[test]
	fn property_plugin_detection_consistency() {
		fn prop(input_data: Vec<u8>) -> TestResult {
			// Skip very large inputs
			if input_data.len() > 1000 {
				return TestResult::discard();
			}

			let mut registry = PluginRegistry::new();

			// Register plugins with different detection capabilities
			registry.register(Box::new(MockPlugin::new("always_detect".to_string(), true)));
			registry.register(Box::new(MockPlugin::new("never_detect".to_string(), false)));

			// Property: Detection should be consistent across multiple calls
			let first_result = registry.find_plugin(&input_data);
			let second_result = registry.find_plugin(&input_data);

			match (first_result, second_result) {
				(Some(first), Some(second)) => {
					// Should find the same plugin (by name) both times
					TestResult::from_bool(first.name() == second.name())
				}
				(None, None) => TestResult::passed(),
				_ => TestResult::failed(), // Inconsistent results
			}
		}

		quickcheck(prop as fn(Vec<u8>) -> TestResult);
	}

	#[test]
	fn test_plugin_registry_basic_functionality() {
		let mut registry = PluginRegistry::new();

		// Test empty registry
		assert_eq!(registry.get_plugin_names().len(), 0);
		assert!(registry.find_plugin(b"test").is_none());

		// Add a plugin
		registry.register(Box::new(MockPlugin::new("test_plugin".to_string(), true)));

		// Verify plugin is registered
		let names = registry.get_plugin_names();
		assert_eq!(names.len(), 1);
		assert_eq!(names[0], "test_plugin");

		// Verify plugin can be found
		let found = registry.find_plugin(b"test");
		assert!(found.is_some());
		assert_eq!(found.unwrap().name(), "test_plugin");
	}
}
