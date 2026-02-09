use serde::{Deserialize, Serialize};

use crate::parsers::Section;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkData {
    pub content: String,
    pub chunk_index: i32,
    pub start_offset: i32,
    pub end_offset: i32,
    pub section_title: Option<String>,
    pub token_count: i32,
}

pub fn chunk_text(
    text: &str,
    sections: &[Section],
    chunk_size: usize,
    chunk_overlap: usize,
) -> Vec<ChunkData> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    // Build a section map: for each character offset, track which section we're in
    let section_at_offset = build_section_map(text, sections);

    // Split on paragraph boundaries
    let paragraphs = split_paragraphs(text);

    if paragraphs.is_empty() {
        return Vec::new();
    }

    let mut chunks: Vec<ChunkData> = Vec::new();
    let mut current_tokens: Vec<String> = Vec::new();
    let mut current_start_offset: usize = 0;
    let mut chunk_index: i32 = 0;

    // Track the byte offset as we walk through paragraphs
    let mut byte_offset: usize = 0;

    for para in &paragraphs {
        let para_tokens: Vec<String> = para.split_whitespace().map(|s| s.to_string()).collect();

        if para_tokens.is_empty() {
            byte_offset += para.len() + 2; // account for \n\n separator
            continue;
        }

        // Check if adding this paragraph would exceed chunk_size
        if !current_tokens.is_empty()
            && current_tokens.len() + para_tokens.len() > chunk_size
        {
            // Emit current chunk
            let content = current_tokens.join(" ");
            let end_offset = byte_offset;
            let section_title = find_section_at(current_start_offset, &section_at_offset);

            let final_content = if let Some(ref title) = section_title {
                format!("[Section: {}] {}", title, content)
            } else {
                content
            };

            let token_count = current_tokens.len() as i32;

            chunks.push(ChunkData {
                content: final_content,
                chunk_index,
                start_offset: current_start_offset as i32,
                end_offset: end_offset as i32,
                section_title,
                token_count,
            });

            chunk_index += 1;

            // Compute overlap: take last chunk_overlap tokens
            let overlap_start = if current_tokens.len() > chunk_overlap {
                current_tokens.len() - chunk_overlap
            } else {
                0
            };
            let overlap: Vec<String> = current_tokens[overlap_start..].to_vec();

            current_tokens.clear();
            current_tokens.extend(overlap);

            current_start_offset = if byte_offset > 0 { byte_offset } else { 0 };
        }

        // Handle very long paragraphs that exceed chunk_size on their own
        if para_tokens.len() > chunk_size && current_tokens.is_empty() {
            let mut i = 0;
            while i < para_tokens.len() {
                let end = std::cmp::min(i + chunk_size, para_tokens.len());
                let slice = &para_tokens[i..end];
                let content = slice.join(" ");
                let section_title = find_section_at(byte_offset + i, &section_at_offset);

                let final_content = if let Some(ref title) = section_title {
                    format!("[Section: {}] {}", title, content)
                } else {
                    content
                };

                chunks.push(ChunkData {
                    content: final_content,
                    chunk_index,
                    start_offset: (byte_offset + i) as i32,
                    end_offset: (byte_offset + end) as i32,
                    section_title,
                    token_count: slice.len() as i32,
                });

                chunk_index += 1;

                // Advance with overlap
                if end < para_tokens.len() && end > chunk_overlap {
                    i = end - chunk_overlap;
                } else {
                    i = end;
                }
            }
            byte_offset += para.len() + 2;
            continue;
        }

        // Add paragraph tokens to current chunk
        current_tokens.extend(para_tokens);

        byte_offset += para.len() + 2; // +2 for \n\n separator
    }

    // Emit final chunk
    if !current_tokens.is_empty() {
        let content = current_tokens.join(" ");
        let section_title = find_section_at(current_start_offset, &section_at_offset);

        let final_content = if let Some(ref title) = section_title {
            format!("[Section: {}] {}", title, content)
        } else {
            content
        };

        let token_count = current_tokens.len() as i32;

        chunks.push(ChunkData {
            content: final_content,
            chunk_index,
            start_offset: current_start_offset as i32,
            end_offset: byte_offset as i32,
            section_title,
            token_count,
        });
    }

    // Handle edge case: if text was smaller than chunk_size and no chunks created
    if chunks.is_empty() && !text.trim().is_empty() {
        let tokens: Vec<&str> = text.split_whitespace().collect();
        let section_title = sections.first().map(|s| s.title.clone());
        let content = tokens.join(" ");
        let final_content = if let Some(ref title) = section_title {
            format!("[Section: {}] {}", title, content)
        } else {
            content
        };
        chunks.push(ChunkData {
            content: final_content,
            chunk_index: 0,
            start_offset: 0,
            end_offset: text.len() as i32,
            section_title,
            token_count: tokens.len() as i32,
        });
    }

    chunks
}

fn split_paragraphs(text: &str) -> Vec<&str> {
    text.split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Build a sorted list of (byte_offset, section_title) for quick lookup.
/// Tracks used positions to handle duplicate section titles correctly —
/// each section gets matched to a unique occurrence in the text.
fn build_section_map(text: &str, sections: &[Section]) -> Vec<(usize, String)> {
    let mut entries: Vec<(usize, String)> = Vec::new();
    let mut used_positions: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for section in sections {
        let mut search_from = 0;
        while search_from < text.len() {
            if let Some(relative_pos) = text[search_from..].find(&section.title) {
                let absolute_pos = search_from + relative_pos;
                if !used_positions.contains(&absolute_pos) {
                    entries.push((absolute_pos, section.title.clone()));
                    used_positions.insert(absolute_pos);
                    break;
                }
                // Position already used by another section — keep searching
                search_from = absolute_pos + 1;
            } else {
                break;
            }
        }
    }

    entries.sort_by_key(|(pos, _)| *pos);
    entries
}

fn find_section_at(offset: usize, section_map: &[(usize, String)]) -> Option<String> {
    let mut current: Option<&String> = None;
    for (pos, title) in section_map {
        if *pos <= offset {
            current = Some(title);
        } else {
            break;
        }
    }
    current.cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::Section;

    #[test]
    fn test_empty_text_returns_empty() {
        let chunks = chunk_text("", &[], 100, 10);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_whitespace_only_returns_empty() {
        let chunks = chunk_text("   \n\n  \t  ", &[], 100, 10);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_small_text_returns_single_chunk() {
        let text = "Hello world this is a short text.";
        let chunks = chunk_text(text, &[], 100, 10);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_index, 0);
        assert!(chunks[0].content.contains("Hello"));
    }

    #[test]
    fn test_text_splits_at_paragraph_boundaries() {
        // Create text with multiple paragraphs that exceed chunk_size
        let para1 = (0..30).map(|i| format!("word{}", i)).collect::<Vec<_>>().join(" ");
        let para2 = (30..60).map(|i| format!("word{}", i)).collect::<Vec<_>>().join(" ");
        let para3 = (60..90).map(|i| format!("word{}", i)).collect::<Vec<_>>().join(" ");
        let text = format!("{}\n\n{}\n\n{}", para1, para2, para3);

        let chunks = chunk_text(&text, &[], 40, 5);
        assert!(chunks.len() >= 2, "Should split into multiple chunks, got {}", chunks.len());

        // Verify chunk indices are sequential
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.chunk_index, i as i32);
        }
    }

    #[test]
    fn test_overlap_works() {
        // Create paragraphs small enough to test overlap behavior
        let para1 = (0..25).map(|i| format!("w{}", i)).collect::<Vec<_>>().join(" ");
        let para2 = (25..50).map(|i| format!("w{}", i)).collect::<Vec<_>>().join(" ");
        let para3 = (50..75).map(|i| format!("w{}", i)).collect::<Vec<_>>().join(" ");
        let text = format!("{}\n\n{}\n\n{}", para1, para2, para3);

        let chunks = chunk_text(&text, &[], 30, 5);

        if chunks.len() >= 2 {
            // The second chunk should contain some words from the end of the first
            let first_words: Vec<&str> = chunks[0].content.split_whitespace().collect();
            let second_words: Vec<&str> = chunks[1].content.split_whitespace().collect();

            // Last 5 words of first chunk should appear at start of second chunk
            if first_words.len() > 5 {
                let overlap_from_first = &first_words[first_words.len() - 5..];
                let start_of_second = &second_words[..5.min(second_words.len())];
                assert_eq!(
                    overlap_from_first, start_of_second,
                    "Overlap tokens should match"
                );
            }
        }
    }

    #[test]
    fn test_section_titles_prepended() {
        let text = "Introduction\n\nThis is the introduction paragraph with enough words to fill it.";
        let sections = vec![Section {
            title: "Introduction".to_string(),
            content: "This is the introduction paragraph with enough words to fill it.".to_string(),
            level: 1,
        }];

        let chunks = chunk_text(text, &sections, 100, 10);
        assert!(!chunks.is_empty());
        assert!(
            chunks[0].content.starts_with("[Section: Introduction]"),
            "Chunk should start with section prefix, got: '{}'",
            chunks[0].content
        );
        assert_eq!(chunks[0].section_title, Some("Introduction".to_string()));
    }

    #[test]
    fn test_chunk_index_sequential() {
        let words: Vec<String> = (0..200).map(|i| format!("word{}", i)).collect();
        // Put every 20 words in a paragraph
        let paragraphs: Vec<String> = words.chunks(20).map(|c| c.join(" ")).collect();
        let text = paragraphs.join("\n\n");

        let chunks = chunk_text(&text, &[], 30, 5);
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.chunk_index, i as i32, "chunk_index should be sequential");
        }
    }

    #[test]
    fn test_token_count_reasonable() {
        let text = "one two three four five";
        let chunks = chunk_text(text, &[], 100, 10);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].token_count, 5);
    }

    #[test]
    fn test_duplicate_section_titles_assigned_correctly() {
        // Two sections both titled "Overview" — each chunk should get the correct one
        let text = "Overview\n\nThis is Product A overview with enough words to be a real paragraph for testing.\n\nDetails\n\nProduct A details paragraph here.\n\nOverview\n\nThis is Product B overview with enough words to be a real paragraph for testing.\n\nDetails\n\nProduct B details paragraph here.";

        let sections = vec![
            Section { title: "Overview".to_string(), content: "This is Product A overview".to_string(), level: 2 },
            Section { title: "Details".to_string(), content: "Product A details".to_string(), level: 2 },
            Section { title: "Overview".to_string(), content: "This is Product B overview".to_string(), level: 2 },
            Section { title: "Details".to_string(), content: "Product B details".to_string(), level: 2 },
        ];

        let chunks = chunk_text(text, &sections, 500, 10);
        assert!(!chunks.is_empty());

        // The section map should have 4 entries (not 2)
        let map = build_section_map(text, &sections);
        assert_eq!(map.len(), 4, "Should have 4 section entries for 4 sections, got {:?}", map);

        // First "Overview" should be at a different position than second "Overview"
        let overview_positions: Vec<usize> = map.iter()
            .filter(|(_, title)| title == "Overview")
            .map(|(pos, _)| *pos)
            .collect();
        assert_eq!(overview_positions.len(), 2, "Should find both Overview positions");
        assert_ne!(overview_positions[0], overview_positions[1], "Duplicate titles should map to different positions");
    }

    #[test]
    fn test_section_map_handles_missing_title() {
        let text = "Some text without any matching headings here.";
        let sections = vec![
            Section { title: "Nonexistent".to_string(), content: String::new(), level: 1 },
        ];

        let map = build_section_map(text, &sections);
        assert!(map.is_empty(), "Should not find nonexistent section title");
    }
}
