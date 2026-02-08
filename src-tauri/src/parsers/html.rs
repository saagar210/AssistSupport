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
