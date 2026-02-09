use std::path::Path;

use crate::error::AppError;

use super::{count_words, DocumentMetadata, ParsedDocument};

pub fn parse(path: &Path) -> Result<ParsedDocument, AppError> {
    let mut reader = csv::Reader::from_path(path)
        .map_err(|e| AppError::Parse(format!("Failed to open CSV: {}", e)))?;

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| AppError::Parse(format!("Failed to read CSV headers: {}", e)))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let mut rows: Vec<String> = Vec::new();

    for result in reader.records() {
        let record = result.map_err(|e| AppError::Parse(format!("CSV record error: {}", e)))?;
        let formatted: Vec<String> = headers
            .iter()
            .zip(record.iter())
            .map(|(header, value)| format!("{}: {}", header, value))
            .collect();
        rows.push(formatted.join(", "));
    }

    let text = rows.join("\n");
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
