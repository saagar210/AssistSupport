//! YAML source file parser for AssistSupport
//! Parses source definitions for batch content ingestion

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Error type for source parsing
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Type of content source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Url,
    YouTube,
    GitHub,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::Url => write!(f, "url"),
            SourceType::YouTube => write!(f, "youtube"),
            SourceType::GitHub => write!(f, "github"),
        }
    }
}

/// A single source definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDefinition {
    /// Source name for display
    pub name: String,
    /// Type of source
    #[serde(rename = "type")]
    pub source_type: SourceType,
    /// URI or path to the source
    pub uri: String,
    /// Crawl depth for URLs (default: 0 for single page)
    #[serde(default)]
    pub depth: u32,
    /// Maximum pages to crawl (default: 50)
    #[serde(default = "default_max_pages")]
    pub max_pages: usize,
    /// Maximum total bytes to ingest (default: 20MB)
    #[serde(default = "default_max_total_bytes")]
    pub max_total_bytes: usize,
    /// Allow private/loopback IPs (default: false)
    #[serde(default)]
    pub allow_private: bool,
    /// Allowed hosts for private access
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    /// Whether this source is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_max_pages() -> usize {
    50
}

fn default_max_total_bytes() -> usize {
    20 * 1024 * 1024 // 20MB
}

fn default_enabled() -> bool {
    true
}

/// A source file containing multiple source definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    /// Namespace for all sources in this file
    pub namespace: String,
    /// List of source definitions
    pub sources: Vec<SourceDefinition>,
}

impl SourceFile {
    /// Parse a YAML source file from a path
    pub fn from_path(path: &Path) -> Result<Self, ParseError> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(content.as_str())
    }

    /// Parse a YAML source file from a string
    pub fn parse(yaml: &str) -> Result<Self, ParseError> {
        let source_file: SourceFile = serde_yaml::from_str(yaml)?;
        source_file.validate()?;
        Ok(source_file)
    }

    /// Validate the source file
    fn validate(&self) -> Result<(), ParseError> {
        // Validate namespace
        if self.namespace.is_empty() {
            return Err(ParseError::Validation("Namespace cannot be empty".into()));
        }
        if !self
            .namespace
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ParseError::Validation(
                "Namespace must contain only alphanumeric characters, hyphens, and underscores"
                    .into(),
            ));
        }

        // Validate sources
        if self.sources.is_empty() {
            return Err(ParseError::Validation(
                "At least one source must be defined".into(),
            ));
        }

        for source in &self.sources {
            source.validate()?;
        }

        Ok(())
    }

    /// Get enabled sources only
    pub fn enabled_sources(&self) -> impl Iterator<Item = &SourceDefinition> {
        self.sources.iter().filter(|s| s.enabled)
    }
}

impl SourceDefinition {
    /// Validate a single source definition
    fn validate(&self) -> Result<(), ParseError> {
        if self.name.is_empty() {
            return Err(ParseError::Validation("Source name cannot be empty".into()));
        }
        if self.uri.is_empty() {
            return Err(ParseError::Validation(format!(
                "Source '{}' has empty URI",
                self.name
            )));
        }

        // Validate URI based on type
        match self.source_type {
            SourceType::Url => {
                if !self.uri.starts_with("http://") && !self.uri.starts_with("https://") {
                    return Err(ParseError::Validation(format!(
                        "URL source '{}' must start with http:// or https://",
                        self.name
                    )));
                }
            }
            SourceType::YouTube => {
                if !self.uri.contains("youtube.com") && !self.uri.contains("youtu.be") {
                    return Err(ParseError::Validation(format!(
                        "YouTube source '{}' must be a valid YouTube URL",
                        self.name
                    )));
                }
            }
            SourceType::GitHub => {
                // Can be a URL or local path
                if !self.uri.starts_with("https://github.com") && !Path::new(&self.uri).exists() {
                    // It might be a repo path like "owner/repo"
                    if !self.uri.contains('/') {
                        return Err(ParseError::Validation(
                            format!("GitHub source '{}' must be a GitHub URL, repo path (owner/repo), or local path", self.name)
                        ));
                    }
                }
            }
        }

        // Validate limits
        if self.max_pages == 0 {
            return Err(ParseError::Validation(format!(
                "Source '{}' max_pages must be greater than 0",
                self.name
            )));
        }
        if self.max_total_bytes == 0 {
            return Err(ParseError::Validation(format!(
                "Source '{}' max_total_bytes must be greater than 0",
                self.name
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_source_file() {
        let yaml = r#"
namespace: it-support
sources:
  - name: microsoft-365
    type: url
    uri: https://learn.microsoft.com/en-us/microsoft-365/
    depth: 2
    max_pages: 50
    enabled: true
  - name: youtube-tutorial
    type: youtube
    uri: https://www.youtube.com/watch?v=dQw4w9WgXcQ
    enabled: true
  - name: local-repo
    type: github
    uri: owner/repo
    enabled: false
"#;

        let source_file = SourceFile::parse(yaml).unwrap();
        assert_eq!(source_file.namespace, "it-support");
        assert_eq!(source_file.sources.len(), 3);
        assert_eq!(source_file.enabled_sources().count(), 2);
    }

    #[test]
    fn test_parse_minimal_source_file() {
        let yaml = r#"
namespace: default
sources:
  - name: example
    type: url
    uri: https://example.com
"#;

        let source_file = SourceFile::parse(yaml).unwrap();
        assert_eq!(source_file.namespace, "default");
        assert_eq!(source_file.sources.len(), 1);
        assert!(source_file.sources[0].enabled); // default true
        assert_eq!(source_file.sources[0].max_pages, 50); // default
    }

    #[test]
    fn test_parse_empty_namespace() {
        let yaml = r#"
namespace: ""
sources:
  - name: example
    type: url
    uri: https://example.com
"#;

        let result = SourceFile::parse(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_url() {
        let yaml = r#"
namespace: test
sources:
  - name: bad-url
    type: url
    uri: not-a-url
"#;

        let result = SourceFile::parse(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_source_type_display() {
        assert_eq!(format!("{}", SourceType::Url), "url");
        assert_eq!(format!("{}", SourceType::YouTube), "youtube");
        assert_eq!(format!("{}", SourceType::GitHub), "github");
    }
}
