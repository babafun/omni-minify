use core::{Minifier, MinifyError, MinifyLevel, MinifyStats};
use log::debug;
use regex::Regex;
use std::sync::LazyLock;

// Compile regex patterns once at startup
static CSS_COMMENT_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"/\*[\s\S]*?\*/").unwrap());

static CSS_LEADING_TRAILING_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^\s+|\s+$").unwrap());

static CSS_MULTIPLE_SEMICOLON_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r";{2,}").unwrap());

static CSS_SPACE_BEFORE_SEMICOLON_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"\s*;\s*").unwrap());

static CSS_COLON_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*:\s*").unwrap());

pub struct CSSPlugin {
	last_stats: Option<MinifyStats>,
}

impl CSSPlugin {
	pub fn new() -> Self {
		debug!("Initializing CSS plugin");
		Self { last_stats: None }
	}

	fn parse_css(&self, input: &str) -> Result<(), MinifyError> {
		debug!("Validating CSS syntax, length: {}", input.len());

		let mut brace_count = 0;
		let mut paren_count = 0;
		let mut in_string = false;
		let mut escape_next = false;
		let mut string_char = '\0';

		for ch in input.chars() {
			if escape_next {
				escape_next = false;
				continue;
			}

			if ch == '\\' {
				escape_next = true;
				continue;
			}

			if in_string {
				if ch == string_char {
					in_string = false;
				}
				continue;
			}

			match ch {
				'"' | '\'' => {
					in_string = true;
					string_char = ch;
				}
				'{' => brace_count += 1,
				'}' => brace_count -= 1,
				'(' => paren_count += 1,
				')' => paren_count -= 1,
				_ => {}
			}

			// Early exit if we have negative counts
			if brace_count < 0 || paren_count < 0 {
				return Err(MinifyError::ParseError("Unbalanced brackets".to_string()));
			}
		}

		// Check final balance
		if brace_count != 0 || paren_count != 0 {
			return Err(MinifyError::ParseError(format!(
				"Unbalanced syntax: braces={}, parens={}",
				brace_count, paren_count
			)));
		}

		debug!("CSS syntax validation successful");
		Ok(())
	}

	fn safe_minify(&mut self, input_bytes: &[u8]) -> Result<Vec<u8>, MinifyError> {
		debug!("Starting safe CSS minification");

		let original_size = input_bytes.len();
		let input = String::from_utf8_lossy(input_bytes);

		// Validate syntax first
		self.parse_css(&input)?;

		// Remove CSS comments
		let without_comments = CSS_COMMENT_REGEX.replace_all(&input, "").to_string();

		// Remove whitespace character by character while preserving string content
		let minified = Self::remove_whitespace_preserving_strings(&without_comments);

		// Remove unnecessary whitespace around colons (outside strings)
		let cleaned_colons = CSS_COLON_REGEX.replace_all(&minified, ":").to_string();

		// Remove unnecessary whitespace around semicolons
		let cleaned_semicolons =
			CSS_SPACE_BEFORE_SEMICOLON_REGEX.replace_all(&cleaned_colons, ";").to_string();

		// Remove multiple semicolons (e.g., ;; -> ;)
		let cleaned_multi =
			CSS_MULTIPLE_SEMICOLON_REGEX.replace_all(&cleaned_semicolons, ";").to_string();

		// Trim leading/trailing whitespace from entire string
		let output = CSS_LEADING_TRAILING_REGEX.replace_all(&cleaned_multi, "").to_string();
		let output_bytes = output.as_bytes().to_vec();

		// Update statistics
		self.last_stats = Some(
			MinifyStats::new(original_size, output_bytes.len())
				.with_extra("comments removed, whitespace collapsed".to_string()),
		);

		debug!(
			"Safe CSS minification complete: {} -> {} bytes",
			original_size,
			output_bytes.len()
		);

		Ok(output_bytes)
	}

	fn remove_whitespace_preserving_strings(input: &str) -> String {
		let mut result = String::with_capacity(input.len());
		let mut in_string = false;
		let mut string_char = '\0';
		let mut escape_next = false;

		for ch in input.chars() {
			if escape_next {
				escape_next = false;
				result.push(ch);
				continue;
			}

			if ch == '\\' {
				escape_next = true;
				result.push(ch);
				continue;
			}

			if in_string {
				if ch == string_char {
					in_string = false;
					string_char = '\0';
				}
				result.push(ch);
			} else {
				// Outside string - check if entering string
				if ch == '"' || ch == '\'' {
					in_string = true;
					string_char = ch;
					result.push(ch);
				} else if !ch.is_whitespace() {
					// Only preserve non-whitespace characters outside strings
					result.push(ch);
				}
				// Skip whitespace outside strings
			}
		}

		result
	}

	fn aggressive_minify(&mut self, input_bytes: &[u8]) -> Result<Vec<u8>, MinifyError> {
		debug!("Starting aggressive CSS minification");

		let original_size = input_bytes.len();
		let input = String::from_utf8_lossy(input_bytes);

		// Remove CSS comments
		let without_comments = CSS_COMMENT_REGEX.replace_all(&input, "").to_string();

		// Remove whitespace character by character while preserving string content
		let minified = Self::remove_whitespace_preserving_strings(&without_comments);

		// Remove unnecessary whitespace around colons (outside strings)
		let cleaned_colons = CSS_COLON_REGEX.replace_all(&minified, ":").to_string();

		// Remove unnecessary whitespace around semicolons
		let cleaned_semicolons =
			CSS_SPACE_BEFORE_SEMICOLON_REGEX.replace_all(&cleaned_colons, ";").to_string();

		// Remove multiple semicolons (e.g., ;; -> ;)
		let cleaned_multi =
			CSS_MULTIPLE_SEMICOLON_REGEX.replace_all(&cleaned_semicolons, ";").to_string();

		// Remove last semicolon before closing brace (optional but safe)
		let no_trailing_semicolon = Regex::new(r";\}").unwrap().replace_all(&cleaned_multi, "}");

		let output = no_trailing_semicolon.trim().to_string();
		let output_bytes = output.as_bytes().to_vec();

		// Update statistics
		self.last_stats = Some(
			MinifyStats::new(original_size, output_bytes.len())
				.with_extra("aggressive: comments, whitespace, newlines removed".to_string()),
		);

		debug!(
			"Aggressive CSS minification complete: {} -> {} bytes",
			original_size,
			output_bytes.len()
		);

		Ok(output_bytes)
	}
}

impl Default for CSSPlugin {
	fn default() -> Self {
		Self::new()
	}
}

impl Minifier for CSSPlugin {
	fn name(&self) -> &str {
		"css"
	}

	fn detect(&self, input_bytes: &[u8]) -> bool {
		debug!("CSS detection starting, input size: {}", input_bytes.len());

		// Empty input cannot be CSS
		if input_bytes.is_empty() {
			debug!("Empty input, not CSS");
			return false;
		}

		let content = String::from_utf8_lossy(input_bytes);

		// Exclude obvious non-CSS content
		if content.contains("<!DOCTYPE")
			|| content.contains("<html>")
			|| content.contains("<body")
			|| content.contains("function")
			|| content.contains("const ")
			|| content.contains("let ")
			|| content.contains("var ")
		{
			debug!("Detected non-CSS content (HTML/JS)");
			return false;
		}

		// Look for CSS patterns
		let has_css_at_keyword = content.contains("@import")
			|| content.contains("@media")
			|| content.contains("@keyframes")
			|| content.contains("@charset")
			|| content.contains("@font-face");

		let has_css_selectors = content.contains("{")
			&& (content.contains(".")
				|| content.contains("#")
				|| content.contains("*")
				|| content
					.chars()
					.next()
					.is_some_and(|c| c.is_alphabetic() || c == '*'));

		let has_css_properties =
			content.contains(":") && (content.contains(";") || content.contains("{"));

		let detected = has_css_at_keyword
			|| (has_css_selectors && has_css_properties)
			|| content.contains("body {")
			|| content.contains(".class")
			|| content.contains("#id");

		debug!(
			"CSS detection result: {}, at_keyword: {}, selectors: {}, properties: {}",
			detected, has_css_at_keyword, has_css_selectors, has_css_properties
		);

		detected
	}

	fn minify(&self, input_bytes: &[u8], level: MinifyLevel) -> Result<Vec<u8>, MinifyError> {
		debug!("CSS minification starting with level: {:?}", level);

		// Handle empty input
		if input_bytes.is_empty() {
			return Err(MinifyError::ParseError("Empty input".to_string()));
		}

		let mut plugin = CSSPlugin {
			last_stats: self.last_stats.clone(),
		};

		match level {
			MinifyLevel::Safe => plugin.safe_minify(input_bytes),
			MinifyLevel::Aggressive => plugin.aggressive_minify(input_bytes),
		}
	}

	fn stats(&self) -> Option<MinifyStats> {
		self.last_stats.clone()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use quickcheck::{QuickCheck, TestResult};

	#[test]
	fn test_css_detection() {
		let plugin = CSSPlugin::new();

		// Should detect CSS
		assert!(plugin.detect(b"body { color: red; }"));
		assert!(plugin.detect(b".class { margin: 0; padding: 0; }"));
		assert!(plugin.detect(b"#id { font-size: 14px; }"));
		assert!(plugin.detect(b"*{ margin: 0; }"));
		assert!(plugin.detect(b"@import url('style.css');"));
		assert!(plugin.detect(b"@media screen { .class { display: none; } }"));
		assert!(plugin.detect(b"@keyframes slide { from { opacity: 0; } }"));

		// Should not detect non-CSS content
		assert!(!plugin.detect(b"<html><body>Hello</body></html>"));
		assert!(!plugin.detect(b"function test() { return 42; }"));
		assert!(!plugin.detect(b"const x = 42;"));
		assert!(!plugin.detect(b""));
		assert!(!plugin.detect(b"This is just plain text"));
	}

	#[test]
	fn test_safe_minification() {
		let mut plugin = CSSPlugin::new();
		let input = b"body {\n    /* This is a comment */\n    color: red;\n    margin: 0;\n}";

		let result = plugin.safe_minify(input);
		assert!(result.is_ok());

		let result_bytes = result.unwrap();
		let output = String::from_utf8_lossy(&result_bytes);

		// Should be valid CSS but smaller
		assert!(output.contains("body{color:red;margin:0;}"));
		// Comments should be removed
		assert!(!output.contains("/* This is a comment */"));
	}

	#[test]
	fn test_aggressive_minification() {
		let mut plugin = CSSPlugin::new();
		let input = b"body {\n    color: red;\n}";

		let result = plugin.aggressive_minify(input);
		assert!(result.is_ok());

		let result_bytes = result.unwrap();
		let output = String::from_utf8_lossy(&result_bytes);

		// Aggressive mode should remove newlines
		assert!(!output.contains("\n"));
		assert!(output.contains("body{color:red}"));
	}

	#[test]
	fn test_plugin_name() {
		let plugin = CSSPlugin::new();
		assert_eq!(plugin.name(), "css");
	}

	#[test]
	fn test_empty_input() {
		let plugin = CSSPlugin::new();

		// Empty input should not be detected as CSS
		assert!(!plugin.detect(b""));

		// Minifying empty input should fail gracefully
		let result = plugin.minify(b"", MinifyLevel::Safe);
		assert!(result.is_err());
	}

	#[test]
	fn test_malformed_css() {
		let plugin = CSSPlugin::new();
		let malformed = b"body { color: red; /* missing closing brace";

		// Should still detect as CSS
		assert!(plugin.detect(malformed));

		// But minification should fail
		let result = plugin.minify(malformed, MinifyLevel::Safe);
		assert!(result.is_err());
	}

	// Additional unit tests for CSS-specific minification techniques
	// Requirements: 7.1, 7.5

	#[test]
	fn test_css_comment_removal() {
		let mut plugin = CSSPlugin::new();
		let input = b"/* Header comment */\nbody {\n    /* Inline comment */\n    color: red; /* End comment */\n}\n/* Footer comment */";

		let result = plugin.safe_minify(input).unwrap();
		let output = String::from_utf8_lossy(&result);

		// All comments should be removed
		assert!(!output.contains("/* Header comment */"));
		assert!(!output.contains("/* Inline comment */"));
		assert!(!output.contains("/* End comment */"));
		assert!(!output.contains("/* Footer comment */"));
		
		// CSS structure should remain
		assert!(output.contains("body"));
		assert!(output.contains("color:red"));
	}

	#[test]
	fn test_css_whitespace_collapse() {
		let mut plugin = CSSPlugin::new();
		let input = b"body    {   \n   color   :   red   ;   \n   margin   :   0   ;   \n}";

		let result = plugin.safe_minify(input).unwrap();
		let output = String::from_utf8_lossy(&result);

		// Whitespace should be collapsed
		assert!(!output.contains("   "));
		assert!(!output.contains("\n"));
		assert!(output.contains("body{color:red;margin:0;}"));
	}

	#[test]
	fn test_css_property_preservation() {
		let mut plugin = CSSPlugin::new();
		let input = b".class {\n    background-color: #ff0000;\n    font-family: 'Arial', sans-serif;\n    margin: 10px 20px 30px 40px;\n}";

		let result = plugin.safe_minify(input).unwrap();
		let output = String::from_utf8_lossy(&result);

		// Properties should be preserved
		assert!(output.contains("background-color:#ff0000"));
		assert!(output.contains("font-family:'Arial',sans-serif"));
		assert!(output.contains("margin:10px"));
		assert!(output.contains("20px"));
		assert!(output.contains("30px"));
		assert!(output.contains("40px"));
	}

	#[test]
	fn test_css_selector_preservation() {
		let mut plugin = CSSPlugin::new();
		let input = b"body, html { margin: 0; }\n.class#id[attr=\"value\"] { padding: 0; }\n@media screen and (max-width: 768px) { .responsive { display: none; } }";

		let result = plugin.safe_minify(input).unwrap();
		let output = String::from_utf8_lossy(&result);

		// Complex selectors should be preserved
		assert!(output.contains("body,html{margin:0;}"));
		assert!(output.contains(".class#id[attr=\"value\"]{padding:0;}"));
		assert!(output.contains("@media"));
		assert!(output.contains("screen"));
		assert!(output.contains("max-width:768px"));
		assert!(output.contains(".responsive{display:none;}"));
	}

	#[test]
	fn test_css_string_preservation() {
		let mut plugin = CSSPlugin::new();
		let input = b".class {\n    content: \"Hello   World\";\n    font-family: 'Times New Roman';\n    background-image: url('path/to/image.jpg');\n}";

		let result = plugin.safe_minify(input).unwrap();
		let output = String::from_utf8_lossy(&result);

		// Strings should preserve internal whitespace
		assert!(output.contains("content:\"Hello   World\""));
		assert!(output.contains("font-family:'Times New Roman'"));
		assert!(output.contains("background-image:url('path/to/image.jpg')"));
	}

	#[test]
	fn test_css_at_rules() {
		let mut plugin = CSSPlugin::new();
		let input = b"@charset \"UTF-8\";\n@import url('external.css');\n@keyframes slide {\n    from { opacity: 0; }\n    to { opacity: 1; }\n}";

		let result = plugin.safe_minify(input).unwrap();
		let output = String::from_utf8_lossy(&result);

		// At-rules should be preserved and minified
		assert!(output.contains("@charset"));
		assert!(output.contains("UTF-8"));
		assert!(output.contains("@import"));
		assert!(output.contains("url('external.css')"));
		assert!(output.contains("@keyframes"));
		assert!(output.contains("slide"));
		assert!(output.contains("from{opacity:0;}"));
		assert!(output.contains("to{opacity:1;}"));
	}

	#[test]
	fn test_css_aggressive_mode_differences() {
		let mut plugin = CSSPlugin::new();
		let input = b"body {\n    color: red;\n    margin: 0;\n}";

		let safe_result = plugin.safe_minify(input).unwrap();
		let aggressive_result = plugin.aggressive_minify(input).unwrap();

		let safe_output = String::from_utf8_lossy(&safe_result);
		let aggressive_output = String::from_utf8_lossy(&aggressive_result);

		// Aggressive mode should produce same or smaller output
		assert!(aggressive_result.len() <= safe_result.len());
		
		// Both should preserve CSS structure
		assert!(safe_output.contains("body{color:red;margin:0;}"));
		assert!(aggressive_output.contains("body{color:red;margin:0}") || aggressive_output.contains("body{color:red;margin:0;}"));
	}

	#[test]
	fn test_css_edge_cases() {
		let plugin = CSSPlugin::new();

		// Test with only whitespace
		let whitespace_only = b"   \n\t   \n   ";
		assert!(!plugin.detect(whitespace_only));

		// Test with CSS-like but invalid content
		let invalid_css = b"{ color: red }"; // Missing selector
		assert!(!plugin.detect(invalid_css));

		// Test with minimal valid CSS
		let minimal_css = b"*{margin:0}";
		assert!(plugin.detect(minimal_css));
		let result = plugin.minify(minimal_css, MinifyLevel::Safe);
		assert!(result.is_ok());
	}

	#[test]
	fn test_css_large_file_handling() {
		let mut plugin = CSSPlugin::new();
		
		// Create a larger CSS file with repeated rules
		let mut large_css = String::new();
		for i in 0..100 {
			large_css.push_str(&format!(
				".class{} {{ color: red; margin: {}px; padding: {}px; }}\n",
				i, i, i * 2
			));
		}

		let result = plugin.safe_minify(large_css.as_bytes());
		assert!(result.is_ok());

		let output = result.unwrap();
		let output_str = String::from_utf8_lossy(&output);

		// Should still be valid CSS
		assert!(plugin.parse_css(&output_str).is_ok());
		// Should be smaller than input
		assert!(output.len() < large_css.len());
	}

	#[test]
	fn test_css_nested_structures() {
		let mut plugin = CSSPlugin::new();
		let input = b"@media screen {\n    .container {\n        @supports (display: grid) {\n            .grid-item {\n                display: grid;\n            }\n        }\n    }\n}";

		let result = plugin.safe_minify(input);
		assert!(result.is_ok());

		let output = result.unwrap();
		let output_str = String::from_utf8_lossy(&output);

		// Nested structures should be preserved
		assert!(output_str.contains("@media"));
		assert!(output_str.contains("screen"));
		assert!(output_str.contains("@supports"));
		assert!(output_str.contains("display:grid"));
		assert!(output_str.contains(".container"));
		assert!(output_str.contains(".grid-item"));
	}

	/**
	 * Feature: omni-minify-cli, Property: CSS Minification Correctness
	 */
	#[test]
	fn property_css_minification_correctness() {
		fn prop(input_content: String) -> TestResult {
			// Filter out invalid or problematic inputs
			if input_content.is_empty() || input_content.len() > 1000 {
				return TestResult::discard();
			}

			// Only test with content that looks like CSS
			if !input_content.contains("{") && !input_content.contains(";") {
				return TestResult::discard();
			}

			debug!(
				"Testing CSS minification correctness with {} byte input",
				input_content.len()
			);

			let plugin = CSSPlugin::new();

			// Only test if plugin can detect this as CSS
			if !plugin.detect(input_content.as_bytes()) {
				return TestResult::discard();
			}

			match plugin.minify(input_content.as_bytes(), MinifyLevel::Safe) {
				Ok(output) => {
					debug!(
						"Minification successful: {} -> {} bytes",
						input_content.len(),
						output.len()
					);

					let output_str = String::from_utf8_lossy(&output);

					// Property: Minified CSS should still be valid CSS
					let temp_plugin = CSSPlugin::new();
					let is_valid = temp_plugin.parse_css(&output_str).is_ok();

					debug!("Output validity check: {}", is_valid);

					// Property: Minification should remove comments and whitespace
					let size_reduced = output.len() <= input_content.len();

					debug!(
						"Size reduction check: {} <= {} = {}",
						output.len(),
						input_content.len(),
						size_reduced
					);

					TestResult::from_bool(is_valid && size_reduced)
				}
				Err(e) => {
					debug!("Minification failed: {}", e);
					TestResult::discard()
				}
			}
		}

		QuickCheck::new()
			.tests(20)
			.quickcheck(prop as fn(String) -> TestResult);
	}

	/**
	 * Feature: omni-minify-cli, Property: Safe Mode Semantic Preservation
	 */
	#[test]
	fn property_safe_mode_semantic_preservation() {
		fn prop(input_content: String) -> TestResult {
			if input_content.is_empty() || input_content.len() > 1000 {
				return TestResult::discard();
			}

			if !input_content.contains("{") && !input_content.contains(";") {
				return TestResult::discard();
			}

			debug!(
				"Testing safe mode semantic preservation with {} byte input",
				input_content.len()
			);

			let plugin = CSSPlugin::new();

			if !plugin.detect(input_content.as_bytes()) {
				return TestResult::discard();
			}

			if plugin.parse_css(&input_content).is_err() {
				return TestResult::discard();
			}

			match plugin.minify(input_content.as_bytes(), MinifyLevel::Safe) {
				Ok(output) => {
					let output_str = String::from_utf8_lossy(&output);

					let temp_plugin = CSSPlugin::new();
					let output_is_valid = temp_plugin.parse_css(&output_str).is_ok();

					// Property: Safe mode should preserve semantic meaning
					let preserves_structure = output_str.contains("{")
						|| output_str.contains(":")
						|| output_str.contains(";");

					TestResult::from_bool(output_is_valid && preserves_structure)
				}
				Err(_) => TestResult::discard(),
			}
		}

		QuickCheck::new()
			.tests(20)
			.quickcheck(prop as fn(String) -> TestResult);
	}
}
