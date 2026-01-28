//! XLSX/XLS text extraction for KB indexing

use calamine::{open_workbook_auto, Data, Reader};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum XlsxError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Excel parse error: {0}")]
    Parse(String),
    #[error("Calamine error: {0}")]
    Calamine(#[from] calamine::Error),
}

/// Extract text content from an Excel file (XLSX or XLS)
pub fn extract_text(path: &Path) -> Result<String, XlsxError> {
    let mut workbook = open_workbook_auto(path)?;

    let mut text = String::new();
    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

    for sheet_name in sheet_names {
        // Add sheet name as a header
        text.push_str(&format!("## {}\n\n", sheet_name));

        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            for row in range.rows() {
                let row_text: Vec<String> = row.iter().map(cell_to_string).collect();

                // Skip completely empty rows
                if row_text.iter().all(|s| s.is_empty()) {
                    continue;
                }

                text.push_str(&row_text.join(" | "));
                text.push('\n');
            }
        }

        text.push('\n');
    }

    Ok(text)
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            // Format floats nicely (remove trailing zeros)
            if f.fract() == 0.0 {
                format!("{:.0}", f)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => {
            if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        Data::Error(e) => format!("#ERROR: {:?}", e),
        Data::DateTime(dt) => format!("{}", dt),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_nonexistent() {
        let result = extract_text(Path::new("/nonexistent.xlsx"));
        assert!(result.is_err());
    }
}
