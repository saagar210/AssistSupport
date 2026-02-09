use std::path::Path;

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::error::AppError;

use super::{count_words, DocumentMetadata, ParsedDocument, Section};

pub fn parse(path: &Path) -> Result<ParsedDocument, AppError> {
    let source = std::fs::read_to_string(path)?;
    let cleaned = strip_confluence_boilerplate(&source);
    let parser = Parser::new(&cleaned);

    let mut full_text = String::new();
    let mut sections: Vec<Section> = Vec::new();
    let mut current_heading: Option<(String, i32)> = None;
    let mut current_content = String::new();
    let mut in_heading = false;
    let mut heading_text = String::new();
    let mut heading_level: i32 = 1;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                // Flush previous section
                if let Some((title, lvl)) = current_heading.take() {
                    sections.push(Section {
                        title,
                        content: current_content.trim().to_string(),
                        level: lvl,
                    });
                    current_content.clear();
                }
                in_heading = true;
                heading_text.clear();
                heading_level = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                current_heading = Some((heading_text.clone(), heading_level));
                full_text.push_str(&heading_text);
                full_text.push('\n');
            }
            Event::Text(text) => {
                if in_heading {
                    heading_text.push_str(&text);
                } else {
                    current_content.push_str(&text);
                    full_text.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if !in_heading {
                    current_content.push('\n');
                    full_text.push('\n');
                }
            }
            Event::End(TagEnd::Paragraph) => {
                current_content.push_str("\n\n");
                full_text.push_str("\n\n");
            }
            Event::Code(code) => {
                if in_heading {
                    heading_text.push_str(&code);
                } else {
                    current_content.push_str(&code);
                    full_text.push_str(&code);
                }
            }
            _ => {}
        }
    }

    // Flush last section
    if let Some((title, lvl)) = current_heading.take() {
        sections.push(Section {
            title,
            content: current_content.trim().to_string(),
            level: lvl,
        });
    }

    let full_text = full_text.trim().to_string();
    let word_count = count_words(&full_text);

    Ok(ParsedDocument {
        text: full_text,
        metadata: DocumentMetadata {
            title: sections.first().map(|s| s.title.clone()),
            author: None,
            page_count: None,
            word_count,
        },
        sections,
    })
}

/// Strip Confluence-export boilerplate from markdown source before parsing.
/// Handles: YAML frontmatter, breadcrumb lines, metadata lines (Created/Modified by),
/// Confluence macros ({toc}, {children}, {excerpt}, etc.), and "Powered by" footers.
fn strip_confluence_boilerplate(source: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    let mut in_frontmatter = false;
    let mut frontmatter_started = false;

    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // YAML frontmatter: strip --- delimited blocks at start of file
        if i == 0 && trimmed == "---" {
            in_frontmatter = true;
            frontmatter_started = true;
            continue;
        }
        if in_frontmatter {
            if trimmed == "---" {
                in_frontmatter = false;
            }
            continue;
        }

        // Skip breadcrumb lines: "Space > Parent > Page" or "Home / Docs / Page"
        if is_breadcrumb_line(trimmed) {
            continue;
        }

        // Skip Confluence metadata lines
        if is_confluence_metadata_line(trimmed) {
            continue;
        }

        // Skip Confluence macro markers
        if is_confluence_macro(trimmed) {
            continue;
        }

        // Skip "Powered by Confluence" / "Powered by Atlassian" footers
        let lower = trimmed.to_lowercase();
        if lower.starts_with("powered by confluence")
            || lower.starts_with("powered by atlassian")
        {
            continue;
        }

        lines.push(line);
    }

    // If frontmatter opened but never closed, return original (not a real frontmatter block)
    if frontmatter_started && in_frontmatter {
        return source.to_string();
    }

    lines.join("\n")
}

fn is_breadcrumb_line(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    // Breadcrumbs use ">" or "/" separators with 2+ segments
    // e.g., "Engineering > Docs > API Guide" or "Home / Knowledge Base / Setup"
    let has_arrow_sep = line.contains(" > ") && line.split(" > ").count() >= 3;
    let has_slash_sep = line.contains(" / ") && line.split(" / ").count() >= 3;

    if !has_arrow_sep && !has_slash_sep {
        return false;
    }

    // Breadcrumbs are short (no segment > ~60 chars) and don't look like prose
    let segments: Vec<&str> = if has_arrow_sep {
        line.split(" > ").collect()
    } else {
        line.split(" / ").collect()
    };

    // Each segment should be short (a page title, not a sentence)
    segments.iter().all(|s| s.trim().len() <= 60 && !s.contains('.'))
}

fn is_confluence_metadata_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    // "Created by John on Jan 1, 2024" / "Last modified by..." / "Labels: foo, bar"
    lower.starts_with("created by ")
        || lower.starts_with("last modified by ")
        || lower.starts_with("last updated by ")
        || lower.starts_with("modified by ")
        || (lower.starts_with("labels:") && !lower.contains('\n'))
}

fn is_confluence_macro(line: &str) -> bool {
    let trimmed = line.trim();
    // Confluence macros: {toc}, {children}, {excerpt}, {include}, {info}, {note}, {warning}, {tip}, {panel}
    // Also matches {toc:param=value} style
    if trimmed.starts_with('{') && trimmed.contains('}') {
        let inner = &trimmed[1..];
        if let Some(end) = inner.find('}') {
            let macro_content = &inner[..end];
            let macro_name = macro_content.split(':').next().unwrap_or("");
            return matches!(
                macro_name,
                "toc" | "children" | "excerpt" | "include" | "info" | "note"
                    | "warning" | "tip" | "panel" | "expand" | "status"
                    | "recently-updated" | "page-tree"
            );
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_confluence_boilerplate_frontmatter() {
        let source = "---\ntitle: My Page\nspace: ENG\n---\n# Real Content\n\nThis is the body.";
        let result = strip_confluence_boilerplate(source);
        assert!(result.starts_with("# Real Content"), "Frontmatter should be stripped, got: {}", result);
        assert!(!result.contains("title: My Page"));
    }

    #[test]
    fn test_strip_confluence_boilerplate_breadcrumbs() {
        let source = "Engineering > Docs > API Guide\n\n# API Guide\n\nContent here.";
        let result = strip_confluence_boilerplate(source);
        assert!(!result.contains("Engineering > Docs"), "Breadcrumbs should be stripped");
        assert!(result.contains("# API Guide"));
    }

    #[test]
    fn test_strip_confluence_boilerplate_metadata_lines() {
        let source = "# Page Title\n\nCreated by John Smith on Jan 1, 2024\nLast modified by Jane Doe on Feb 15, 2024\nLabels: api, internal\n\nActual content here.";
        let result = strip_confluence_boilerplate(source);
        assert!(!result.contains("Created by"), "Created by should be stripped");
        assert!(!result.contains("Last modified by"), "Last modified should be stripped");
        assert!(!result.contains("Labels:"), "Labels should be stripped");
        assert!(result.contains("Actual content here."));
    }

    #[test]
    fn test_strip_confluence_boilerplate_macros() {
        let source = "# Setup Guide\n\n{toc}\n\n{children}\n\n{excerpt}Summary text{excerpt}\n\nReal paragraph content here.";
        let result = strip_confluence_boilerplate(source);
        assert!(!result.contains("{toc}"), "{{toc}} should be stripped");
        assert!(!result.contains("{children}"), "{{children}} should be stripped");
        assert!(result.contains("Real paragraph content here."));
    }

    #[test]
    fn test_strip_confluence_boilerplate_powered_by() {
        let source = "# Page\n\nContent.\n\nPowered by Confluence\nPowered by Atlassian Confluence 7.19";
        let result = strip_confluence_boilerplate(source);
        assert!(!result.to_lowercase().contains("powered by confluence"));
        assert!(result.contains("Content."));
    }

    #[test]
    fn test_strip_confluence_boilerplate_preserves_normal_content() {
        let source = "# Normal Document\n\nThis has no Confluence boilerplate.\n\nJust regular content with > quotes.";
        let result = strip_confluence_boilerplate(source);
        assert_eq!(result, source, "Normal content should be unchanged");
    }

    #[test]
    fn test_strip_confluence_boilerplate_unclosed_frontmatter() {
        let source = "---\nThis is not frontmatter, just a horizontal rule context";
        let result = strip_confluence_boilerplate(source);
        assert_eq!(result, source, "Unclosed frontmatter should return original");
    }

    #[test]
    fn test_strip_confluence_macro_with_params() {
        let source = "{toc:maxLevel=3}\n\n# Heading\n\nContent.";
        let result = strip_confluence_boilerplate(source);
        assert!(!result.contains("{toc:maxLevel=3}"), "Parameterized macro should be stripped");
        assert!(result.contains("# Heading"));
    }

    #[test]
    fn test_breadcrumb_detection_requires_three_segments() {
        // Two segments is not a breadcrumb — could be normal prose
        assert!(!is_breadcrumb_line("foo > bar"));
        // Three segments qualifies
        assert!(is_breadcrumb_line("Home > Docs > Page"));
    }

    #[test]
    fn test_breadcrumb_not_triggered_by_prose() {
        // Long segments with periods look like prose, not breadcrumbs
        assert!(!is_breadcrumb_line("This is a sentence. > Another sentence. > Third one."));
    }
}
