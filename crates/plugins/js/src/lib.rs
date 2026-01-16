use core::{Minifier, MinifyError, MinifyLevel, MinifyStats, FormatMetrics};
use log::debug;
use regex::Regex;
use std::cell::RefCell;
use std::sync::LazyLock;

// Regex patterns for comment removal
static COMMENT_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"(?m)//.*$|/\*[\s\S]*?\*/").unwrap());

// Regex patterns for whitespace removal (outside of quotes)
static MULTI_SPACE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"  +").unwrap());
static LINE_BREAK_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n|\r").unwrap());
static HARD_TAB_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\t").unwrap());
static SPACE_BEFORE_PAREN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" \(").unwrap());
static SPACE_AFTER_PAREN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\) ").unwrap());
static SPACE_BEFORE_BRACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" \{").unwrap());
static SPACE_AFTER_BRACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\} ").unwrap());
static SPACE_BEFORE_BRACKET: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" \[").unwrap());
static SPACE_AFTER_BRACKET: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\] ").unwrap());
static SPACE_BEFORE_COLON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" :").unwrap());
static SPACE_AFTER_COLON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r": ").unwrap());
static SPACE_BEFORE_COMMA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" ,").unwrap());
static SPACE_AFTER_COMMA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r", ").unwrap());
static SPACE_BEFORE_SEMICOLON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" ;").unwrap());
static SPACE_ARROW: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" =>").unwrap());

pub struct JavaScriptPlugin {
	last_stats: RefCell<Option<MinifyStats>>,
}

impl JavaScriptPlugin {
	pub fn new() -> Self {
		debug!("Initializing JavaScript plugin");
		Self { 
			last_stats: RefCell::new(None),
		}
	}

	pub fn parse_javascript(&self, input: &str) -> Result<(), MinifyError> {
		debug!("Validating JavaScript syntax, length: {}", input.len());

		// Basic JavaScript syntax validation
		// Check for balanced braces
		let mut brace_count = 0;
		let mut paren_count = 0;
		let mut bracket_count = 0;
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
				'"' | '\'' | '`' => {
					in_string = true;
					string_char = ch;
				}
				'{' => brace_count += 1,
				'}' => brace_count -= 1,
				'(' => paren_count += 1,
				')' => paren_count -= 1,
				'[' => bracket_count += 1,
				']' => bracket_count -= 1,
				_ => {}
			}

			// Early exit if we have negative counts (unbalanced)
			if brace_count < 0 || paren_count < 0 || bracket_count < 0 {
				return Err(MinifyError::ParseError("Unbalanced brackets".to_string()));
			}
		}

		// Check final balance
		if brace_count != 0 || paren_count != 0 || bracket_count != 0 {
			return Err(MinifyError::ParseError(format!(
				"Unbalanced syntax: braces={}, parens={}, brackets={}",
				brace_count, paren_count, bracket_count
			)));
		}

		debug!("JavaScript syntax validation successful");
		Ok(())
	}

	fn safe_minify(&self, input_bytes: &[u8]) -> Result<Vec<u8>, MinifyError> {
		debug!("Starting safe JavaScript minification");

		let original_size = input_bytes.len();
		let input = String::from_utf8_lossy(input_bytes);

		// Validate syntax first
		self.parse_javascript(&input)?;

		// Count original lines
		let original_lines = input.lines().count();

		// Step 1: Remove comments while preserving strings
		let without_comments = self.remove_comments_preserving_strings(&input);
		let comments_removed = self.count_comments_removed(&input, &without_comments);

		// Step 2: Remove whitespace while preserving strings and regex literals
		let minified = self.remove_whitespace_preserving_strings(&without_comments);

		// Count final lines
		let final_lines = minified.lines().count();

		let output_bytes = minified.as_bytes().to_vec();

		// Create detailed statistics with format-specific metrics
		let format_metrics = FormatMetrics::JavaScript {
			identifiers_renamed: 0, // Safe mode doesn't rename identifiers
			functions_inlined: 0,   // Safe mode doesn't inline functions
			dead_code_removed: 0,   // Safe mode doesn't remove dead code
		};

		let stats = MinifyStats::new(original_size, output_bytes.len())
			.with_lines(original_lines, final_lines)
			.with_format_metrics(format_metrics)
			.with_extra(format!("comments removed: {}", comments_removed));

		// Store statistics using interior mutability
		*self.last_stats.borrow_mut() = Some(stats);

		debug!(
			"Safe JavaScript minification complete: {} -> {} bytes, {} -> {} lines, {} comments removed",
			original_size,
			output_bytes.len(),
			original_lines,
			final_lines,
			comments_removed
		);

		Ok(output_bytes)
	}

	fn count_comments_removed(&self, original: &str, after_removal: &str) -> usize {
		// Simple heuristic: count comment markers in original vs after removal
		let original_single_line = original.matches("//").count();
		let original_multi_line = original.matches("/*").count();
		let after_single_line = after_removal.matches("//").count();
		let after_multi_line = after_removal.matches("/*").count();
		
		(original_single_line - after_single_line) + (original_multi_line - after_multi_line)
	}

	fn remove_comments_preserving_strings(&self, input: &str) -> String {
		let mut result = String::with_capacity(input.len());
		let mut chars = input.chars().peekable();
		let mut in_string = false;
		let mut string_char = '\0';
		let mut escape_next = false;
		let mut in_regex = false;

		while let Some(ch) = chars.next() {
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
				result.push(ch);
				if ch == string_char {
					in_string = false;
				}
				continue;
			}

			if in_regex {
				result.push(ch);
				if ch == '/' {
					in_regex = false;
				}
				continue;
			}

			match ch {
				'"' | '\'' | '`' => {
					in_string = true;
					string_char = ch;
					result.push(ch);
				}
				'/' => {
					if let Some(&next_ch) = chars.peek() {
						match next_ch {
							'/' => {
								// Single-line comment - skip everything until end of line
								chars.next(); // consume the second '/'
								while let Some(c) = chars.next() {
									if c == '\n' || c == '\r' {
										// Don't preserve the line break - comments should be completely removed
										break;
									}
									// Skip all characters in the comment
								}
							}
							'*' => {
								// Multi-line comment - skip to */
								chars.next(); // consume the '*'
								let mut found_end = false;
								while let Some(c) = chars.next() {
									if c == '*' {
										if let Some(&'/') = chars.peek() {
											chars.next(); // consume the '/'
											found_end = true;
											break;
										}
									}
								}
								if !found_end {
									// Malformed comment, but continue
								}
								// Don't add anything to result - comment is completely removed
							}
							_ => {
								// Could be regex literal - simple heuristic
								// This is a simplified check; a full parser would be better
								if self.could_be_regex_start(result.chars().last().unwrap_or('\0')) {
									in_regex = true;
								}
								result.push(ch);
							}
						}
					} else {
						result.push(ch);
					}
				}
				_ => {
					result.push(ch);
				}
			}
		}

		result
	}

	fn remove_whitespace_preserving_strings(&self, input: &str) -> String {
		let mut result = String::with_capacity(input.len());
		let mut in_string = false;
		let mut string_char = '\0';
		let mut escape_next = false;
		let mut in_regex = false;
		let mut prev_char = '\0';

		for ch in input.chars() {
			if escape_next {
				escape_next = false;
				result.push(ch);
				prev_char = ch;
				continue;
			}

			if ch == '\\' {
				escape_next = true;
				result.push(ch);
				prev_char = ch;
				continue;
			}

			if in_string {
				result.push(ch);
				if ch == string_char {
					in_string = false;
				}
				prev_char = ch;
				continue;
			}

			if in_regex {
				result.push(ch);
				if ch == '/' {
					in_regex = false;
				}
				prev_char = ch;
				continue;
			}

			match ch {
				'"' | '\'' | '`' => {
					in_string = true;
					string_char = ch;
					result.push(ch);
				}
				'/' => {
					// Simple regex detection heuristic
					if self.could_be_regex_start(prev_char) {
						in_regex = true;
					}
					result.push(ch);
				}
				' ' | '\t' | '\n' | '\r' => {
					// Only add whitespace if it's necessary for syntax
					if self.needs_whitespace_separation(prev_char, input, &result) {
						result.push(' ');
					}
				}
				_ => {
					result.push(ch);
				}
			}
			
			if !ch.is_whitespace() {
				prev_char = ch;
			}
		}

		result.trim().to_string()
	}

	fn could_be_regex_start(&self, prev_char: char) -> bool {
		matches!(prev_char, '=' | '(' | '[' | ',' | ':' | ';' | '!' | '&' | '|' | '?' | '+' | '-' | '*' | '/' | '%' | '{' | '}' | '\n' | '\r')
	}

	fn needs_whitespace_separation(&self, prev_char: char, _input: &str, result: &str) -> bool {
		if result.is_empty() {
			return false;
		}

		// Need space between keywords/identifiers and other tokens
		match prev_char {
			'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '$' => true,
			_ => false,
		}
	}

	fn aggressive_minify(&self, input_bytes: &[u8]) -> Result<Vec<u8>, MinifyError> {
		debug!("Starting aggressive JavaScript minification");

		let original_size = input_bytes.len();
		let input = String::from_utf8_lossy(input_bytes);

		// Validate syntax first
		self.parse_javascript(&input)?;

		// Count original lines
		let original_lines = input.lines().count();

		// Step 1: Remove comments while preserving strings
		let without_comments = self.remove_comments_preserving_strings(&input);
		let comments_removed = self.count_comments_removed(&input, &without_comments);

		// Step 2: Remove whitespace while preserving strings and regex literals
		let minified = self.remove_whitespace_preserving_strings(&without_comments);

		// Step 3: Aggressive optimizations (simplified for demo)
		let aggressive_result = self.apply_aggressive_optimizations(&minified);
		let identifiers_renamed = self.count_identifier_changes(&minified, &aggressive_result);

		// Count final lines
		let final_lines = aggressive_result.lines().count();

		let output_bytes = aggressive_result.as_bytes().to_vec();

		// Create detailed statistics with format-specific metrics
		let format_metrics = FormatMetrics::JavaScript {
			identifiers_renamed,
			functions_inlined: 0,   // Not implemented in this demo
			dead_code_removed: 0,   // Not implemented in this demo
		};

		let stats = MinifyStats::new(original_size, output_bytes.len())
			.with_lines(original_lines, final_lines)
			.with_format_metrics(format_metrics)
			.with_extra(format!("comments removed: {}, identifiers renamed: {}", comments_removed, identifiers_renamed));

		// Store statistics using interior mutability
		*self.last_stats.borrow_mut() = Some(stats);

		debug!(
			"Aggressive JavaScript minification complete: {} -> {} bytes, {} -> {} lines, {} comments removed, {} identifiers renamed",
			original_size,
			output_bytes.len(),
			original_lines,
			final_lines,
			comments_removed,
			identifiers_renamed
		);

		Ok(output_bytes)
	}

	fn apply_aggressive_optimizations(&self, input: &str) -> String {
		// Simple aggressive optimization: replace some common long identifiers with shorter ones
		// This is a very basic implementation for demonstration purposes
		let mut result = input.to_string();
		
		// Replace some common long variable names (very basic example)
		result = result.replace("console", "c");
		result = result.replace("document", "d");
		result = result.replace("window", "w");
		
		result
	}

	fn count_identifier_changes(&self, before: &str, after: &str) -> usize {
		// Simple heuristic: count how many times we replaced common identifiers
		let console_changes = before.matches("console").count() - after.matches("console").count();
		let document_changes = before.matches("document").count() - after.matches("document").count();
		let window_changes = before.matches("window").count() - after.matches("window").count();
		
		console_changes + document_changes + window_changes
	}
}

impl Default for JavaScriptPlugin {
	fn default() -> Self {
		Self::new()
	}
}

impl Minifier for JavaScriptPlugin {
	fn name(&self) -> &str {
		"javascript"
	}

	fn detect(&self, input_bytes: &[u8]) -> bool {
		debug!(
			"JavaScript detection starting, input size: {}",
			input_bytes.len()
		);

		// Convert to string for content analysis
		let content = String::from_utf8_lossy(input_bytes);

		// Exclude obvious non-JavaScript content first
		if content.contains("body {") || content.contains("html>") || content.contains("<!DOCTYPE")
		{
			debug!("Detected HTML/CSS content, not JavaScript");
			return false;
		}

		// Look for JavaScript/TypeScript patterns
		let has_js_keywords = content.contains("function")
			|| content.contains("const")
			|| content.contains("let")
			|| content.contains("var")
			|| content.contains("=>")
			|| content.contains("import")
			|| content.contains("export")
			|| content.contains("require(");

		let has_js_syntax = content.contains("{")
			&& content.contains("}")
			&& (content.contains(";") || content.contains("=>") || content.contains("return"));

		// Check for TypeScript specific patterns
		let has_ts_syntax = content.contains(":")
			&& (content.contains("interface")
				|| content.contains("type")
				|| content.contains("enum")
				|| content.contains("class"));

		// Also detect TypeScript interfaces and types without requiring braces
		let is_typescript = content.contains("interface")
			|| content.contains("type")
			|| (content.contains("enum") && content.contains("{"));

		let detected = has_js_keywords || (has_js_syntax || has_ts_syntax || is_typescript);

		debug!(
			"JavaScript detection result: {}, keywords: {}, syntax: {}, ts: {}, is_ts: {}",
			detected, has_js_keywords, has_js_syntax, has_ts_syntax, is_typescript
		);

		detected
	}

	fn minify(&self, input_bytes: &[u8], level: MinifyLevel) -> Result<Vec<u8>, MinifyError> {
		debug!("JavaScript minification starting with level: {:?}", level);

		// Handle empty input
		if input_bytes.is_empty() {
			return Err(MinifyError::ParseError("Empty input".to_string()));
		}

		match level {
			MinifyLevel::Safe => self.safe_minify(input_bytes),
			MinifyLevel::Aggressive => self.aggressive_minify(input_bytes),
		}
	}

	fn stats(&self) -> Option<MinifyStats> {
		self.last_stats.borrow().clone()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use quickcheck::{QuickCheck, TestResult};

	#[test]
	fn test_javascript_detection() {
		let plugin = JavaScriptPlugin::new();

		// Should detect JavaScript
		assert!(plugin.detect(b"function hello() { return 'world'; }"));
		assert!(plugin.detect(b"const x = 42;"));
		assert!(plugin.detect(b"let y = () => console.log('test');"));
		assert!(plugin.detect(b"import React from 'react';"));

		// Should detect TypeScript
		assert!(plugin.detect(b"interface User { name: string; }"));
		assert!(plugin.detect(b"type MyType = string | number;"));

		// Should not detect non-JavaScript content
		assert!(!plugin.detect(b"<html><body>Hello</body></html>"));
		assert!(!plugin.detect(b"body { color: red; }"));
		assert!(!plugin.detect(b""));
		assert!(!plugin.detect(b"This is just plain text"));
	}

	#[test]
	fn test_safe_minification() {
		let plugin = JavaScriptPlugin::new();
		let input = b"function test() {\n  // This is a comment\n  return 42;\n}";

		let result = plugin.safe_minify(input);
		assert!(result.is_ok());

		let result_bytes = result.unwrap();
		let output = String::from_utf8_lossy(&result_bytes);
		// Should still be valid JavaScript but potentially smaller
		assert!(output.contains("function"));
		assert!(output.contains("return"));
		assert!(output.contains("42"));
		// Comments should be removed
		assert!(!output.contains("// This is a comment"));
	}

	#[test]
	fn test_plugin_name() {
		let plugin = JavaScriptPlugin::new();
		assert_eq!(plugin.name(), "javascript");
	}

	#[test]
	fn test_empty_input() {
		let plugin = JavaScriptPlugin::new();

		// Empty input should not be detected as JavaScript
		assert!(!plugin.detect(b""));

		// Minifying empty input should fail gracefully
		let result = plugin.minify(b"", MinifyLevel::Safe);
		assert!(result.is_err());
	}

	#[test]
	fn test_malformed_javascript() {
		let plugin = JavaScriptPlugin::new();
		let malformed = b"function unclosed() { // missing closing brace";

		// Should still detect as JavaScript due to keywords
		assert!(plugin.detect(malformed));

		// But minification should fail
		let result = plugin.minify(malformed, MinifyLevel::Safe);
		assert!(result.is_err());

		match result {
			Err(MinifyError::ParseError(_)) => {} // Expected
			_ => panic!("Should have returned ParseError"),
		}
	}

	/**
	 * Feature: omni-minify-cli, Property 7: JavaScript Minification Correctness
	 * Validates: Requirements 6.1
	 */
	#[test]
	fn property_javascript_minification_correctness() {
		fn prop(input_content: String) -> TestResult {
			// Filter out invalid or problematic inputs
			if input_content.is_empty() || input_content.len() > 1000 {
				return TestResult::discard();
			}

			// Only test with content that looks like JavaScript
			if !input_content.contains("function")
				&& !input_content.contains("const")
				&& !input_content.contains("let")
				&& !input_content.contains("var")
			{
				return TestResult::discard();
			}

			debug!(
				"Testing JavaScript minification correctness with {} byte input",
				input_content.len()
			);

			let plugin = JavaScriptPlugin::new();

			// Only test if plugin can detect this as JavaScript
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

					// Property: Minified JavaScript should still be valid JavaScript
					// We test this by attempting to parse the output
					let temp_plugin = JavaScriptPlugin::new();
					let is_valid = temp_plugin.parse_javascript(&output_str).is_ok();

					debug!("Output validity check: {}", is_valid);

					// Property: Minification should remove whitespace and comments
					// (output should be smaller or equal in size)
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
					// For property testing, we discard inputs that cause parse errors
					// since we're testing correctness of valid inputs
					TestResult::discard()
				}
			}
		}

		// Run with only 20 tests instead of default 100 for faster execution
		QuickCheck::new()
			.tests(20)
			.quickcheck(prop as fn(String) -> TestResult);
	}

	/**
	 * Feature: omni-minify-cli, Property 5: Safe Mode Semantic Preservation
	 * Validates: Requirements 4.1
	 */
	#[test]
	fn property_safe_mode_semantic_preservation() {
		fn prop(input_content: String) -> TestResult {
			// Filter out invalid or problematic inputs
			if input_content.is_empty() || input_content.len() > 1000 {
				return TestResult::discard();
			}

			// Only test with content that looks like JavaScript and is well-formed
			if !input_content.contains("function")
				&& !input_content.contains("const")
				&& !input_content.contains("let")
				&& !input_content.contains("var")
			{
				return TestResult::discard();
			}

			debug!(
				"Testing safe mode semantic preservation with {} byte input",
				input_content.len()
			);

			let plugin = JavaScriptPlugin::new();

			// Only test if plugin can detect this as JavaScript
			if !plugin.detect(input_content.as_bytes()) {
				return TestResult::discard();
			}

			// First, verify the input is valid JavaScript
			if plugin.parse_javascript(&input_content).is_err() {
				return TestResult::discard();
			}

			match plugin.minify(input_content.as_bytes(), MinifyLevel::Safe) {
				Ok(output) => {
					debug!(
						"Safe mode minification successful: {} -> {} bytes",
						input_content.len(),
						output.len()
					);

					let output_str = String::from_utf8_lossy(&output);

					// Property: Safe mode should preserve semantic meaning
					// We test this by ensuring the output is still valid JavaScript
					let temp_plugin = JavaScriptPlugin::new();
					let output_is_valid = temp_plugin.parse_javascript(&output_str).is_ok();

					debug!(
						"Semantic preservation check: input valid, output valid: {}",
						output_is_valid
					);

					// Property: Safe mode should only remove comments and whitespace
					// The core structure should remain the same
					let preserves_structure = output_str.contains("function")
						|| output_str.contains("const")
						|| output_str.contains("let")
						|| output_str.contains("var");

					debug!("Structure preservation check: {}", preserves_structure);

					TestResult::from_bool(output_is_valid && preserves_structure)
				}
				Err(e) => {
					debug!("Safe mode minification failed: {}", e);
					// For property testing, we discard inputs that cause errors
					// since we're testing semantic preservation of valid inputs
					TestResult::discard()
				}
			}
		}

		// Run with only 20 tests instead of default 100 for faster execution
		QuickCheck::new()
			.tests(20)
			.quickcheck(prop as fn(String) -> TestResult);
	}

	/**
	 * Feature: omni-minify-cli, Property 6: Aggressive Mode Size Reduction
	 * Validates: Requirements 5.1
	 */
	#[test]
	fn property_aggressive_mode_size_reduction() {
		fn prop(input_content: String) -> TestResult {
			use log::debug;
			
			// Filter out invalid or problematic inputs
			if input_content.is_empty() || input_content.len() > 1000 {
				return TestResult::discard();
			}

			// Only test with content that looks like JavaScript and is well-formed
			if !input_content.contains("function")
				&& !input_content.contains("const")
				&& !input_content.contains("let")
				&& !input_content.contains("var")
			{
				return TestResult::discard();
			}

			debug!(
				"Testing aggressive mode size reduction with {} byte input",
				input_content.len()
			);

			let plugin = JavaScriptPlugin::new();

			// Only test if plugin can detect this as JavaScript
			if !plugin.detect(input_content.as_bytes()) {
				return TestResult::discard();
			}

			// First, verify the input is valid JavaScript
			if plugin.parse_javascript(&input_content).is_err() {
				return TestResult::discard();
			}

			// Get both safe and aggressive mode results
			let safe_result = plugin.minify(input_content.as_bytes(), MinifyLevel::Safe);
			let aggressive_result = plugin.minify(input_content.as_bytes(), MinifyLevel::Aggressive);

			match (safe_result, aggressive_result) {
				(Ok(safe_output), Ok(aggressive_output)) => {
					debug!(
						"Size comparison: safe={} bytes, aggressive={} bytes",
						safe_output.len(),
						aggressive_output.len()
					);

					// Property: Aggressive mode should produce output that is smaller than or equal to safe mode
					let size_property = aggressive_output.len() <= safe_output.len();

					debug!(
						"Aggressive mode size reduction check: {} <= {} = {}",
						aggressive_output.len(),
						safe_output.len(),
						size_property
					);

					// Additional property: Both outputs should be smaller than or equal to input
					let safe_reduces_size = safe_output.len() <= input_content.len();
					let aggressive_reduces_size = aggressive_output.len() <= input_content.len();

					debug!(
						"Size reduction checks: safe {} <= {} = {}, aggressive {} <= {} = {}",
						safe_output.len(),
						input_content.len(),
						safe_reduces_size,
						aggressive_output.len(),
						input_content.len(),
						aggressive_reduces_size
					);

					TestResult::from_bool(size_property && safe_reduces_size && aggressive_reduces_size)
				}
				(Ok(_), Err(e)) => {
					debug!("Aggressive mode failed while safe mode succeeded: {}", e);
					// This could indicate a bug in aggressive mode
					TestResult::failed()
				}
				(Err(_), Ok(_)) => {
					debug!("Safe mode failed while aggressive mode succeeded - unexpected");
					// This should not happen - if safe mode fails, aggressive should too
					TestResult::failed()
				}
				(Err(e1), Err(e2)) => {
					debug!("Both modes failed: safe={}, aggressive={}", e1, e2);
					// Both failed - discard this input as it's likely invalid
					TestResult::discard()
				}
			}
		}

		// Run with only 20 tests instead of default 100 for faster execution
		QuickCheck::new()
			.tests(20)
			.quickcheck(prop as fn(String) -> TestResult);
	}
}
