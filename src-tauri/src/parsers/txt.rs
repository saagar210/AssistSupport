use std::path::Path;

use crate::error::AppError;

use super::{count_words, DocumentMetadata, ParsedDocument};

pub fn parse(path: &Path) -> Result<ParsedDocument, AppError> {
    let bytes = std::fs::read(path)?;

    // Detect encoding and decode to UTF-8
    let (encoding, _confident) = detect_encoding(&bytes);
    let (text, _used_encoding, had_errors) = encoding.decode(&bytes);

    if had_errors {
        tracing::warn!("Encoding errors while decoding file: {:?}", path);
    }

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

fn detect_encoding(bytes: &[u8]) -> (&'static encoding_rs::Encoding, bool) {
    // Check for BOM first
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        return (encoding_rs::UTF_8, true);
    }
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        return (encoding_rs::UTF_16LE, true);
    }
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        return (encoding_rs::UTF_16BE, true);
    }

    // Try UTF-8 first
    if std::str::from_utf8(bytes).is_ok() {
        return (encoding_rs::UTF_8, true);
    }

    // Fallback to Windows-1252 (common for non-UTF-8 Western text)
    (encoding_rs::WINDOWS_1252, false)
}
