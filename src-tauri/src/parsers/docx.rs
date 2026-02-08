use std::io::Read;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;

use super::{count_words, DocumentMetadata, ParsedDocument};

pub fn parse(path: &Path) -> Result<ParsedDocument, AppError> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::Parse(format!("Failed to open docx as ZIP: {}", e)))?;

    let mut xml_content = String::new();
    {
        let mut doc_file = archive
            .by_name("word/document.xml")
            .map_err(|e| AppError::Parse(format!("No word/document.xml found in docx: {}", e)))?;
        doc_file.read_to_string(&mut xml_content)?;
    }

    let paragraphs = extract_paragraphs(&xml_content)?;
    let text = paragraphs.join("\n\n");
    let text = text.trim().to_string();
    let word_count = count_words(&text);

    Ok(ParsedDocument {
        text,
        metadata: DocumentMetadata {
            title: None,
            author: None,
            page_count: None,
            word_count,
        },
        sections: Vec::new(),
    })
}

fn extract_paragraphs(xml: &str) -> Result<Vec<String>, AppError> {
    let mut reader = Reader::from_str(xml);
    let mut paragraphs: Vec<String> = Vec::new();
    let mut current_paragraph = String::new();
    let mut in_paragraph = false;
    let mut in_text = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"p" {
                    in_paragraph = true;
                    current_paragraph.clear();
                } else if local.as_ref() == b"t" && in_paragraph {
                    in_text = true;
                }
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                if local.as_ref() == b"p" {
                    if in_paragraph && !current_paragraph.trim().is_empty() {
                        paragraphs.push(current_paragraph.trim().to_string());
                    }
                    in_paragraph = false;
                    current_paragraph.clear();
                } else if local.as_ref() == b"t" {
                    in_text = false;
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_text {
                    let text = reader.decoder().decode(e.as_ref())
                        .map_err(|err| AppError::Parse(format!("XML text decode error: {}", err)))?;
                    current_paragraph.push_str(&text);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(AppError::Parse(format!(
                    "Error parsing docx XML at position {}: {}",
                    reader.error_position(),
                    e
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(paragraphs)
}
