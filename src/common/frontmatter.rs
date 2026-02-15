use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FrontmatterError {
    #[error("Invalid frontmatter format: {0}")]
    InvalidFormat(String),
    #[error("YAML parse error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

/// Parse markdown content with YAML frontmatter.
///
/// Returns a tuple of:
/// - Deserialized metadata from frontmatter
/// - Title extracted from the H1 heading after frontmatter
/// - Body content (everything after the title)
///
/// # Format
/// ```markdown
/// ---
/// key: value
/// ---
/// # Title
///
/// Body content...
/// ```
pub fn parse_frontmatter<T: DeserializeOwned>(
    content: &str,
) -> Result<(T, String, String), FrontmatterError> {
    let lines: Vec<&str> = content.lines().collect();

    // Check for frontmatter opening
    if lines.first() != Some(&"---") {
        return Err(FrontmatterError::InvalidFormat(
            "Content must start with '---'".to_string(),
        ));
    }

    // Find closing ---
    let end_idx = lines
        .iter()
        .skip(1)
        .position(|&line| line == "---")
        .ok_or_else(|| {
            FrontmatterError::InvalidFormat("Missing closing '---' for frontmatter".to_string())
        })?;

    // Extract and parse frontmatter YAML
    let frontmatter_yaml = lines.get(1..=end_idx).unwrap_or(&[]).join("\n");
    let metadata: T = serde_yaml::from_str(&frontmatter_yaml)?;

    // Extract content after frontmatter
    let body_start = end_idx.saturating_add(2); // Skip the closing ---
    let body_lines: Vec<&str> = lines
        .get(body_start..)
        .unwrap_or(&[])
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect();

    // Extract title from H1 heading
    let (title, body) = if body_lines.first().is_some_and(|l| l.starts_with("# ")) {
        let first_line = body_lines.first().unwrap_or(&"");
        let title = first_line.strip_prefix("# ").unwrap_or("").to_string();
        let body = body_lines
            .get(1..)
            .unwrap_or(&[])
            .iter()
            .skip_while(|line| line.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_string();
        (title, body)
    } else {
        // No H1 heading, use empty title and full body
        (String::new(), body_lines.join("\n").trim_end().to_string())
    };

    Ok((metadata, title, body))
}

/// Parse markdown content with YAML frontmatter into a raw `serde_yaml::Value`.
///
/// Same splitting logic as `parse_frontmatter<T>` but deserializes YAML into
/// `serde_yaml::Value` so that unknown/type-specific fields are preserved.
pub fn parse_frontmatter_raw(
    content: &str,
) -> Result<(serde_yaml::Value, String, String), FrontmatterError> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.first() != Some(&"---") {
        return Err(FrontmatterError::InvalidFormat(
            "Content must start with '---'".to_string(),
        ));
    }

    let end_idx = lines
        .iter()
        .skip(1)
        .position(|&line| line == "---")
        .ok_or_else(|| {
            FrontmatterError::InvalidFormat("Missing closing '---' for frontmatter".to_string())
        })?;

    let frontmatter_yaml = lines.get(1..=end_idx).unwrap_or(&[]).join("\n");
    let value: serde_yaml::Value = serde_yaml::from_str(&frontmatter_yaml)?;

    let body_start = end_idx.saturating_add(2);
    let body_lines: Vec<&str> = lines
        .get(body_start..)
        .unwrap_or(&[])
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect();

    let (title, body) = if body_lines.first().is_some_and(|l| l.starts_with("# ")) {
        let first_line = body_lines.first().unwrap_or(&"");
        let title = first_line.strip_prefix("# ").unwrap_or("").to_string();
        let body = body_lines
            .get(1..)
            .unwrap_or(&[])
            .iter()
            .skip_while(|line| line.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_string();
        (title, body)
    } else {
        (String::new(), body_lines.join("\n").trim_end().to_string())
    };

    Ok((value, title, body))
}

/// Generate markdown content from a raw `serde_yaml::Value`.
pub fn generate_frontmatter_raw(value: &serde_yaml::Value, title: &str, body: &str) -> String {
    let yaml = serde_yaml::to_string(value).unwrap_or_default();
    let yaml = yaml.trim_end();

    if body.is_empty() {
        format!("---\n{yaml}\n---\n\n# {title}\n")
    } else {
        format!("---\n{yaml}\n---\n\n# {title}\n\n{body}\n")
    }
}

/// Generate markdown content with YAML frontmatter.
///
/// # Arguments
/// - `metadata`: Struct to serialize as YAML frontmatter
/// - `title`: The H1 heading title
/// - `body`: The body content after the title
///
/// # Returns
/// Formatted markdown string with frontmatter
pub fn generate_frontmatter<T: Serialize>(metadata: &T, title: &str, body: &str) -> String {
    // Serialize metadata to YAML
    let yaml = serde_yaml::to_string(metadata).unwrap_or_default();
    // serde_yaml adds a trailing newline, so trim it
    let yaml = yaml.trim_end();

    if body.is_empty() {
        format!("---\n{yaml}\n---\n\n# {title}\n")
    } else {
        format!("---\n{yaml}\n---\n\n# {title}\n\n{body}\n")
    }
}

#[cfg(test)]
mod tests {
    /// Escape a string for use in YAML values.
    /// Handles backslashes and double quotes.
    fn escape_yaml_string(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    struct TestMetadata {
        display_number: u32,
        status: String,
        #[serde(default)]
        draft: bool,
    }

    #[test]
    fn test_parse_frontmatter_basic() {
        let content = r"---
displayNumber: 42
status: open
draft: false
---

# Test Title

This is the body content.";

        let (metadata, title, body): (TestMetadata, String, String) =
            parse_frontmatter(content).unwrap();

        assert_eq!(metadata.display_number, 42);
        assert_eq!(metadata.status, "open");
        assert!(!metadata.draft);
        assert_eq!(title, "Test Title");
        assert_eq!(body, "This is the body content.");
    }

    #[test]
    fn test_parse_frontmatter_empty_body() {
        let content = r"---
displayNumber: 1
status: closed
---

# Just a Title";

        let (metadata, title, body): (TestMetadata, String, String) =
            parse_frontmatter(content).unwrap();

        assert_eq!(metadata.display_number, 1);
        assert_eq!(title, "Just a Title");
        assert_eq!(body, "");
    }

    #[test]
    fn test_parse_frontmatter_multiline_body() {
        let content = r"---
displayNumber: 5
status: in-progress
---

# Multi Line

Line 1.

Line 2.

Line 3.";

        let (metadata, title, body): (TestMetadata, String, String) =
            parse_frontmatter(content).unwrap();

        assert_eq!(metadata.display_number, 5);
        assert_eq!(title, "Multi Line");
        assert_eq!(body, "Line 1.\n\nLine 2.\n\nLine 3.");
    }

    #[test]
    fn test_parse_frontmatter_missing_opening() {
        let content = "# No Frontmatter\n\nJust content.";
        let result: Result<(TestMetadata, String, String), _> = parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_frontmatter_missing_closing() {
        let content = "---\ndisplayNumber: 1\nstatus: open\n# Title";
        let result: Result<(TestMetadata, String, String), _> = parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_frontmatter_basic() {
        let metadata = TestMetadata {
            display_number: 42,
            status: "open".to_string(),
            draft: false,
        };

        let result = generate_frontmatter(&metadata, "Test Title", "Body content.");

        assert!(result.starts_with("---\n"));
        assert!(result.contains("displayNumber: 42"));
        assert!(result.contains("status: open"));
        assert!(result.contains("# Test Title"));
        assert!(result.contains("Body content."));
    }

    #[test]
    fn test_generate_frontmatter_empty_body() {
        let metadata = TestMetadata {
            display_number: 1,
            status: "closed".to_string(),
            draft: true,
        };

        let result = generate_frontmatter(&metadata, "Title Only", "");

        assert!(result.contains("# Title Only"));
        assert!(result.ends_with("# Title Only\n"));
    }

    #[test]
    fn test_roundtrip() {
        let original_metadata = TestMetadata {
            display_number: 99,
            status: "review".to_string(),
            draft: true,
        };
        let original_title = "Round Trip Test";
        let original_body = "This should survive the round trip.";

        let generated = generate_frontmatter(&original_metadata, original_title, original_body);
        let (parsed_metadata, parsed_title, parsed_body): (TestMetadata, String, String) =
            parse_frontmatter(&generated).unwrap();

        assert_eq!(parsed_metadata, original_metadata);
        assert_eq!(parsed_title, original_title);
        assert_eq!(parsed_body, original_body);
    }

    #[test]
    fn test_escape_yaml_string() {
        assert_eq!(escape_yaml_string("hello"), "hello");
        assert_eq!(escape_yaml_string(r#"say "hi""#), r#"say \"hi\""#);
        assert_eq!(escape_yaml_string(r"back\slash"), r"back\\slash");
    }
}
