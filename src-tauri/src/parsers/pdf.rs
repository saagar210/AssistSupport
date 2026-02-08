use std::path::Path;

use crate::error::AppError;

use super::{count_words, DocumentMetadata, ParsedDocument};

pub fn parse(path: &Path) -> Result<ParsedDocument, AppError> {
    let text = pdf_extract::extract_text(path)
        .map_err(|e| AppError::Parse(format!("Failed to extract PDF text: {}", e)))?;

    let page_count = match lopdf::Document::load(path) {
        Ok(doc) => Some(doc.get_pages().len() as i32),
        Err(_) => None,
    };

    let text = text.trim().to_string();
    let word_count = count_words(&text);

    if text.is_empty() {
        return Ok(ParsedDocument {
            text: String::new(),
            metadata: DocumentMetadata {
                title: None,
                author: None,
                page_count,
                word_count: 0,
            },
            sections: Vec::new(),
        });
    }

    Ok(ParsedDocument {
        text,
        metadata: DocumentMetadata {
            title: None,
            author: None,
            page_count,
            word_count,
        },
        sections: Vec::new(),
    })
}
