pub mod csv_parser;
pub mod docx;
pub mod epub;
pub mod html;
pub mod markdown;
pub mod pdf;
pub mod txt;

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub text: String,
    pub metadata: DocumentMetadata,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: Option<i32>,
    pub word_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub title: String,
    pub content: String,
    pub level: i32,
}

fn count_words(text: &str) -> i32 {
    text.split_whitespace().count() as i32
}

pub fn parse_document(path: &Path, file_type: &str) -> Result<ParsedDocument, AppError> {
    // Validate file type first
    match file_type {
        "pdf" | "md" | "markdown" | "html" | "htm" | "txt" | "text" | "docx" | "csv" | "epub" => {}
        _ => {
            return Err(AppError::Parse(format!(
                "Unsupported file type: {}",
                file_type
            )));
        }
    }

    const MAX_FILE_SIZE: u64 = 500 * 1024 * 1024; // 500MB limit

    let file_size = std::fs::metadata(path)
        .map_err(AppError::Io)?
        .len();

    if file_size > MAX_FILE_SIZE {
        return Err(AppError::Validation(format!(
            "File too large: {} bytes (max {} bytes)",
            file_size, MAX_FILE_SIZE
        )));
    }

    match file_type {
        "pdf" => pdf::parse(path),
        "md" | "markdown" => markdown::parse(path),
        "html" | "htm" => html::parse(path),
        "txt" | "text" => txt::parse(path),
        "docx" => docx::parse(path),
        "csv" => csv_parser::parse(path),
        "epub" => epub::parse(path),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    // --- TXT parser ---

    #[test]
    fn test_txt_parser_content() {
        let path = fixtures_dir().join("test.txt");
        let result = parse_document(&path, "txt").unwrap();
        assert_eq!(result.text, "Hello VaultMind. This is a test document.");
        assert_eq!(result.metadata.word_count, 7);
    }

    // --- Markdown parser ---

    #[test]
    fn test_markdown_parser_sections() {
        let path = fixtures_dir().join("test.md");
        let result = parse_document(&path, "md").unwrap();

        assert!(!result.text.is_empty());
        assert!(result.sections.len() >= 3, "Should have Title + 2 sections, got {}", result.sections.len());

        let titles: Vec<&str> = result.sections.iter().map(|s| s.title.as_str()).collect();
        assert!(titles.contains(&"Title"));
        assert!(titles.contains(&"Section 1"));
        assert!(titles.contains(&"Section 2"));
    }

    #[test]
    fn test_markdown_parser_heading_levels() {
        let path = fixtures_dir().join("test.md");
        let result = parse_document(&path, "md").unwrap();

        let title_section = result.sections.iter().find(|s| s.title == "Title").unwrap();
        assert_eq!(title_section.level, 1);

        let s1 = result.sections.iter().find(|s| s.title == "Section 1").unwrap();
        assert_eq!(s1.level, 2);
    }

    // --- CSV parser ---

    #[test]
    fn test_csv_parser_rows() {
        let path = fixtures_dir().join("test.csv");
        let result = parse_document(&path, "csv").unwrap();

        assert!(result.text.contains("Alice"));
        assert!(result.text.contains("Bob"));
        assert!(result.text.contains("Name:"));
        assert!(result.text.contains("Age:"));
        assert!(result.text.contains("City:"));

        // Should have 2 rows (excluding header)
        let lines: Vec<&str> = result.text.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    // --- HTML parser ---

    #[test]
    fn test_html_parser_text_extracted() {
        let path = fixtures_dir().join("test.html");
        let result = parse_document(&path, "html").unwrap();

        assert!(result.text.contains("Title"), "Should extract h1 text");
        assert!(result.text.contains("Paragraph text"), "Should extract p text");
    }

    #[test]
    fn test_html_parser_script_stripped() {
        let path = fixtures_dir().join("test.html");
        let result = parse_document(&path, "html").unwrap();

        assert!(!result.text.contains("alert"), "Script content should be stripped");
    }

    #[test]
    fn test_html_parser_sections() {
        let path = fixtures_dir().join("test.html");
        let result = parse_document(&path, "html").unwrap();

        let titles: Vec<&str> = result.sections.iter().map(|s| s.title.as_str()).collect();
        assert!(titles.contains(&"Title"), "Should extract h1 as section");
    }

    // --- PDF parser (error handling) ---

    #[test]
    fn test_pdf_parser_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/file.pdf");
        let result = parse_document(&path, "pdf");
        assert!(result.is_err());
    }

    #[test]
    fn test_pdf_parser_invalid_file() {
        // Feed the txt fixture as a PDF -- should fail
        let path = fixtures_dir().join("test.txt");
        let result = parse_document(&path, "pdf");
        assert!(result.is_err());
    }

    // --- DOCX parser (error handling) ---

    #[test]
    fn test_docx_parser_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/file.docx");
        let result = parse_document(&path, "docx");
        assert!(result.is_err());
    }

    #[test]
    fn test_docx_parser_invalid_file() {
        let path = fixtures_dir().join("test.txt");
        let result = parse_document(&path, "docx");
        assert!(result.is_err());
    }

    // --- EPUB parser (error handling) ---

    #[test]
    fn test_epub_parser_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/file.epub");
        let result = parse_document(&path, "epub");
        assert!(result.is_err());
    }

    #[test]
    fn test_epub_parser_invalid_file() {
        let path = fixtures_dir().join("test.txt");
        let result = parse_document(&path, "epub");
        assert!(result.is_err());
    }

    // --- Unsupported type ---

    #[test]
    fn test_unsupported_file_type() {
        let path = PathBuf::from("/some/file.xyz");
        let result = parse_document(&path, "xyz");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Unsupported file type"));
    }
}
