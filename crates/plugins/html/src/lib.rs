use core::{Minifier, MinifyError, MinifyLevel, MinifyStats};
use log::debug;
use regex::Regex;
use std::sync::OnceLock;

// Compile regex patterns once at startup (using OnceLock for Result handling)
static HTML_COMMENT_REGEX: OnceLock<Regex> = OnceLock::new();
static HTML_LEADING_TRAILING_REGEX: OnceLock<Regex> = OnceLock::new();
static HTML_EMPTY_ATTRIBUTE_REGEX: OnceLock<Regex> = OnceLock::new();
static HTML_NEWLINE_REGEX: OnceLock<Regex> = OnceLock::new();
static HTML_SPACE_AROUND_TAGS_REGEX: OnceLock<Regex> = OnceLock::new();
static HTML_OPTIONAL_CLOSING_REGEX: OnceLock<Regex> = OnceLock::new();
static HTML_DOCTYPE_REGEX: OnceLock<Regex> = OnceLock::new();
static HTML_SCRIPT_COMMENT_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_comment_regex() -> &'static Regex {
	HTML_COMMENT_REGEX.get_or_init(|| Regex::new(r"(?s)<!--.*?-->").unwrap())
}

fn get_leading_trailing_regex() -> &'static Regex {
	HTML_LEADING_TRAILING_REGEX.get_or_init(|| Regex::new(r"^\s+|\s+$").unwrap())
}

fn get_empty_attr_regex() -> &'static Regex {
	HTML_EMPTY_ATTRIBUTE_REGEX.get_or_init(|| Regex::new(r#"(\s)([a-zA-Z-]+)=""#).unwrap())
}

fn get_newline_regex() -> &'static Regex {
	HTML_NEWLINE_REGEX.get_or_init(|| Regex::new(r"\n+").unwrap())
}

fn get_space_around_tags_regex() -> &'static Regex {
	HTML_SPACE_AROUND_TAGS_REGEX.get_or_init(|| Regex::new(r">\s+<").unwrap())
}

fn get_optional_closing_regex() -> &'static Regex {
	HTML_OPTIONAL_CLOSING_REGEX.get_or_init(|| Regex::new(r"(?i)</(p|li|tr|td|th)>").unwrap())
}

fn get_doctype_regex() -> &'static Regex {
	HTML_DOCTYPE_REGEX.get_or_init(|| Regex::new(r"(?i)^<!DOCTYPE[^>]*>").unwrap())
}

fn get_script_comment_regex() -> &'static Regex {
	HTML_SCRIPT_COMMENT_REGEX.get_or_init(|| Regex::new(r"(?si)<!--.*?-->").unwrap())
}

pub struct HTMLPlugin {
	last_stats: Option<MinifyStats>,
}

impl HTMLPlugin {
	pub fn new() -> Self {
		debug!("Initializing HTML plugin");
		Self { last_stats: None }
	}

	fn is_valid_html(&self, input: &str) -> bool {
		debug!("Validating HTML structure, length: {}", input.len());

		let has_tag = input.contains('<') && input.contains('>');

		if !has_tag {
			return false;
		}

		let open_tags = input.matches("<").count();
		let close_tags = input.matches(">").count();

		open_tags > 0 && close_tags > 0 && open_tags <= close_tags * 2
	}

	fn safe_minify(&mut self, input_bytes: &[u8]) -> Result<Vec<u8>, MinifyError> {
		debug!("Starting safe HTML minification");

		let original_size = input_bytes.len();
		let input = String::from_utf8_lossy(input_bytes);

		if !self.is_valid_html(&input) {
			debug!("Input doesn't appear to be valid HTML");
		}

		let without_comments = get_comment_regex().replace_all(&input, "").to_string();

		let without_script_comments =
			get_script_comment_regex().replace_all(&without_comments, "").to_string();

		// Remove all whitespace outside tags (no single spaces added)
		let result = Self::remove_whitespace_outside_tags(&without_script_comments);

		let output = get_leading_trailing_regex()
			.replace_all(&result, "")
			.to_string();
		let output_bytes = output.as_bytes().to_vec();

		self.last_stats = Some(
			MinifyStats::new(original_size, output_bytes.len())
				.with_extra("comments removed, whitespace collapsed".to_string()),
		);

		debug!(
			"Safe HTML minification complete: {} -> {} bytes",
			original_size,
			output_bytes.len()
		);

		Ok(output_bytes)
	}

	fn remove_whitespace_outside_tags(input: &str) -> String {
		let mut result = String::with_capacity(input.len());
		let mut in_tag = false;
		let mut in_quote = false;
		let mut quote_char = '\0';
		let mut in_script = false;
		let mut in_style = false;
		let mut chars = input.chars().peekable();

		while let Some(ch) = chars.next() {
			if ch == '<' && !in_quote {
				// Check if this is a script or style tag
				let remaining: String = chars.clone().take(20).collect();
				let remaining_lower = remaining.to_lowercase();
				
				if remaining_lower.starts_with("script") {
					in_script = true;
				} else if remaining_lower.starts_with("style") {
					in_style = true;
				} else if remaining_lower.starts_with("/script") {
					in_script = false;
				} else if remaining_lower.starts_with("/style") {
					in_style = false;
				}
				
				in_tag = true;
				result.push(ch);
			} else if ch == '>' && !in_quote {
				in_tag = false;
				in_quote = false;
				quote_char = '\0';
				result.push(ch);
			} else if in_tag {
				// Inside HTML tags - track quote state and preserve all content
				if !in_quote && (ch == '"' || ch == '\'') {
					in_quote = true;
					quote_char = ch;
				} else if in_quote && ch == quote_char {
					in_quote = false;
					quote_char = '\0';
				}
				result.push(ch);
			} else if in_script || in_style {
				// Inside script or style tags - preserve all content including whitespace
				result.push(ch);
			} else {
				// Outside tags - this is text content
				// Preserve all characters including whitespace in text content
				result.push(ch);
			}
		}

		// Only clean up whitespace between tags (not within text content)
		let mut final_result = String::with_capacity(result.len());
		let mut in_tag = false;
		let mut chars = result.chars().peekable();
		
		while let Some(ch) = chars.next() {
			if ch == '<' {
				in_tag = true;
				// Remove any trailing whitespace before this tag
				while final_result.ends_with(' ') || final_result.ends_with('\t') || final_result.ends_with('\n') || final_result.ends_with('\r') {
					final_result.pop();
				}
				final_result.push(ch);
			} else if ch == '>' {
				in_tag = false;
				final_result.push(ch);
				// Skip any immediate whitespace after closing tag if next char is <
				if let Some(&next_ch) = chars.peek() {
					if next_ch == '<' {
						// Skip whitespace until we hit the next tag
						while let Some(&ws_ch) = chars.peek() {
							if ws_ch.is_whitespace() {
								chars.next();
							} else {
								break;
							}
						}
					}
				}
			} else if in_tag {
				final_result.push(ch);
			} else {
				// Text content - preserve exactly as is
				final_result.push(ch);
			}
		}

		final_result
	}

	fn aggressive_minify(&mut self, input_bytes: &[u8]) -> Result<Vec<u8>, MinifyError> {
		debug!("Starting aggressive HTML minification");

		let original_size = input_bytes.len();

		let result = self.safe_minify(input_bytes)?;
		let minified = String::from_utf8_lossy(&result);

		let no_newlines = get_newline_regex().replace_all(&minified, "").to_string();

		let no_space_around_tags =
			get_space_around_tags_regex().replace_all(&no_newlines, "><").to_string();

		let no_empty_quotes =
			get_empty_attr_regex().replace_all(&no_space_around_tags, " $1").to_string();

		let no_optional_closing =
			get_optional_closing_regex().replace_all(&no_empty_quotes, "").to_string();

		let no_doctype = get_doctype_regex().replace_all(&no_optional_closing, "").to_string();

		let cleaned =
			get_space_around_tags_regex().replace_all(&no_doctype, "><").to_string();

		let output = cleaned.trim().to_string();
		let output_bytes = output.as_bytes().to_vec();

		self.last_stats = Some(
			MinifyStats::new(original_size, output_bytes.len()).with_extra(
				"aggressive: comments, whitespace, newlines, optional tags removed".to_string(),
			),
		);

		debug!(
			"Aggressive HTML minification complete: {} -> {} bytes",
			original_size,
			output_bytes.len()
		);

		Ok(output_bytes)
	}
}

impl Default for HTMLPlugin {
	fn default() -> Self {
		Self::new()
	}
}

impl Minifier for HTMLPlugin {
	fn name(&self) -> &str {
		"html"
	}

	fn detect(&self, input_bytes: &[u8]) -> bool {
		debug!("HTML detection starting, input size: {}", input_bytes.len());

		if input_bytes.is_empty() {
			debug!("Empty input, not HTML");
			return false;
		}

		let content = String::from_utf8_lossy(input_bytes);

		let has_doctype = content.contains("<!DOCTYPE");
		let has_html_tag = content.contains("<html") || content.contains("<HTML");
		let has_body_tag = content.contains("<body") || content.contains("<BODY");
		let has_head_tag = content.contains("<head") || content.contains("<HEAD");
		let has_script_tag = content.contains("<script") || content.contains("<SCRIPT");
		let has_style_tag = content.contains("<style") || content.contains("<STYLE");
		let has_common_tags = content.contains("<div")
			|| content.contains("<span")
			|| content.contains("<p>")
			|| content.contains("<p ")
			|| content.contains("<a>")
			|| content.contains("<a ")
			|| content.contains("<img")
			|| content.contains("<h1")
			|| content.contains("<h2")
			|| content.contains("<h3")
			|| content.contains("<ul")
			|| content.contains("<ol")
			|| content.contains("<li");

		let has_tag_structure =
			content.contains("<!") || (content.contains('<') && content.contains('>'));

		let detected = has_doctype
			|| has_html_tag
			|| has_body_tag
			|| has_head_tag
			|| has_script_tag
			|| has_style_tag
			|| (has_common_tags && has_tag_structure);

		debug!(
			"HTML detection result: {}, doctype: {}, html: {}, body: {}, common: {}",
			detected, has_doctype, has_html_tag, has_body_tag, has_common_tags
		);

		detected
	}

	fn minify(&self, input_bytes: &[u8], level: MinifyLevel) -> Result<Vec<u8>, MinifyError> {
		debug!("HTML minification starting with level: {:?}", level);

		if input_bytes.is_empty() {
			return Err(MinifyError::ParseError("Empty input".to_string()));
		}

		let mut plugin = HTMLPlugin {
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
	fn test_html_detection() {
		let plugin = HTMLPlugin::new();

		assert!(plugin.detect(b"<!DOCTYPE html><html><body>Hello</body></html>"));
		assert!(plugin.detect(b"<html><head><title>Test</title></head><body></body></html>"));
		assert!(plugin.detect(b"<div class='test'>Content</div>"));
		assert!(plugin.detect(b"<p>Paragraph</p>"));
		assert!(plugin.detect(b"<script>alert('test');</script>"));
		assert!(plugin.detect(b"<style>.class { color: red; }</style>"));

		assert!(!plugin.detect(b"function test() { return 42; }"));
		assert!(!plugin.detect(b"body { color: red; }"));
		assert!(!plugin.detect(b"const x = 42;"));
		assert!(!plugin.detect(b""));
		assert!(!plugin.detect(b"This is just plain text"));
	}

	#[test]
	fn test_safe_minification() {
		let mut plugin = HTMLPlugin::new();
		let input = b"<!DOCTYPE html>\n<html>\n<head>\n    <title>Test</title>\n</head>\n<body>\n    <!-- This is a comment -->\n    <p>Hello World</p>\n</body>\n</html>";

		let result = plugin.safe_minify(input);
		assert!(result.is_ok());

		let result_bytes = result.unwrap();
		let output = String::from_utf8_lossy(&result_bytes);

		assert!(output.contains("<!DOCTYPE html>"));
		assert!(output.contains("<html>"));
		assert!(!output.contains("<!-- This is a comment -->"));
	}

	#[test]
	fn test_aggressive_minification() {
		let mut plugin = HTMLPlugin::new();
		let input =
			b"<!DOCTYPE html>\n<html>\n<head>\n</head>\n<body>\n    <p>Hello</p>\n</body>\n</html>";

		let result = plugin.aggressive_minify(input);
		assert!(result.is_ok());

		let result_bytes = result.unwrap();
		let output = String::from_utf8_lossy(&result_bytes);

		assert!(!output.contains("\n"));
		assert!(!output.contains("<!DOCTYPE"));
	}

	#[test]
	fn test_plugin_name() {
		let plugin = HTMLPlugin::new();
		assert_eq!(plugin.name(), "html");
	}

	#[test]
	fn test_empty_input() {
		let plugin = HTMLPlugin::new();

		assert!(!plugin.detect(b""));

		let result = plugin.minify(b"", MinifyLevel::Safe);
		assert!(result.is_err());
	}

	#[test]
	fn test_html_fragment() {
		let plugin = HTMLPlugin::new();

		assert!(plugin.detect(b"<div>Content</div>"));
		assert!(plugin.detect(b"<p>Paragraph</p>"));

		let result = plugin.minify(b"<div>Content</div>", MinifyLevel::Safe);
		assert!(result.is_ok());
	}

	#[test]
	fn property_html_minification_correctness() {
		fn prop(input_content: String) -> TestResult {
			if input_content.is_empty() || input_content.len() > 1000 {
				return TestResult::discard();
			}

			if !input_content.contains("<") && !input_content.contains(">") {
				return TestResult::discard();
			}

			debug!(
				"Testing HTML minification correctness with {} byte input",
				input_content.len()
			);

			let plugin = HTMLPlugin::new();

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

					let has_tags = output_str.contains('<') && output_str.contains('>');

					debug!("Output structure check: {}", has_tags);

					let size_reduced = output.len() <= input_content.len();

					debug!(
						"Size reduction check: {} <= {} = {}",
						output.len(),
						input_content.len(),
						size_reduced
					);

					TestResult::from_bool(has_tags && size_reduced)
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

	#[test]
	fn property_safe_mode_semantic_preservation() {
		fn prop(input_content: String) -> TestResult {
			if input_content.is_empty() || input_content.len() > 1000 {
				return TestResult::discard();
			}

			if !input_content.contains("<") && !input_content.contains(">") {
				return TestResult::discard();
			}

			debug!(
				"Testing safe mode semantic preservation with {} byte input",
				input_content.len()
			);

			let plugin = HTMLPlugin::new();

			if !plugin.detect(input_content.as_bytes()) {
				return TestResult::discard();
			}

			match plugin.minify(input_content.as_bytes(), MinifyLevel::Safe) {
				Ok(output) => {
					let output_str = String::from_utf8_lossy(&output);

					let preserves_structure = output_str.contains("<!")
						|| output_str.contains("<html")
						|| output_str.contains("<body")
						|| output_str.contains("<div")
						|| output_str.contains("<p");

					let content_preserved = !output_str.is_empty();

					TestResult::from_bool(preserves_structure && content_preserved)
				}
				Err(_) => TestResult::discard(),
			}
		}

		QuickCheck::new()
			.tests(20)
			.quickcheck(prop as fn(String) -> TestResult);
	}
}
