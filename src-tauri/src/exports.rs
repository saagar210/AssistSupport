//! Export utilities for drafts and responses (Phase 18)
//! Provides HTML, plaintext, and clipboard-ready formats

use serde::{Deserialize, Serialize};

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExportFormat {
    /// Plain text with markdown preserved
    #[default]
    Plaintext,
    /// HTML formatted for email
    Html,
    /// Simplified HTML for ticket systems
    TicketHtml,
    /// JSON structured format
    Json,
}

/// Safe export options - strips sensitive data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SafeExportOptions {
    /// Strip usernames
    pub strip_usernames: bool,
    /// Strip internal IDs (ticket IDs, chunk IDs)
    pub strip_internal_ids: bool,
    /// Strip file paths
    pub strip_file_paths: bool,
    /// Strip email addresses
    pub strip_emails: bool,
    /// Custom patterns to strip (regex)
    pub custom_patterns: Vec<String>,
}

/// Exported draft content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedDraft {
    /// The main response text
    pub response: String,
    /// Summary if available
    pub summary: Option<String>,
    /// KB sources used
    pub sources: Vec<ExportedSource>,
    /// Case intake data (if present)
    pub case_intake: Option<serde_json::Value>,
    /// Handoff summary (if present)
    pub handoff_summary: Option<String>,
    /// Export metadata
    pub metadata: ExportMetadata,
}

/// KB source in export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedSource {
    pub title: String,
    pub path: Option<String>,
    pub url: Option<String>,
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub exported_at: String,
    pub format: String,
    pub app_version: String,
    pub model_name: Option<String>,
}

/// Format a draft for export
pub fn format_draft(
    response_text: &str,
    summary: Option<&str>,
    sources: &[ExportedSource],
    format: ExportFormat,
    safe_options: Option<&SafeExportOptions>,
) -> String {
    let mut response = response_text.to_string();
    let mut summary_text = summary.map(|s| s.to_string());

    // Apply safe export transformations if requested
    if let Some(opts) = safe_options {
        response = apply_safe_export(&response, opts);
        summary_text = summary_text.map(|s| apply_safe_export(&s, opts));
    }

    match format {
        ExportFormat::Plaintext => {
            format_plaintext(&response, summary_text.as_deref(), sources)
        }
        ExportFormat::Html => {
            format_html(&response, summary_text.as_deref(), sources, false)
        }
        ExportFormat::TicketHtml => {
            format_html(&response, summary_text.as_deref(), sources, true)
        }
        ExportFormat::Json => {
            let export = serde_json::json!({
                "response": response,
                "summary": summary_text,
                "sources": sources,
            });
            serde_json::to_string_pretty(&export).unwrap_or_default()
        }
    }
}

/// Format as plaintext
fn format_plaintext(
    response: &str,
    summary: Option<&str>,
    sources: &[ExportedSource],
) -> String {
    let mut output = String::new();

    if let Some(sum) = summary {
        output.push_str("Summary:\n");
        output.push_str(sum);
        output.push_str("\n\n---\n\n");
    }

    output.push_str(response);

    if !sources.is_empty() {
        output.push_str("\n\n---\nSources:\n");
        for (i, source) in sources.iter().enumerate() {
            output.push_str(&format!("[{}] {}", i + 1, source.title));
            if let Some(url) = &source.url {
                output.push_str(&format!(" - {}", url));
            }
            output.push('\n');
        }
    }

    output
}

/// Format as HTML
fn format_html(
    response: &str,
    summary: Option<&str>,
    sources: &[ExportedSource],
    simplified: bool,
) -> String {
    let mut html = String::new();

    if !simplified {
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; ");
        html.push_str("line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 20px; }\n");
        html.push_str("h1, h2, h3 { color: #333; }\n");
        html.push_str(".summary { background: #f5f5f5; padding: 15px; border-radius: 8px; margin-bottom: 20px; }\n");
        html.push_str(".sources { margin-top: 20px; padding-top: 20px; border-top: 1px solid #ddd; }\n");
        html.push_str(".sources ul { padding-left: 20px; }\n");
        html.push_str("</style>\n</head>\n<body>\n");
    }

    if let Some(sum) = summary {
        if simplified {
            html.push_str("<p><strong>Summary:</strong> ");
            html.push_str(&escape_html(sum));
            html.push_str("</p>\n<hr>\n");
        } else {
            html.push_str("<div class=\"summary\">\n<h3>Summary</h3>\n<p>");
            html.push_str(&escape_html(sum));
            html.push_str("</p>\n</div>\n");
        }
    }

    // Convert markdown-ish content to HTML paragraphs
    html.push_str("<div class=\"response\">\n");
    for para in response.split("\n\n") {
        let para = para.trim();
        if !para.is_empty() {
            html.push_str("<p>");
            html.push_str(&escape_html(para).replace('\n', "<br>"));
            html.push_str("</p>\n");
        }
    }
    html.push_str("</div>\n");

    if !sources.is_empty() {
        if simplified {
            html.push_str("<hr>\n<p><strong>Sources:</strong></p>\n<ul>\n");
        } else {
            html.push_str("<div class=\"sources\">\n<h4>Sources</h4>\n<ul>\n");
        }
        for (i, source) in sources.iter().enumerate() {
            html.push_str("<li>");
            if let Some(url) = &source.url {
                html.push_str(&format!(
                    "[{}] <a href=\"{}\">{}</a>",
                    i + 1,
                    escape_html(url),
                    escape_html(&source.title)
                ));
            } else {
                html.push_str(&format!("[{}] {}", i + 1, escape_html(&source.title)));
            }
            html.push_str("</li>\n");
        }
        if simplified {
            html.push_str("</ul>\n");
        } else {
            html.push_str("</ul>\n</div>\n");
        }
    }

    if !simplified {
        html.push_str("</body>\n</html>");
    }

    html
}

/// Apply safe export transformations
fn apply_safe_export(text: &str, opts: &SafeExportOptions) -> String {
    let mut result = text.to_string();

    if opts.strip_emails {
        // Simple email pattern replacement
        let email_re = regex_lite::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
            .unwrap();
        result = email_re.replace_all(&result, "[email]").to_string();
    }

    if opts.strip_usernames {
        // Replace common username patterns
        let user_re = regex_lite::Regex::new(r"@[A-Za-z0-9_]+\b").unwrap();
        result = user_re.replace_all(&result, "@[user]").to_string();
    }

    if opts.strip_file_paths {
        // Replace file paths
        let path_re = regex_lite::Regex::new(r"(/[A-Za-z0-9._-]+)+(/[A-Za-z0-9._-]+)").unwrap();
        result = path_re.replace_all(&result, "[path]").to_string();
        // Windows paths
        let win_path_re = regex_lite::Regex::new(r"[A-Za-z]:\\([A-Za-z0-9._-]+\\)+[A-Za-z0-9._-]+").unwrap();
        result = win_path_re.replace_all(&result, "[path]").to_string();
    }

    if opts.strip_internal_ids {
        // Replace UUIDs
        let uuid_re = regex_lite::Regex::new(
            r"\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b"
        ).unwrap();
        result = uuid_re.replace_all(&result, "[id]").to_string();
    }

    // Apply custom patterns
    for pattern in &opts.custom_patterns {
        if let Ok(re) = regex_lite::Regex::new(pattern) {
            result = re.replace_all(&result, "[redacted]").to_string();
        }
    }

    result
}

/// Escape HTML special characters
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Format content for clipboard (optimized for ticket systems)
pub fn format_for_clipboard(
    response_text: &str,
    sources: &[ExportedSource],
    include_sources: bool,
) -> String {
    let mut output = response_text.to_string();

    if include_sources && !sources.is_empty() {
        output.push_str("\n\n---\nReferences:\n");
        for (i, source) in sources.iter().enumerate() {
            output.push_str(&format!("• [{}] {}\n", i + 1, source.title));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_plaintext() {
        let sources = vec![
            ExportedSource {
                title: "VPN Guide".to_string(),
                path: Some("/docs/vpn.md".to_string()),
                url: None,
            },
        ];

        let output = format_plaintext("Test response", Some("Test summary"), &sources);

        assert!(output.contains("Summary:"));
        assert!(output.contains("Test summary"));
        assert!(output.contains("Test response"));
        assert!(output.contains("[1] VPN Guide"));
    }

    #[test]
    fn test_format_html() {
        let sources = vec![];
        let output = format_html("Test <script>alert('xss')</script>", None, &sources, false);

        // Should escape HTML
        assert!(output.contains("&lt;script&gt;"));
        assert!(!output.contains("<script>"));
    }

    #[test]
    fn test_safe_export_strips_emails() {
        let opts = SafeExportOptions {
            strip_emails: true,
            ..Default::default()
        };

        let result = apply_safe_export("Contact john@example.com for help", &opts);
        assert!(result.contains("[email]"));
        assert!(!result.contains("john@example.com"));
    }

    #[test]
    fn test_safe_export_strips_paths() {
        let opts = SafeExportOptions {
            strip_file_paths: true,
            ..Default::default()
        };

        let result = apply_safe_export("File at /Users/john/Documents/secret.txt", &opts);
        assert!(result.contains("[path]"));
        assert!(!result.contains("/Users/john"));
    }

    #[test]
    fn test_safe_export_strips_uuids() {
        let opts = SafeExportOptions {
            strip_internal_ids: true,
            ..Default::default()
        };

        let result = apply_safe_export("Chunk ID: 550e8400-e29b-41d4-a716-446655440000", &opts);
        assert!(result.contains("[id]"));
        assert!(!result.contains("550e8400"));
    }

    #[test]
    fn test_format_for_clipboard() {
        let sources = vec![
            ExportedSource {
                title: "Doc 1".to_string(),
                path: None,
                url: Some("https://example.com".to_string()),
            },
        ];

        let output = format_for_clipboard("Response text", &sources, true);
        assert!(output.contains("Response text"));
        assert!(output.contains("• [1] Doc 1"));
    }
}
