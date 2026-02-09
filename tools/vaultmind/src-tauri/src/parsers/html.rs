use std::path::Path;

use scraper::{Html, Selector};

use crate::error::AppError;

use super::{count_words, DocumentMetadata, ParsedDocument, Section};

pub fn parse(path: &Path) -> Result<ParsedDocument, AppError> {
    let content = std::fs::read_to_string(path)?;
    let document = Html::parse_document(&content);

    // Collect element IDs of script/style tags to skip
    let script_sel = Selector::parse("script")
        .map_err(|e| AppError::Parse(format!("Invalid selector: {}", e)))?;
    let style_sel = Selector::parse("style")
        .map_err(|e| AppError::Parse(format!("Invalid selector: {}", e)))?;

    let mut skip_ids: std::collections::HashSet<ego_tree::NodeId> =
        std::collections::HashSet::new();
    for el in document.select(&script_sel) {
        skip_ids.insert(el.id());
    }
    for el in document.select(&style_sel) {
        skip_ids.insert(el.id());
    }

    // Skip Confluence/generic boilerplate elements: nav, footer, aside, and
    // elements with boilerplate class names
    let boilerplate_selectors = [
        "nav", "footer", "aside",
        ".breadcrumb", ".breadcrumbs",
        ".page-metadata", ".page-metadata-modification-info",
        ".navigation", ".nav-breadcrumb",
        ".confluence-information-macro",
        ".footer-body", ".page-restrictions",
        "#footer", "#breadcrumbs", "#navigation",
    ];
    for sel_str in &boilerplate_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            for el in document.select(&sel) {
                skip_ids.insert(el.id());
            }
        }
    }

    // Extract title
    let title_sel =
        Selector::parse("title").map_err(|e| AppError::Parse(format!("Invalid selector: {}", e)))?;
    let title = document
        .select(&title_sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string());

    // Extract sections from headings
    let mut sections: Vec<Section> = Vec::new();
    for (level, tag) in [(1i32, "h1"), (2, "h2"), (3, "h3")] {
        let sel = Selector::parse(tag)
            .map_err(|e| AppError::Parse(format!("Invalid selector: {}", e)))?;
        for el in document.select(&sel) {
            let heading_text = el.text().collect::<String>().trim().to_string();
            if !heading_text.is_empty() {
                sections.push(Section {
                    title: heading_text,
                    content: String::new(),
                    level,
                });
            }
        }
    }

    // Extract body text, skipping script/style
    let body_sel =
        Selector::parse("body").map_err(|e| AppError::Parse(format!("Invalid selector: {}", e)))?;
    let body = document.select(&body_sel).next();

    let full_text = if let Some(body_el) = body {
        extract_text_recursive(&body_el, &skip_ids)
    } else {
        // No body tag, extract from root
        let html_sel = Selector::parse("html").ok();
        if let Some(sel) = html_sel {
            if let Some(root) = document.select(&sel).next() {
                extract_text_recursive(&root, &skip_ids)
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    };

    let full_text = full_text.trim().to_string();
    let word_count = count_words(&full_text);

    Ok(ParsedDocument {
        text: full_text,
        metadata: DocumentMetadata {
            title,
            author: None,
            page_count: None,
            word_count,
        },
        sections,
    })
}

fn extract_text_recursive(
    element: &scraper::ElementRef,
    skip_ids: &std::collections::HashSet<ego_tree::NodeId>,
) -> String {
    let mut result = String::new();
    for child in element.children() {
        if let Some(el) = scraper::ElementRef::wrap(child) {
            if skip_ids.contains(&el.id()) {
                continue;
            }
            result.push_str(&extract_text_recursive(&el, skip_ids));
            // Add spacing after block elements
            let tag = el.value().name();
            if matches!(tag, "p" | "div" | "br" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" | "tr") {
                result.push('\n');
            }
        } else if let Some(text) = child.value().as_text() {
            result.push_str(text);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn parse_html_string(html: &str) -> Result<ParsedDocument, AppError> {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("test.html");
        let mut f = std::fs::File::create(&file_path).expect("create file");
        f.write_all(html.as_bytes()).expect("write html");
        parse(&file_path)
    }

    #[test]
    fn test_html_strips_nav_element() {
        let html = r#"<html><body>
            <nav><a href="/">Home</a> > <a href="/docs">Docs</a></nav>
            <h1>API Guide</h1>
            <p>This is the real content.</p>
        </body></html>"#;
        let doc = parse_html_string(html).unwrap();
        assert!(!doc.text.contains("Home"), "Nav content should be stripped");
        assert!(doc.text.contains("real content"));
    }

    #[test]
    fn test_html_strips_footer_element() {
        let html = r#"<html><body>
            <h1>Page</h1>
            <p>Content here.</p>
            <footer>Powered by Confluence 7.19</footer>
        </body></html>"#;
        let doc = parse_html_string(html).unwrap();
        assert!(!doc.text.contains("Powered by"), "Footer should be stripped");
        assert!(doc.text.contains("Content here."));
    }

    #[test]
    fn test_html_strips_breadcrumb_class() {
        let html = r#"<html><body>
            <div class="breadcrumb">Space > Parent > Child</div>
            <h1>Child Page</h1>
            <p>Body text.</p>
        </body></html>"#;
        let doc = parse_html_string(html).unwrap();
        assert!(!doc.text.contains("Space > Parent"), "Breadcrumb class should be stripped");
        assert!(doc.text.contains("Body text."));
    }

    #[test]
    fn test_html_strips_page_metadata_class() {
        let html = r#"<html><body>
            <h1>Setup</h1>
            <div class="page-metadata">Last modified by admin on 2024-01-15</div>
            <p>Installation steps below.</p>
        </body></html>"#;
        let doc = parse_html_string(html).unwrap();
        assert!(!doc.text.contains("Last modified"), "Page metadata should be stripped");
        assert!(doc.text.contains("Installation steps"));
    }

    #[test]
    fn test_html_strips_aside_element() {
        let html = r#"<html><body>
            <aside>Related pages: Foo, Bar</aside>
            <h1>Main</h1>
            <p>Main content.</p>
        </body></html>"#;
        let doc = parse_html_string(html).unwrap();
        assert!(!doc.text.contains("Related pages"), "Aside should be stripped");
        assert!(doc.text.contains("Main content."));
    }

    #[test]
    fn test_html_preserves_normal_content() {
        let html = r#"<html><body>
            <h1>Normal Page</h1>
            <p>Paragraph one.</p>
            <p>Paragraph two.</p>
        </body></html>"#;
        let doc = parse_html_string(html).unwrap();
        assert!(doc.text.contains("Paragraph one."));
        assert!(doc.text.contains("Paragraph two."));
    }
}
