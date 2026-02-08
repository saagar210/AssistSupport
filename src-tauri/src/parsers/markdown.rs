use std::path::Path;

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::error::AppError;

use super::{count_words, DocumentMetadata, ParsedDocument, Section};

pub fn parse(path: &Path) -> Result<ParsedDocument, AppError> {
    let source = std::fs::read_to_string(path)?;
    let parser = Parser::new(&source);

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
