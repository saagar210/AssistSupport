use std::io::Read;
use std::path::Path;

use scraper::{Html, Selector};

use crate::error::AppError;

use super::{count_words, DocumentMetadata, ParsedDocument, Section};

pub fn parse(path: &Path) -> Result<ParsedDocument, AppError> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::Parse(format!("Failed to open epub as ZIP: {}", e)))?;

    // Find the OPF file
    let opf_path = find_opf_path(&mut archive)?;

    // Parse OPF to get spine items
    let mut opf_content = String::new();
    {
        let mut opf_file = archive
            .by_name(&opf_path)
            .map_err(|e| AppError::Parse(format!("Failed to read OPF file '{}': {}", opf_path, e)))?;
        opf_file.read_to_string(&mut opf_content)?;
    }

    let content_paths = parse_opf_spine(&opf_content, &opf_path)?;

    // Extract text from each spine item
    let mut full_text = String::new();
    let mut sections: Vec<Section> = Vec::new();

    for content_path in &content_paths {
        let chapter_text = match archive.by_name(content_path) {
            Ok(mut file) => {
                let mut html_content = String::new();
                file.read_to_string(&mut html_content)?;
                extract_text_from_xhtml(&html_content, &mut sections)
            }
            Err(_) => continue, // Skip missing files
        };

        if !chapter_text.is_empty() {
            if !full_text.is_empty() {
                full_text.push_str("\n\n");
            }
            full_text.push_str(&chapter_text);
        }
    }

    let full_text = full_text.trim().to_string();
    let word_count = count_words(&full_text);

    // Try to extract title from first heading or OPF metadata
    let title = sections.first().map(|s| s.title.clone());

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

fn find_opf_path(archive: &mut zip::ZipArchive<std::fs::File>) -> Result<String, AppError> {
    // First check container.xml for the rootfile
    if let Ok(mut container) = archive.by_name("META-INF/container.xml") {
        let mut content = String::new();
        container.read_to_string(&mut content)?;
        if let Some(path) = extract_rootfile_path(&content) {
            return Ok(path);
        }
    }

    // Fallback: scan for .opf file
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| AppError::Parse(format!("Failed to read ZIP entry: {}", e)))?;
        let name = file.name().to_string();
        if name.ends_with(".opf") {
            return Ok(name);
        }
    }

    Err(AppError::Parse(
        "No OPF file found in epub".to_string(),
    ))
}

fn extract_rootfile_path(container_xml: &str) -> Option<String> {
    // Simple extraction of full-path attribute from rootfile element
    let lower = container_xml.to_lowercase();
    if let Some(idx) = lower.find("full-path=") {
        let rest = &container_xml[idx + 10..];
        let quote = rest.chars().next()?;
        if quote == '"' || quote == '\'' {
            let inner = &rest[1..];
            if let Some(end) = inner.find(quote) {
                return Some(inner[..end].to_string());
            }
        }
    }
    None
}

fn parse_opf_spine(opf_content: &str, opf_path: &str) -> Result<Vec<String>, AppError> {
    // Determine base directory of OPF file
    let base_dir = if let Some(pos) = opf_path.rfind('/') {
        &opf_path[..=pos]
    } else {
        ""
    };

    // Extract manifest items (id -> href mapping)
    let mut manifest: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    // Simple XML parsing for manifest items
    for line in opf_content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("<item") && trimmed.contains("id=") && trimmed.contains("href=") {
            if let (Some(id), Some(href)) = (extract_attr(trimmed, "id"), extract_attr(trimmed, "href")) {
                if href.contains("..") {
                    tracing::warn!("Skipping suspicious path in EPUB manifest: {}", href);
                    continue;
                }
                let full_path = if href.starts_with('/') {
                    href[1..].to_string()
                } else {
                    format!("{}{}", base_dir, href)
                };
                manifest.insert(id, full_path);
            }
        }
    }

    // Extract spine itemrefs
    let mut spine_ids: Vec<String> = Vec::new();
    for line in opf_content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("<itemref") {
            if let Some(idref) = extract_attr(trimmed, "idref") {
                spine_ids.push(idref);
            }
        }
    }

    // Map spine IDs to file paths
    let content_paths: Vec<String> = spine_ids
        .iter()
        .filter_map(|id| manifest.get(id).cloned())
        .collect();

    if content_paths.is_empty() {
        // Fallback: return all manifest items that look like content
        let mut paths: Vec<String> = manifest
            .values()
            .filter(|p| p.ends_with(".xhtml") || p.ends_with(".html") || p.ends_with(".htm"))
            .cloned()
            .collect();
        paths.sort();
        return Ok(paths);
    }

    Ok(content_paths)
}

fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    let patterns = [
        format!("{}=\"", attr_name),
        format!("{}='", attr_name),
        format!("{}= \"", attr_name),
        format!("{}= '", attr_name),
        format!("{} =\"", attr_name),
        format!("{} ='", attr_name),
        format!("{} = \"", attr_name),
        format!("{} = '", attr_name),
    ];

    for pattern in &patterns {
        if let Some(start) = tag.find(pattern.as_str()) {
            let value_start = start + pattern.len();
            let quote_char = if pattern.ends_with('"') { '"' } else { '\'' };
            if let Some(end) = tag[value_start..].find(quote_char) {
                return Some(tag[value_start..value_start + end].to_string());
            }
        }
    }
    None
}

fn extract_text_from_xhtml(html_content: &str, sections: &mut Vec<Section>) -> String {
    let document = Html::parse_document(html_content);

    // Extract headings as sections
    for (level, sel_str) in [(1i32, "h1"), (2, "h2"), (3, "h3")] {
        if let Ok(sel) = Selector::parse(sel_str) {
            for el in document.select(&sel) {
                let heading = el.text().collect::<String>().trim().to_string();
                if !heading.is_empty() {
                    sections.push(Section {
                        title: heading,
                        content: String::new(),
                        level,
                    });
                }
            }
        }
    }

    // Extract all text from body
    if let Ok(body_sel) = Selector::parse("body") {
        if let Some(body) = document.select(&body_sel).next() {
            return body.text().collect::<Vec<_>>().join(" ").trim().to_string();
        }
    }

    // Fallback: all text
    document.root_element().text().collect::<Vec<_>>().join(" ").trim().to_string()
}
