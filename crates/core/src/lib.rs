use std::fmt;

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
}

impl MinifyStats {
	/// Create new stats with before and after byte counts
	pub fn new(before_bytes: usize, after_bytes: usize) -> Self {
		Self {
			before_bytes,
			after_bytes,
			extra: None,
		}
	}

	/// Add format-specific extra information
	pub fn with_extra(mut self, extra: String) -> Self {
		self.extra = Some(extra);
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
}

impl fmt::Display for MinifyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			MinifyError::ParseError(msg) => write!(f, "Parse error: {}", msg),
			MinifyError::IoError(err) => write!(f, "I/O error: {}", err),
			MinifyError::UnsupportedFormat => write!(f, "Unsupported file format"),
			MinifyError::PluginError(msg) => write!(f, "Plugin error: {}", msg),
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
		MinifyError::IoError(err)
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
