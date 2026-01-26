//! Filter Injection Tests
//!
//! Tests for LanceDB filter sanitization to prevent injection attacks.
//! Covers OWASP filter bypass patterns, Unicode injection, and legitimate edge cases.

use unicode_normalization::UnicodeNormalization;

/// Characters that look like quotes or operators but are Unicode variants
fn is_unicode_confusable(c: char) -> bool {
    matches!(
        c,
        // Apostrophe lookalikes
        '\u{02BC}' | // MODIFIER LETTER APOSTROPHE
        '\u{02B9}' | // MODIFIER LETTER PRIME
        '\u{2018}' | // LEFT SINGLE QUOTATION MARK
        '\u{2019}' | // RIGHT SINGLE QUOTATION MARK
        '\u{201B}' | // SINGLE HIGH-REVERSED-9 QUOTATION MARK
        '\u{FF07}' | // FULLWIDTH APOSTROPHE
        // Quote lookalikes
        '\u{02BA}' | // MODIFIER LETTER DOUBLE PRIME
        '\u{201C}' | // LEFT DOUBLE QUOTATION MARK
        '\u{201D}' | // RIGHT DOUBLE QUOTATION MARK
        '\u{201F}' | // DOUBLE HIGH-REVERSED-9 QUOTATION MARK
        '\u{FF02}' | // FULLWIDTH QUOTATION MARK
        // Operator lookalikes
        '\u{FF1D}' | // FULLWIDTH EQUALS SIGN
        '\u{2260}' | // NOT EQUAL TO
        '\u{FF1C}' | // FULLWIDTH LESS-THAN SIGN
        '\u{FF1E}' | // FULLWIDTH GREATER-THAN SIGN
        // Hyphen/minus lookalikes
        '\u{2010}' | // HYPHEN
        '\u{2011}' | // NON-BREAKING HYPHEN
        '\u{2012}' | // FIGURE DASH
        '\u{2013}' | // EN DASH
        '\u{2014}' | // EM DASH
        '\u{2015}' | // HORIZONTAL BAR
        '\u{FE58}' | // SMALL EM DASH
        '\u{FF0D}' | // FULLWIDTH HYPHEN-MINUS
        // Semicolon lookalikes
        '\u{037E}' | // GREEK QUESTION MARK (looks like ;)
        '\u{FF1B}'   // FULLWIDTH SEMICOLON
    )
}

/// Sanitize a filter value for use in LanceDB queries
fn sanitize_filter_value(value: &str) -> Option<String> {
    // Unicode normalize (NFC)
    let normalized: String = value.nfc().collect();

    // Check for confusable characters
    if normalized.chars().any(is_unicode_confusable) {
        return None;
    }

    // Create ASCII-only lowercase version for pattern matching
    let ascii_lower: String = normalized
        .chars()
        .filter(|c| c.is_ascii())
        .flat_map(|c| c.to_lowercase())
        .collect();

    // SQL keywords to block (with word-boundary awareness)
    let sql_keywords = [
        "select", "insert", "update", "delete", "drop", "union",
    ];

    let has_boundary_before = |pos: usize| -> bool {
        pos == 0
            || matches!(
                ascii_lower.as_bytes().get(pos - 1),
                Some(b' ' | b'\'' | b'"' | b'(' | b')' | b';' | b',')
            )
    };
    let has_boundary_after = |pos: usize| -> bool {
        pos >= ascii_lower.len()
            || matches!(
                ascii_lower.as_bytes().get(pos),
                Some(b' ' | b'\'' | b'"' | b'(' | b')' | b';' | b',') | None
            )
    };

    for keyword in &sql_keywords {
        let kw_len = keyword.len();
        let mut search_from = 0;
        while let Some(rel_pos) = ascii_lower[search_from..].find(keyword) {
            let pos = search_from + rel_pos;
            let end = pos + kw_len;
            if has_boundary_before(pos) && has_boundary_after(end) {
                return None;
            }
            search_from = pos + 1;
        }
    }

    // Other patterns to block (non-word-boundary)
    let patterns = [
        " or ", " and ", " not ",
        "--", "/*", "*/",
        "';", "\";", "1=1", "1 = 1",
    ];

    for pattern in &patterns {
        if ascii_lower.contains(pattern) {
            return None;
        }
    }

    // Escape quotes for SQL safety
    let escaped = normalized.replace('\'', "''").replace('"', "\"\"");

    Some(escaped)
}

/// Sanitize an ID (stricter - only alphanumeric, hyphen, underscore)
fn sanitize_id(id: &str) -> Option<String> {
    // Empty IDs are invalid
    if id.is_empty() {
        return None;
    }

    // Check length limit
    if id.len() > 256 {
        return None;
    }

    // Only allow ASCII alphanumeric, hyphen, underscore
    if id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        Some(id.to_string())
    } else {
        None
    }
}

// ============================================================================
// OWASP Filter Bypass Pattern Tests
// ============================================================================

#[test]
fn test_blocks_sql_select() {
    assert!(sanitize_filter_value("'; SELECT * FROM users --").is_none());
    assert!(sanitize_filter_value("SELECT password FROM users").is_none());
    assert!(sanitize_filter_value("a' UNION SELECT * FROM secrets--").is_none());
}

#[test]
fn test_blocks_sql_modify() {
    assert!(sanitize_filter_value("'; INSERT INTO users VALUES('hacker', 'pass')--").is_none());
    assert!(sanitize_filter_value("'; UPDATE users SET admin=1 WHERE 1=1--").is_none());
    assert!(sanitize_filter_value("'; DELETE FROM users WHERE 1=1--").is_none());
    assert!(sanitize_filter_value("'; DROP TABLE users--").is_none());
}

#[test]
fn test_blocks_sql_union() {
    assert!(sanitize_filter_value("' UNION SELECT username, password FROM users--").is_none());
    assert!(sanitize_filter_value("1 UNION ALL SELECT NULL,NULL,NULL--").is_none());
}

#[test]
fn test_blocks_boolean_injection() {
    assert!(sanitize_filter_value("' OR '1'='1").is_none());
    assert!(sanitize_filter_value("' AND '1'='1").is_none());
    assert!(sanitize_filter_value("' OR 1=1--").is_none());
    assert!(sanitize_filter_value("admin' AND 1=1--").is_none());
}

#[test]
fn test_blocks_comment_injection() {
    assert!(sanitize_filter_value("admin'--").is_none());
    assert!(sanitize_filter_value("admin'/*").is_none());
    assert!(sanitize_filter_value("*/admin").is_none());
    assert!(sanitize_filter_value("admin'-- -").is_none());
}

#[test]
fn test_blocks_tautology() {
    assert!(sanitize_filter_value("' OR '1'='1' --").is_none());
    assert!(sanitize_filter_value("1=1").is_none());
    assert!(sanitize_filter_value("1 = 1").is_none());
}

// ============================================================================
// Case Insensitive Pattern Detection Tests
// ============================================================================

#[test]
fn test_case_insensitive_keywords() {
    assert!(sanitize_filter_value("SELECT").is_none());
    assert!(sanitize_filter_value("Select").is_none());
    assert!(sanitize_filter_value("sElEcT").is_none());
    assert!(sanitize_filter_value("INSERT").is_none());
    assert!(sanitize_filter_value("Insert").is_none());
    assert!(sanitize_filter_value("DELETE").is_none());
}

#[test]
fn test_case_insensitive_operators() {
    assert!(sanitize_filter_value("x OR y").is_none());
    assert!(sanitize_filter_value("x Or y").is_none());
    assert!(sanitize_filter_value("x or y").is_none());
    assert!(sanitize_filter_value("x AND y").is_none());
    assert!(sanitize_filter_value("x And y").is_none());
}

// ============================================================================
// Unicode Confusable Tests
// ============================================================================

#[test]
fn test_blocks_curly_quotes() {
    // Right single quotation mark (common in word processors)
    assert!(sanitize_filter_value("don\u{2019}t").is_none());
    // Left single quotation mark
    assert!(sanitize_filter_value("\u{2018}test").is_none());
    // Double curly quotes
    assert!(sanitize_filter_value("\u{201C}test\u{201D}").is_none());
}

#[test]
fn test_blocks_modifier_apostrophe() {
    // MODIFIER LETTER APOSTROPHE (looks like ' but different codepoint)
    assert!(sanitize_filter_value("test\u{02BC}value").is_none());
}

#[test]
fn test_blocks_fullwidth_quotes() {
    // Fullwidth apostrophe
    assert!(sanitize_filter_value("test\u{FF07}value").is_none());
    // Fullwidth quotation mark
    assert!(sanitize_filter_value("test\u{FF02}value").is_none());
}

#[test]
fn test_blocks_unicode_operators() {
    // Fullwidth equals
    assert!(sanitize_filter_value("x\u{FF1D}1").is_none());
    // Fullwidth less-than
    assert!(sanitize_filter_value("x\u{FF1C}1").is_none());
    // Fullwidth greater-than
    assert!(sanitize_filter_value("x\u{FF1E}1").is_none());
}

#[test]
fn test_blocks_unicode_hyphens() {
    // Various dash characters that could bypass -- comment detection
    assert!(sanitize_filter_value("test\u{2013}\u{2013}comment").is_none()); // EN DASH
    assert!(sanitize_filter_value("test\u{2014}\u{2014}comment").is_none()); // EM DASH
}

#[test]
fn test_greek_question_mark_normalizes() {
    // U+037E GREEK QUESTION MARK is canonically equivalent to U+003B SEMICOLON
    // NFC normalization converts it to ASCII semicolon, which is allowed
    // This is correct behavior - we only block confusables that DON'T normalize
    let result = sanitize_filter_value("test\u{037E}");
    assert!(result.is_some()); // Normalizes to regular semicolon, which is fine

    // However, fullwidth semicolon does NOT normalize and should be blocked
    assert!(sanitize_filter_value("test\u{FF1B}").is_none());
}

// ============================================================================
// Legitimate Value Tests
// ============================================================================

#[test]
fn test_allows_simple_values() {
    assert!(sanitize_filter_value("hello").is_some());
    assert!(sanitize_filter_value("world").is_some());
    assert!(sanitize_filter_value("test-value").is_some());
    assert!(sanitize_filter_value("test_value").is_some());
}

#[test]
fn test_allows_numbers() {
    assert!(sanitize_filter_value("123").is_some());
    assert!(sanitize_filter_value("test123").is_some());
    assert!(sanitize_filter_value("v2.0").is_some());
}

#[test]
fn test_allows_unicode_text() {
    // Unicode text without confusables should work
    assert!(sanitize_filter_value("café").is_some());
    assert!(sanitize_filter_value("日本語").is_some());
    assert!(sanitize_filter_value("Zürich").is_some());
}

#[test]
fn test_handles_partial_keywords() {
    // SQL keywords now use word-boundary matching
    // Partial matches should pass
    assert!(sanitize_filter_value("selection").is_some());
    assert!(sanitize_filter_value("inserts").is_some());
    assert!(sanitize_filter_value("deleted").is_some());

    // Full keyword with boundary still blocked
    assert!(sanitize_filter_value("'; SELECT * --").is_none());

    // " or " and " and " require space boundaries
    assert!(sanitize_filter_value("forest").is_some());  // contains "or" but not " or "
    assert!(sanitize_filter_value("android").is_some()); // contains "and" but not " and "
}

#[test]
fn test_escapes_straight_quotes() {
    let result = sanitize_filter_value("test's value").unwrap();
    assert_eq!(result, "test''s value");

    let result2 = sanitize_filter_value("test\"value").unwrap();
    assert_eq!(result2, "test\"\"value");
}

// ============================================================================
// ID Sanitization Tests
// ============================================================================

#[test]
fn test_id_allows_alphanumeric() {
    assert!(sanitize_id("simple").is_some());
    assert!(sanitize_id("test123").is_some());
    assert!(sanitize_id("123test").is_some());
}

#[test]
fn test_id_allows_hyphens_underscores() {
    assert!(sanitize_id("test-value").is_some());
    assert!(sanitize_id("test_value").is_some());
    assert!(sanitize_id("test-value_123").is_some());
}

#[test]
fn test_id_blocks_special_chars() {
    assert!(sanitize_id("test.value").is_none());
    assert!(sanitize_id("test value").is_none());
    assert!(sanitize_id("test@value").is_none());
    assert!(sanitize_id("test'value").is_none());
    assert!(sanitize_id("test\"value").is_none());
}

#[test]
fn test_id_blocks_unicode() {
    assert!(sanitize_id("café").is_none());
    assert!(sanitize_id("日本語").is_none());
    assert!(sanitize_id("test→value").is_none());
}

#[test]
fn test_id_blocks_injection_attempts() {
    assert!(sanitize_id("'; DROP TABLE --").is_none());
    assert!(sanitize_id("test OR 1=1").is_none());
}

#[test]
fn test_id_blocks_empty() {
    assert!(sanitize_id("").is_none());
}

#[test]
fn test_id_blocks_too_long() {
    let long_id = "a".repeat(257);
    assert!(sanitize_id(&long_id).is_none());

    let ok_id = "a".repeat(256);
    assert!(sanitize_id(&ok_id).is_some());
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_empty_value() {
    // Empty values should be allowed (though might be rejected by other validation)
    assert!(sanitize_filter_value("").is_some());
}

#[test]
fn test_whitespace_only() {
    assert!(sanitize_filter_value("   ").is_some());
    assert!(sanitize_filter_value("\t\n").is_some());
}

#[test]
fn test_unicode_normalization_nfc() {
    // Characters with combining marks should be normalized
    // é can be represented as e + combining accent or as single codepoint
    let decomposed = "e\u{0301}"; // e + combining acute accent
    let composed = "é";

    // Both should produce the same result after NFC normalization
    let result1 = sanitize_filter_value(decomposed);
    let result2 = sanitize_filter_value(composed);

    assert!(result1.is_some());
    assert!(result2.is_some());
}

#[test]
fn test_very_long_values() {
    // Very long values should still work (no arbitrary length limit on values)
    let long_value = "a".repeat(10000);
    assert!(sanitize_filter_value(&long_value).is_some());
}

// ============================================================================
// Real-World Attack Pattern Tests
// ============================================================================

#[test]
fn test_stacked_queries() {
    assert!(sanitize_filter_value("'; DELETE FROM users; --").is_none());
    assert!(sanitize_filter_value("1; DROP TABLE users").is_none());
}

#[test]
fn test_second_order_injection() {
    // Stored XSS/injection that gets executed later
    assert!(sanitize_filter_value("<script>alert(1)</script>").is_some()); // HTML, not SQL
    assert!(sanitize_filter_value("${7*7}").is_some()); // Template injection syntax, but no SQL
}

#[test]
fn test_encoded_attacks() {
    // URL-encoded attacks (should be decoded before reaching this function)
    // But if raw encoded values come through, they should be safe
    assert!(sanitize_filter_value("%27%20OR%20%271%27%3D%271").is_some()); // Not decoded
}

#[test]
fn test_null_byte_injection() {
    // Null byte attacks
    assert!(sanitize_filter_value("test\x00value").is_some()); // Null bytes pass through
}

#[test]
fn test_backslash_escape() {
    // Backslash might be used to escape quotes
    assert!(sanitize_filter_value(r"test\'value").is_some()); // Backslash-escaped quote
}
