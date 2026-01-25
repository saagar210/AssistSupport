//! KB Document Indexer
//! Handles parsing, chunking, and indexing of documents into the KB

use std::path::{Path, PathBuf};
use thiserror::Error;
use sha2::{Sha256, Digest};
use uuid::Uuid;
use rusqlite::params;

use crate::db::{Database, DbError};
use super::pdf::PdfExtractor;
use super::ocr::OcrManager;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] DbError),
    #[error("PDF error: {0}")]
    Pdf(#[from] super::pdf::PdfError),
    #[error("OCR error: {0}")]
    Ocr(#[from] super::ocr::OcrError),
    #[error("DOCX error: {0}")]
    Docx(#[from] super::docx::DocxError),
    #[error("XLSX error: {0}")]
    Xlsx(#[from] super::xlsx::XlsxError),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
}

/// Document types that can be indexed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentType {
    Markdown,
    Pdf,
    PlainText,
    Image,
    Docx,
    Xlsx,
}

impl DocumentType {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" => Some(Self::Markdown),
            "pdf" => Some(Self::Pdf),
            "txt" | "text" => Some(Self::PlainText),
            "png" | "jpg" | "jpeg" | "gif" | "tiff" | "tif" => Some(Self::Image),
            "docx" => Some(Self::Docx),
            "xlsx" | "xls" => Some(Self::Xlsx),
            _ => None,
        }
    }
}

/// A document chunk for indexing
#[derive(Debug, Clone)]
pub struct Chunk {
    pub heading_path: Option<String>,
    pub content: String,
    pub word_count: usize,
}

/// Parsed document ready for chunking
#[derive(Debug)]
pub struct ParsedDocument {
    pub title: Option<String>,
    pub sections: Vec<Section>,
}

/// A document section with heading
#[derive(Debug)]
pub struct Section {
    pub heading: Option<String>,
    pub level: u8,
    pub content: String,
}

/// Indexing progress event
#[derive(Debug, Clone, serde::Serialize)]
pub enum IndexProgress {
    Started { total_files: usize },
    Processing { current: usize, total: usize, file_name: String },
    Completed { indexed: usize, skipped: usize, errors: usize },
    Error { file_name: String, message: String },
}

/// KB Indexer
pub struct KbIndexer {
    pdf_extractor: PdfExtractor,
    ocr_manager: OcrManager,
    target_chunk_words: usize,
    max_chunk_words: usize,
}

impl KbIndexer {
    /// Create a new KB indexer
    pub fn new() -> Self {
        Self {
            pdf_extractor: PdfExtractor::new(),
            ocr_manager: OcrManager::new(),
            target_chunk_words: 350, // Target 200-500 words
            max_chunk_words: 500,    // Hard cap
        }
    }

    /// Calculate SHA256 hash of file contents
    pub fn file_hash(path: &Path) -> Result<String, IndexerError> {
        let content = std::fs::read(path)?;
        let hash = Sha256::digest(&content);
        Ok(format!("{:x}", hash))
    }

    /// Scan a folder for indexable documents
    pub fn scan_folder(&self, folder: &Path) -> Result<Vec<PathBuf>, IndexerError> {
        let mut files = Vec::new();
        self.scan_recursive(folder, &mut files)?;
        files.sort();
        Ok(files)
    }

    fn scan_recursive(&self, folder: &Path, files: &mut Vec<PathBuf>) -> Result<(), IndexerError> {
        if !folder.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(folder)? {
            let entry = entry?;
            let path = entry.path();

            // Skip hidden files and directories
            if path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }

            if path.is_dir() {
                self.scan_recursive(&path, files)?;
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if DocumentType::from_extension(ext).is_some() {
                    files.push(path);
                }
            }
        }

        Ok(())
    }

    /// Parse a document into sections
    pub fn parse_document(&self, path: &Path) -> Result<ParsedDocument, IndexerError> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let doc_type = DocumentType::from_extension(ext)
            .ok_or_else(|| IndexerError::UnsupportedFileType(ext.to_string()))?;

        match doc_type {
            DocumentType::Markdown => self.parse_markdown(path),
            DocumentType::Pdf => self.parse_pdf(path),
            DocumentType::PlainText => self.parse_plaintext(path),
            DocumentType::Image => self.parse_image(path),
            DocumentType::Docx => self.parse_docx(path),
            DocumentType::Xlsx => self.parse_xlsx(path),
        }
    }

    /// Parse a Markdown file
    fn parse_markdown(&self, path: &Path) -> Result<ParsedDocument, IndexerError> {
        let content = std::fs::read_to_string(path)?;
        let parser = pulldown_cmark::Parser::new(&content);

        let mut title: Option<String> = None;
        let mut sections = Vec::new();
        let mut current_section = Section {
            heading: None,
            level: 0,
            content: String::new(),
        };

        use pulldown_cmark::{Event, Tag, TagEnd};

        let mut in_heading = false;
        let mut heading_level = 0u8;
        let mut heading_text = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    in_heading = true;
                    heading_level = level as u8;
                    heading_text.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    in_heading = false;

                    // Save current section if it has content
                    if !current_section.content.trim().is_empty() {
                        sections.push(current_section);
                    }

                    // Start new section
                    let heading = heading_text.trim().to_string();
                    if title.is_none() && heading_level == 1 {
                        title = Some(heading.clone());
                    }

                    current_section = Section {
                        heading: Some(heading),
                        level: heading_level,
                        content: String::new(),
                    };
                }
                Event::Text(text) | Event::Code(text) => {
                    if in_heading {
                        heading_text.push_str(&text);
                    } else {
                        current_section.content.push_str(&text);
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if !in_heading {
                        current_section.content.push('\n');
                    }
                }
                Event::Start(Tag::Paragraph) => {}
                Event::End(TagEnd::Paragraph) => {
                    current_section.content.push_str("\n\n");
                }
                Event::Start(Tag::List(_)) => {}
                Event::End(TagEnd::List(_)) => {
                    current_section.content.push('\n');
                }
                Event::Start(Tag::Item) => {
                    current_section.content.push_str("- ");
                }
                Event::End(TagEnd::Item) => {
                    current_section.content.push('\n');
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    current_section.content.push_str("\n```\n");
                }
                Event::End(TagEnd::CodeBlock) => {
                    current_section.content.push_str("\n```\n");
                }
                _ => {}
            }
        }

        // Don't forget the last section
        if !current_section.content.trim().is_empty() {
            sections.push(current_section);
        }

        Ok(ParsedDocument { title, sections })
    }

    /// Parse a PDF file
    fn parse_pdf(&self, path: &Path) -> Result<ParsedDocument, IndexerError> {
        let pages = self.pdf_extractor.extract_text(path)?;

        // Check if any page needs OCR (less than 50 chars)
        let mut all_text = String::new();
        for (_i, page_text) in pages.iter().enumerate() {
            if page_text.trim().len() < 50 {
                // Try OCR on this page
                // Note: Would need to render page to image first
                // For now, just use the extracted text
                all_text.push_str(page_text);
            } else {
                all_text.push_str(page_text);
            }
            all_text.push_str("\n\n");
        }

        // Try to extract title from first line
        let title = all_text.lines()
            .next()
            .filter(|l| l.len() < 100 && !l.is_empty())
            .map(|s| s.to_string());

        Ok(ParsedDocument {
            title,
            sections: vec![Section {
                heading: None,
                level: 0,
                content: all_text,
            }],
        })
    }

    /// Parse a plain text file
    fn parse_plaintext(&self, path: &Path) -> Result<ParsedDocument, IndexerError> {
        let content = std::fs::read_to_string(path)?;
        let title = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());

        Ok(ParsedDocument {
            title,
            sections: vec![Section {
                heading: None,
                level: 0,
                content,
            }],
        })
    }

    /// Parse an image file using OCR
    fn parse_image(&self, path: &Path) -> Result<ParsedDocument, IndexerError> {
        let result = self.ocr_manager.recognize(path)?;
        let title = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());

        Ok(ParsedDocument {
            title,
            sections: vec![Section {
                heading: None,
                level: 0,
                content: result.text,
            }],
        })
    }

    /// Parse a DOCX file
    fn parse_docx(&self, path: &Path) -> Result<ParsedDocument, IndexerError> {
        let content = super::docx::extract_text(path)?;
        let title = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());

        Ok(ParsedDocument {
            title,
            sections: vec![Section {
                heading: None,
                level: 0,
                content,
            }],
        })
    }

    /// Parse an Excel file (XLSX/XLS)
    fn parse_xlsx(&self, path: &Path) -> Result<ParsedDocument, IndexerError> {
        let content = super::xlsx::extract_text(path)?;
        let title = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());

        Ok(ParsedDocument {
            title,
            sections: vec![Section {
                heading: None,
                level: 0,
                content,
            }],
        })
    }

    /// Chunk a parsed document
    pub fn chunk_document(&self, doc: &ParsedDocument) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut current_heading_path = Vec::new();

        for section in &doc.sections {
            // Update heading path based on level
            if let Some(heading) = &section.heading {
                let level = section.level as usize;
                // Truncate to current level and add new heading
                if level > 0 && level <= current_heading_path.len() {
                    current_heading_path.truncate(level - 1);
                }
                current_heading_path.push(heading.clone());
            }

            // Chunk the section content
            let heading_path = if current_heading_path.is_empty() {
                None
            } else {
                Some(current_heading_path.join(" > "))
            };

            let section_chunks = self.chunk_text(&section.content, heading_path);
            chunks.extend(section_chunks);
        }

        chunks
    }

    /// Chunk text into appropriately sized pieces
    fn chunk_text(&self, text: &str, heading_path: Option<String>) -> Vec<Chunk> {
        let words: Vec<&str> = text.split_whitespace().collect();

        if words.is_empty() {
            return vec![];
        }

        // If small enough, return as single chunk
        if words.len() <= self.max_chunk_words {
            return vec![Chunk {
                heading_path,
                content: text.trim().to_string(),
                word_count: words.len(),
            }];
        }

        // Split into chunks at paragraph boundaries when possible
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut chunks = Vec::new();
        let mut current_words = Vec::new();

        for para in paragraphs {
            let para_words: Vec<&str> = para.split_whitespace().collect();

            if current_words.len() + para_words.len() > self.max_chunk_words {
                // Save current chunk if it has content
                if !current_words.is_empty() {
                    chunks.push(Chunk {
                        heading_path: heading_path.clone(),
                        content: current_words.join(" "),
                        word_count: current_words.len(),
                    });
                    current_words.clear();
                }

                // If single paragraph exceeds max, split by sentences
                if para_words.len() > self.max_chunk_words {
                    let sentence_chunks = self.split_large_paragraph(para, heading_path.clone());
                    chunks.extend(sentence_chunks);
                } else {
                    current_words.extend(para_words);
                }
            } else {
                current_words.extend(para_words);
            }

            // Check if we've hit target size
            if current_words.len() >= self.target_chunk_words {
                chunks.push(Chunk {
                    heading_path: heading_path.clone(),
                    content: current_words.join(" "),
                    word_count: current_words.len(),
                });
                current_words.clear();
            }
        }

        // Don't forget remaining content
        if !current_words.is_empty() {
            chunks.push(Chunk {
                heading_path,
                content: current_words.join(" "),
                word_count: current_words.len(),
            });
        }

        chunks
    }

    /// Split a large paragraph by sentences, or by word count if needed
    fn split_large_paragraph(&self, text: &str, heading_path: Option<String>) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut current_words = Vec::new();

        // Simple sentence splitting (could be improved)
        for sentence in text.split(|c| c == '.' || c == '!' || c == '?') {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }

            let words: Vec<&str> = sentence.split_whitespace().collect();

            // If this single sentence exceeds max, split it by word count
            if words.len() > self.max_chunk_words {
                // Flush current buffer first
                if !current_words.is_empty() {
                    chunks.push(Chunk {
                        heading_path: heading_path.clone(),
                        content: current_words.join(" "),
                        word_count: current_words.len(),
                    });
                    current_words.clear();
                }

                // Split the long sentence by word count
                for word in words {
                    current_words.push(word);
                    if current_words.len() >= self.max_chunk_words {
                        chunks.push(Chunk {
                            heading_path: heading_path.clone(),
                            content: current_words.join(" "),
                            word_count: current_words.len(),
                        });
                        current_words.clear();
                    }
                }
                continue;
            }

            if current_words.len() + words.len() > self.max_chunk_words {
                if !current_words.is_empty() {
                    chunks.push(Chunk {
                        heading_path: heading_path.clone(),
                        content: current_words.join(" "),
                        word_count: current_words.len(),
                    });
                    current_words.clear();
                }
            }

            current_words.extend(words);

            if current_words.len() >= self.target_chunk_words {
                chunks.push(Chunk {
                    heading_path: heading_path.clone(),
                    content: current_words.join(" "),
                    word_count: current_words.len(),
                });
                current_words.clear();
            }
        }

        if !current_words.is_empty() {
            chunks.push(Chunk {
                heading_path,
                content: current_words.join(" "),
                word_count: current_words.len(),
            });
        }

        chunks
    }

    /// Index a single document into the database
    pub fn index_document(&self, db: &Database, path: &Path) -> Result<usize, IndexerError> {
        let file_hash = Self::file_hash(path)?;
        let file_path = path.to_string_lossy().to_string();

        // Check if already indexed with same hash
        let existing: Option<String> = db.conn()
            .query_row(
                "SELECT file_hash FROM kb_documents WHERE file_path = ?",
                params![&file_path],
                |row| row.get(0),
            )
            .ok();

        if existing.as_ref() == Some(&file_hash) {
            // Already indexed with same content
            return Ok(0);
        }

        // Parse the document
        let parsed = self.parse_document(path)?;

        // Chunk the document
        let chunks = self.chunk_document(&parsed);

        if chunks.is_empty() {
            return Ok(0);
        }

        // Generate document ID
        let doc_id = if existing.is_some() {
            // Update existing document - get its ID
            let id: String = db.conn()
                .query_row(
                    "SELECT id FROM kb_documents WHERE file_path = ?",
                    params![&file_path],
                    |row| row.get(0),
                )
                .map_err(|e| IndexerError::Database(DbError::Sqlite(e)))?;

            // Delete old chunks (triggers will clean up FTS5)
            db.conn()
                .execute("DELETE FROM kb_chunks WHERE document_id = ?", params![&id])
                .map_err(|e| IndexerError::Database(DbError::Sqlite(e)))?;

            id
        } else {
            Uuid::new_v4().to_string()
        };

        // Insert or update document record
        let now = chrono::Utc::now().to_rfc3339();
        let title = parsed.title
            .or_else(|| path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()));

        db.conn()
            .execute(
                "INSERT OR REPLACE INTO kb_documents (id, file_path, file_hash, title, indexed_at, chunk_count)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![&doc_id, &file_path, &file_hash, &title, &now, chunks.len() as i64],
            )
            .map_err(|e| IndexerError::Database(DbError::Sqlite(e)))?;

        // Insert chunks
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_id = Uuid::new_v4().to_string();
            db.conn()
                .execute(
                    "INSERT INTO kb_chunks (id, document_id, chunk_index, heading_path, content, word_count)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        &chunk_id,
                        &doc_id,
                        i as i64,
                        &chunk.heading_path,
                        &chunk.content,
                        chunk.word_count as i64,
                    ],
                )
                .map_err(|e| IndexerError::Database(DbError::Sqlite(e)))?;
        }

        Ok(chunks.len())
    }

    /// Index an entire folder
    pub fn index_folder<F>(
        &self,
        db: &Database,
        folder: &Path,
        progress_callback: F,
    ) -> Result<IndexResult, IndexerError>
    where
        F: Fn(IndexProgress),
    {
        let files = self.scan_folder(folder)?;
        let total = files.len();

        progress_callback(IndexProgress::Started { total_files: total });

        let mut indexed = 0;
        let mut skipped = 0;
        let mut errors = 0;

        for (i, file) in files.iter().enumerate() {
            let file_name = file.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            progress_callback(IndexProgress::Processing {
                current: i + 1,
                total,
                file_name: file_name.clone(),
            });

            match self.index_document(db, file) {
                Ok(0) => skipped += 1,
                Ok(_) => indexed += 1,
                Err(e) => {
                    errors += 1;
                    progress_callback(IndexProgress::Error {
                        file_name,
                        message: e.to_string(),
                    });
                }
            }
        }

        progress_callback(IndexProgress::Completed {
            indexed,
            skipped,
            errors,
        });

        Ok(IndexResult {
            total_files: total,
            indexed,
            skipped,
            errors,
        })
    }

    /// Remove a document from the index
    pub fn remove_document(&self, db: &Database, file_path: &str) -> Result<bool, IndexerError> {
        // Get document ID
        let doc_id: Option<String> = db.conn()
            .query_row(
                "SELECT id FROM kb_documents WHERE file_path = ?",
                params![file_path],
                |row| row.get(0),
            )
            .ok();

        if let Some(id) = doc_id {
            // Delete document (cascade will delete chunks, triggers clean FTS5)
            db.conn()
                .execute("DELETE FROM kb_documents WHERE id = ?", params![&id])
                .map_err(|e| IndexerError::Database(DbError::Sqlite(e)))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get indexing statistics
    pub fn get_stats(&self, db: &Database) -> Result<IndexStats, IndexerError> {
        let doc_count: i64 = db.conn()
            .query_row("SELECT COUNT(*) FROM kb_documents", [], |row| row.get(0))
            .map_err(|e| IndexerError::Database(DbError::Sqlite(e)))?;

        let chunk_count: i64 = db.conn()
            .query_row("SELECT COUNT(*) FROM kb_chunks", [], |row| row.get(0))
            .map_err(|e| IndexerError::Database(DbError::Sqlite(e)))?;

        let total_words: i64 = db.conn()
            .query_row("SELECT COALESCE(SUM(word_count), 0) FROM kb_chunks", [], |row| row.get(0))
            .map_err(|e| IndexerError::Database(DbError::Sqlite(e)))?;

        Ok(IndexStats {
            document_count: doc_count as usize,
            chunk_count: chunk_count as usize,
            total_words: total_words as usize,
        })
    }
}

impl Default for KbIndexer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of indexing operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexResult {
    pub total_files: usize,
    pub indexed: usize,
    pub skipped: usize,
    pub errors: usize,
}

/// KB statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexStats {
    pub document_count: usize,
    pub chunk_count: usize,
    pub total_words: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_document_type_detection() {
        assert_eq!(DocumentType::from_extension("md"), Some(DocumentType::Markdown));
        assert_eq!(DocumentType::from_extension("PDF"), Some(DocumentType::Pdf));
        assert_eq!(DocumentType::from_extension("txt"), Some(DocumentType::PlainText));
        assert_eq!(DocumentType::from_extension("png"), Some(DocumentType::Image));
        assert_eq!(DocumentType::from_extension("docx"), None);
    }

    #[test]
    fn test_markdown_parsing() {
        let dir = tempdir().unwrap();
        let md_path = dir.path().join("test.md");
        std::fs::write(&md_path, r#"# Main Title

This is the introduction.

## Section One

Some content in section one.
More content here.

## Section Two

Content in section two.

### Subsection

Nested content.
"#).unwrap();

        let indexer = KbIndexer::new();
        let doc = indexer.parse_document(&md_path).unwrap();

        assert_eq!(doc.title, Some("Main Title".to_string()));
        assert!(doc.sections.len() >= 3);
    }

    #[test]
    fn test_chunking() {
        let indexer = KbIndexer::new();

        let doc = ParsedDocument {
            title: Some("Test".to_string()),
            sections: vec![
                Section {
                    heading: Some("Intro".to_string()),
                    level: 1,
                    content: "This is a short introduction.".to_string(),
                },
                Section {
                    heading: Some("Body".to_string()),
                    level: 2,
                    content: "A ".repeat(600), // ~600 words
                },
            ],
        };

        let chunks = indexer.chunk_document(&doc);

        // Should have multiple chunks due to long content
        assert!(!chunks.is_empty());

        // All chunks should be within limits
        for chunk in &chunks {
            assert!(chunk.word_count <= indexer.max_chunk_words);
        }
    }

    #[test]
    fn test_file_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();

        let hash1 = KbIndexer::file_hash(&file_path).unwrap();
        let hash2 = KbIndexer::file_hash(&file_path).unwrap();
        assert_eq!(hash1, hash2);

        // Change content
        std::fs::write(&file_path, "different content").unwrap();
        let hash3 = KbIndexer::file_hash(&file_path).unwrap();
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_folder_scan() {
        let dir = tempdir().unwrap();

        // Create some test files
        std::fs::write(dir.path().join("doc1.md"), "# Test").unwrap();
        std::fs::write(dir.path().join("doc2.txt"), "Plain text").unwrap();
        std::fs::write(dir.path().join("ignored.docx"), "").unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();
        std::fs::write(dir.path().join("subdir/nested.md"), "# Nested").unwrap();

        let indexer = KbIndexer::new();
        let files = indexer.scan_folder(dir.path()).unwrap();

        assert_eq!(files.len(), 3); // doc1.md, doc2.txt, nested.md
    }

    #[test]
    fn test_index_and_search_integration() {
        use crate::security::MasterKey;
        use crate::db::Database;

        // Create temp database
        let db_dir = tempdir().unwrap();
        let db_path = db_dir.path().join("test.db");
        let key = MasterKey::generate();
        let db = Database::open(&db_path, &key).unwrap();
        db.initialize().unwrap();

        // Create test KB folder
        let kb_dir = tempdir().unwrap();
        std::fs::write(
            kb_dir.path().join("vpn-troubleshooting.md"),
            r#"# VPN Troubleshooting Guide

## Connection Issues

If you cannot connect to the VPN, try these steps:

1. Check your internet connection
2. Restart the VPN client
3. Verify your credentials

## Authentication Errors

Authentication failures often occur when:

- Password has expired
- Account is locked
- MFA device is not registered
"#,
        ).unwrap();

        std::fs::write(
            kb_dir.path().join("password-reset.md"),
            r#"# Password Reset Procedures

## Self-Service Reset

Users can reset their own password using the self-service portal.

## Admin-Assisted Reset

If self-service is not available, contact IT support for an admin reset.
"#,
        ).unwrap();

        // Index the folder
        let indexer = KbIndexer::new();
        let result = indexer.index_folder(&db, kb_dir.path(), |_| {}).unwrap();

        assert_eq!(result.indexed, 2);
        assert_eq!(result.errors, 0);

        // Check stats
        let stats = indexer.get_stats(&db).unwrap();
        assert_eq!(stats.document_count, 2);
        assert!(stats.chunk_count >= 2);

        // Test FTS5 search
        let results = db.fts_search("VPN connection", 10).unwrap();
        assert!(!results.is_empty(), "Should find VPN results");

        let results = db.fts_search("password reset", 10).unwrap();
        assert!(!results.is_empty(), "Should find password results");

        let results = db.fts_search("authentication failures", 10).unwrap();
        assert!(!results.is_empty(), "Should find auth results");

        // Test incremental update (re-index same content should skip)
        let result2 = indexer.index_folder(&db, kb_dir.path(), |_| {}).unwrap();
        assert_eq!(result2.skipped, 2);
        assert_eq!(result2.indexed, 0);
    }
}
