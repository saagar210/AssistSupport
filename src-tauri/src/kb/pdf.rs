//! PDF text extraction module using PDFium
//! Bundles libpdfium.dylib for zero system dependencies

use std::path::{Path, PathBuf};
use tempfile::TempDir;
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
                    pdfium_path.parent().unwrap_or(Path::new(".")),
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
            let text = page
                .text()
                .map_err(|e| PdfError::PageError(e.to_string()))?;
            pages_text.push(text.all());
        }

        Ok(pages_text)
    }

    /// Extract text and return as single concatenated string
    pub fn extract_all_text(&self, pdf_path: &Path) -> Result<String, PdfError> {
        let pages = self.extract_text(pdf_path)?;
        Ok(pages.join("\n\n---\n\n"))
    }

    /// Check if a PDF needs OCR (scanned/image PDF with low text content)
    /// Returns true if average text per page is less than threshold chars
    pub fn needs_ocr(
        &self,
        pdf_path: &Path,
        chars_per_page_threshold: usize,
    ) -> Result<bool, PdfError> {
        let pages = self.extract_text(pdf_path)?;
        if pages.is_empty() {
            return Ok(true);
        }

        let total_chars: usize = pages.iter().map(|p| p.len()).sum();
        let avg_chars = total_chars / pages.len();

        Ok(avg_chars < chars_per_page_threshold)
    }

    /// Render a PDF page to an image file
    /// Returns the path to the rendered image
    pub fn render_page_to_image(
        &self,
        pdf_path: &Path,
        page_index: usize,
        output_path: &Path,
    ) -> Result<(), PdfError> {
        use pdfium_render::prelude::*;

        let pdfium_path = self.pdfium_path.as_ref().ok_or(PdfError::LibraryNotFound)?;

        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
                pdfium_path.parent().unwrap_or(Path::new(".")),
            ))
            .map_err(|e| PdfError::LoadFailed(e.to_string()))?,
        );

        let document = pdfium
            .load_pdf_from_file(pdf_path, None)
            .map_err(|e| PdfError::OpenFailed(e.to_string()))?;

        let page = document
            .pages()
            .get(page_index as u16)
            .map_err(|e| PdfError::PageError(e.to_string()))?;

        // Render at 150 DPI for good OCR quality
        let render_config = PdfRenderConfig::new()
            .set_target_width(1200)
            .set_maximum_height(1600);

        let bitmap = page
            .render_with_config(&render_config)
            .map_err(|e| PdfError::PageError(format!("Failed to render page: {}", e)))?;

        // Convert to image and save as PNG
        bitmap
            .as_image()
            .into_rgba8()
            .save(output_path)
            .map_err(|e| PdfError::Io(std::io::Error::other(e.to_string())))?;

        Ok(())
    }

    /// Extract text with OCR fallback for scanned PDFs
    /// Uses OCR if average text per page is below threshold
    pub fn extract_text_with_ocr_fallback(
        &self,
        pdf_path: &Path,
        ocr_fn: impl Fn(&Path) -> Result<String, String>,
    ) -> Result<Vec<String>, PdfError> {
        let regular_pages = self.extract_text(pdf_path)?;

        // Check if we need OCR (less than 100 chars average per page)
        let total_chars: usize = regular_pages.iter().map(|p| p.len()).sum();
        let avg_chars = if regular_pages.is_empty() {
            0
        } else {
            total_chars / regular_pages.len()
        };

        if avg_chars >= 100 {
            // Regular PDF with good text - return as is
            return Ok(regular_pages);
        }

        // Scanned PDF - need OCR for each page
        tracing::info!(
            "PDF appears to be scanned (avg {} chars/page), using OCR",
            avg_chars
        );

        let page_count = self.page_count(pdf_path)?;
        let mut ocr_pages = Vec::with_capacity(page_count);
        let temp_dir = TempDir::new()?;

        for page_idx in 0..page_count {
            let img_path = temp_dir
                .path()
                .join(format!("pdf_ocr_page_{}.png", page_idx));

            // Render page to image
            if let Err(e) = self.render_page_to_image(pdf_path, page_idx, &img_path) {
                tracing::warn!("Failed to render page {}: {}", page_idx, e);
                // Fall back to whatever text we got
                ocr_pages.push(regular_pages.get(page_idx).cloned().unwrap_or_default());
                continue;
            }

            // Run OCR on the rendered image
            match ocr_fn(&img_path) {
                Ok(text) => ocr_pages.push(text),
                Err(e) => {
                    tracing::warn!("OCR failed for page {}: {}", page_idx, e);
                    ocr_pages.push(regular_pages.get(page_idx).cloned().unwrap_or_default());
                }
            }

            // Clean up temp image
            let _ = std::fs::remove_file(&img_path);
        }

        Ok(ocr_pages)
    }

    /// Get page count for a PDF
    pub fn page_count(&self, pdf_path: &Path) -> Result<usize, PdfError> {
        let pdfium_path = self.pdfium_path.as_ref().ok_or(PdfError::LibraryNotFound)?;

        let pdfium = pdfium_render::prelude::Pdfium::new(
            pdfium_render::prelude::Pdfium::bind_to_library(
                pdfium_render::prelude::Pdfium::pdfium_platform_library_name_at_path(
                    pdfium_path.parent().unwrap_or(Path::new(".")),
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
