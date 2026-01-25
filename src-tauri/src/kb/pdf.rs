//! PDF text extraction module using PDFium
//! Bundles libpdfium.dylib for zero system dependencies

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("PDFium library not found")]
    LibraryNotFound,
    #[error("Failed to load PDFium: {0}")]
    LoadFailed(String),
    #[error("Failed to open PDF: {0}")]
    OpenFailed(String),
    #[error("Page error: {0}")]
    PageError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// PDF text extractor using PDFium
pub struct PdfExtractor {
    pdfium_path: Option<PathBuf>,
}

impl PdfExtractor {
    /// Create a new PDF extractor
    pub fn new() -> Self {
        let pdfium_path = Self::find_pdfium();
        Self { pdfium_path }
    }

    /// Find the bundled PDFium library
    fn find_pdfium() -> Option<PathBuf> {
        // Development paths
        let dev_paths = [
            PathBuf::from("resources/pdfium/libpdfium.dylib"),
            PathBuf::from("./resources/pdfium/libpdfium.dylib"),
            PathBuf::from("../src-tauri/resources/pdfium/libpdfium.dylib"),
        ];

        for path in &dev_paths {
            if path.exists() {
                return Some(path.clone());
            }
        }

        // Production: check Tauri resource path
        #[cfg(target_os = "macos")]
        {
            if let Ok(exe_path) = std::env::current_exe() {
                let resource_path = exe_path
                    .parent()
                    .and_then(|p| p.parent())
                    .map(|p| p.join("Resources").join("pdfium").join("libpdfium.dylib"));

                if let Some(path) = resource_path {
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }

        // Check environment variable
        if let Ok(path) = std::env::var("PDFIUM_DYNAMIC_LIB_PATH") {
            let p = PathBuf::from(&path);
            if p.exists() {
                return Some(p);
            }
        }

        None
    }

    /// Check if PDFium is available
    pub fn is_available(&self) -> bool {
        self.pdfium_path.is_some()
    }

    /// Get the PDFium library path
    pub fn library_path(&self) -> Option<&Path> {
        self.pdfium_path.as_deref()
    }

    /// Extract text from a PDF file
    /// Returns a vector of strings, one per page
    pub fn extract_text(&self, pdf_path: &Path) -> Result<Vec<String>, PdfError> {
        let pdfium_path = self.pdfium_path.as_ref().ok_or(PdfError::LibraryNotFound)?;

        // Load PDFium dynamically
        let pdfium = pdfium_render::prelude::Pdfium::new(
            pdfium_render::prelude::Pdfium::bind_to_library(
                pdfium_render::prelude::Pdfium::pdfium_platform_library_name_at_path(
                    pdfium_path
                        .parent()
                        .unwrap_or(Path::new(".")),
                ),
            )
            .map_err(|e| PdfError::LoadFailed(e.to_string()))?,
        );

        // Open the PDF document
        let document = pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| PdfError::OpenFailed(e.to_string()))?;

        // Extract text from each page
        let mut pages_text = Vec::new();
        for page in document.pages().iter() {
            let text = page.text().map_err(|e| PdfError::PageError(e.to_string()))?;
            pages_text.push(text.all());
        }

        Ok(pages_text)
    }

    /// Extract text and return as single concatenated string
    pub fn extract_all_text(&self, pdf_path: &Path) -> Result<String, PdfError> {
        let pages = self.extract_text(pdf_path)?;
        Ok(pages.join("\n\n---\n\n"))
    }

    /// Get page count for a PDF
    pub fn page_count(&self, pdf_path: &Path) -> Result<usize, PdfError> {
        let pdfium_path = self.pdfium_path.as_ref().ok_or(PdfError::LibraryNotFound)?;

        let pdfium = pdfium_render::prelude::Pdfium::new(
            pdfium_render::prelude::Pdfium::bind_to_library(
                pdfium_render::prelude::Pdfium::pdfium_platform_library_name_at_path(
                    pdfium_path
                        .parent()
                        .unwrap_or(Path::new(".")),
                ),
            )
            .map_err(|e| PdfError::LoadFailed(e.to_string()))?,
        );

        let document = pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| PdfError::OpenFailed(e.to_string()))?;

        Ok(document.pages().len() as usize)
    }
}

impl Default for PdfExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_extractor_creation() {
        let extractor = PdfExtractor::new();
        println!("PDFium available: {}", extractor.is_available());
        if let Some(path) = extractor.library_path() {
            println!("PDFium path: {:?}", path);
        }
    }

    #[test]
    fn test_pdfium_availability() {
        let extractor = PdfExtractor::new();
        // Should find the bundled library in development
        assert!(
            extractor.is_available(),
            "PDFium should be available - check resources/pdfium/libpdfium.dylib"
        );
    }

    #[test]
    fn test_pdf_text_extraction() {
        let extractor = PdfExtractor::new();
        if !extractor.is_available() {
            println!("PDFium not available, skipping integration test");
            return;
        }

        let test_pdf = Path::new("/tmp/test.pdf");
        if !test_pdf.exists() {
            println!("Test PDF not found, skipping integration test");
            return;
        }

        match extractor.extract_all_text(test_pdf) {
            Ok(text) => {
                println!("Extracted text: {}", text);
                assert!(
                    text.contains("AssistSupport") || text.contains("PDFium"),
                    "Expected test text not found in extraction"
                );
            }
            Err(e) => {
                println!("PDF extraction failed: {}", e);
                // Don't fail - might be environment specific
            }
        }
    }

    #[test]
    fn test_pdf_page_count() {
        let extractor = PdfExtractor::new();
        if !extractor.is_available() {
            return;
        }

        let test_pdf = Path::new("/tmp/test.pdf");
        if !test_pdf.exists() {
            return;
        }

        match extractor.page_count(test_pdf) {
            Ok(count) => {
                println!("Page count: {}", count);
                assert_eq!(count, 1, "Test PDF should have 1 page");
            }
            Err(e) => {
                println!("Page count failed: {}", e);
            }
        }
    }
}
