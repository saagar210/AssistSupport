//! DOCX text extraction for KB indexing

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocxError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("DOCX parse error: {0}")]
    Parse(String),
}

/// Extract text content from a DOCX file
pub fn extract_text(path: &Path) -> Result<String, DocxError> {
    let file_bytes = std::fs::read(path)?;

    let doc = docx_rs::read_docx(&file_bytes).map_err(|e| DocxError::Parse(e.to_string()))?;

    let mut text = String::new();

    // Iterate through the document body children
    for child in &doc.document.children {
        extract_from_document_child(child, &mut text);
    }

    Ok(text)
}

fn extract_from_document_child(child: &docx_rs::DocumentChild, text: &mut String) {
    match child {
        docx_rs::DocumentChild::Paragraph(p) => {
            extract_from_paragraph(p, text);
            text.push('\n');
        }
        docx_rs::DocumentChild::Table(table) => {
            extract_from_table(table, text);
            text.push('\n');
        }
        _ => {}
    }
}

fn extract_from_paragraph(p: &docx_rs::Paragraph, text: &mut String) {
    for child in &p.children {
        match child {
            docx_rs::ParagraphChild::Run(run) => {
                extract_from_run(run, text);
            }
            docx_rs::ParagraphChild::Hyperlink(hl) => {
                for run in &hl.children {
                    if let docx_rs::ParagraphChild::Run(r) = run {
                        extract_from_run(r, text);
                    }
                }
            }
            _ => {}
        }
    }
}

fn extract_from_run(run: &docx_rs::Run, text: &mut String) {
    for child in &run.children {
        match child {
            docx_rs::RunChild::Text(t) => {
                text.push_str(&t.text);
            }
            docx_rs::RunChild::Tab(_) => {
                text.push('\t');
            }
            docx_rs::RunChild::Break(_) => {
                text.push('\n');
            }
            _ => {}
        }
    }
}

fn extract_from_table(table: &docx_rs::Table, text: &mut String) {
    for row in &table.rows {
        let docx_rs::TableChild::TableRow(tr) = row;
        let mut row_texts = Vec::new();
        for cell in &tr.cells {
            let docx_rs::TableRowChild::TableCell(tc) = cell;
            let mut cell_text = String::new();
            for content in &tc.children {
                if let docx_rs::TableCellContent::Paragraph(p) = content {
                    extract_from_paragraph(p, &mut cell_text);
                }
            }
            row_texts.push(cell_text.trim().to_string());
        }
        if !row_texts.is_empty() {
            text.push_str(&row_texts.join(" | "));
            text.push('\n');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_nonexistent() {
        let result = extract_text(Path::new("/nonexistent.docx"));
        assert!(result.is_err());
    }
}
