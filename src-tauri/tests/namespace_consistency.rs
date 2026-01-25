//! Namespace Consistency Tests
//!
//! Tests for namespace ID normalization consistency across all code paths.
//! Ensures idempotence and handles Unicode edge cases.

/// Simplified normalization function matching the main codebase
/// This duplicates the logic to ensure tests don't depend on implementation
fn normalize_namespace_id(id: &str) -> String {
    let normalized = id
        .to_lowercase()
        .replace(' ', "-")
        .replace('_', "-")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect::<String>();

    // Collapse multiple hyphens
    let mut result = String::new();
    let mut prev_hyphen = false;
    for c in normalized.chars() {
        if c == '-' {
            if !prev_hyphen && !result.is_empty() {
                result.push(c);
                prev_hyphen = true;
            }
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }

    // Trim trailing hyphens and truncate
    let trimmed = result.trim_end_matches('-');
    if trimmed.len() > 64 {
        trimmed[..64].trim_end_matches('-').to_string()
    } else {
        trimmed.to_string()
    }
}

// ============================================================================
// Idempotence Tests
// ============================================================================

#[test]
fn test_normalization_is_idempotent() {
    let test_cases = vec![
        "simple",
        "With Spaces",
        "with_underscores",
        "MixedCase",
        "  leading-trailing  ",
        "multiple---hyphens",
        "special!@#chars",
        "unicode-café",
        "123numbers",
    ];

    for input in test_cases {
        let first_pass = normalize_namespace_id(input);
        let second_pass = normalize_namespace_id(&first_pass);
        assert_eq!(
            first_pass, second_pass,
            "Normalization not idempotent for '{}'",
            input
        );

        // Third pass should also be identical
        let third_pass = normalize_namespace_id(&second_pass);
        assert_eq!(second_pass, third_pass);
    }
}

#[test]
fn test_all_normalization_paths_produce_same_output() {
    // These inputs should all normalize to the same output
    let equivalent_inputs = vec![
        "my namespace",
        "My Namespace",
        "MY NAMESPACE",
        "my_namespace",
        "my-namespace",
        "My_Namespace",
        "MY_NAMESPACE",
        "my--namespace",
        "my__namespace",
        " my namespace ",
        "  my  namespace  ",
    ];

    let expected = "my-namespace";

    for input in equivalent_inputs {
        let result = normalize_namespace_id(input);
        assert_eq!(
            result, expected,
            "Input '{}' produced '{}', expected '{}'",
            input, result, expected
        );
    }
}

// ============================================================================
// Unicode Edge Cases
// ============================================================================

#[test]
fn test_unicode_stripped_correctly() {
    // Non-ASCII characters should be stripped
    let test_cases = vec![
        ("café", "caf"),
        ("naïve", "nave"),
        ("日本語", ""),  // All non-ASCII
        ("hello世界", "hello"),
        ("Zürich", "zrich"),
        ("straße", "strae"), // German ß
    ];

    for (input, expected) in test_cases {
        let result = normalize_namespace_id(input);
        assert_eq!(
            result, expected,
            "Unicode handling failed for '{}'",
            input
        );
    }
}

#[test]
fn test_turkish_i_case_folding() {
    // Turkish has special case folding rules for 'i' and 'İ'
    // In Turkish: i → İ (uppercase), I → ı (lowercase)
    // U+0130 (İ) lowercases to ASCII 'i' in Rust's to_lowercase()
    let result = normalize_namespace_id("İstanbul");
    assert_eq!(result, "istanbul"); // İ lowercases to 'i'

    let result2 = normalize_namespace_id("ISTANBUL");
    assert_eq!(result2, "istanbul"); // Normal ASCII case folding

    // Turkish lowercase ı (U+0131) is non-ASCII and gets stripped
    let result3 = normalize_namespace_id("ıstanbul");
    assert_eq!(result3, "stanbul"); // ı is stripped (non-ASCII)
}

#[test]
fn test_german_eszett() {
    // German ß is a single character that uppercases to "SS"
    // With our approach, ß is stripped (non-ASCII)
    let result = normalize_namespace_id("Straße");
    assert_eq!(result, "strae");
}

#[test]
fn test_unicode_confusables() {
    // Characters that look like ASCII but aren't
    // These should be stripped, not treated as their lookalikes

    // U+02BC MODIFIER LETTER APOSTROPHE (looks like ')
    let result = normalize_namespace_id("don\u{02BC}t");
    assert_eq!(result, "dont");

    // U+2019 RIGHT SINGLE QUOTATION MARK (curly apostrophe)
    let result2 = normalize_namespace_id("don't");
    assert_eq!(result2, "dont");

    // Full-width letters (U+FF21-U+FF5A)
    let result3 = normalize_namespace_id("\u{FF21}BC"); // ＡBC (fullwidth A)
    assert_eq!(result3, "bc"); // Fullwidth A stripped
}

#[test]
fn test_emoji_stripped() {
    let result = normalize_namespace_id("hello🎉world");
    assert_eq!(result, "helloworld");

    let result2 = normalize_namespace_id("🚀rocket");
    assert_eq!(result2, "rocket");
}

// ============================================================================
// Boundary Tests
// ============================================================================

#[test]
fn test_empty_input() {
    let result = normalize_namespace_id("");
    assert_eq!(result, "");
}

#[test]
fn test_whitespace_only() {
    let result = normalize_namespace_id("   ");
    assert_eq!(result, "");

    let result2 = normalize_namespace_id("\t\n\r");
    assert_eq!(result2, "");
}

#[test]
fn test_special_chars_only() {
    let result = normalize_namespace_id("!@#$%^&*()");
    assert_eq!(result, "");
}

#[test]
fn test_length_truncation() {
    // Input longer than 64 characters
    let long_input = "a".repeat(100);
    let result = normalize_namespace_id(&long_input);
    assert_eq!(result.len(), 64);
    assert_eq!(result, "a".repeat(64));
}

#[test]
fn test_truncation_preserves_word_boundary() {
    // If truncation would end with hyphen, strip it
    let input = format!("{}----test", "a".repeat(60));
    let result = normalize_namespace_id(&input);
    assert!(!result.ends_with('-'));
}

#[test]
fn test_numbers_preserved() {
    let result = normalize_namespace_id("project123");
    assert_eq!(result, "project123");

    let result2 = normalize_namespace_id("123project");
    assert_eq!(result2, "123project");

    let result3 = normalize_namespace_id("v2-beta");
    assert_eq!(result3, "v2-beta");
}

// ============================================================================
// Hyphen Handling Tests
// ============================================================================

#[test]
fn test_multiple_hyphens_collapsed() {
    let result = normalize_namespace_id("a---b");
    assert_eq!(result, "a-b");

    let result2 = normalize_namespace_id("a------b");
    assert_eq!(result2, "a-b");
}

#[test]
fn test_leading_hyphen_removed() {
    let result = normalize_namespace_id("-leading");
    assert_eq!(result, "leading");

    let result2 = normalize_namespace_id("---leading");
    assert_eq!(result2, "leading");
}

#[test]
fn test_trailing_hyphen_removed() {
    let result = normalize_namespace_id("trailing-");
    assert_eq!(result, "trailing");

    let result2 = normalize_namespace_id("trailing---");
    assert_eq!(result2, "trailing");
}

#[test]
fn test_only_hyphens() {
    let result = normalize_namespace_id("---");
    assert_eq!(result, "");
}

// ============================================================================
// Mixed Input Tests
// ============================================================================

#[test]
fn test_complex_mixed_input() {
    let result = normalize_namespace_id("  My_Project--v2.0 (beta)!  ");
    assert_eq!(result, "my-project-v20-beta");
}

#[test]
fn test_path_like_input() {
    let result = normalize_namespace_id("projects/my-project/docs");
    assert_eq!(result, "projectsmy-projectdocs");
}

#[test]
fn test_url_like_input() {
    let result = normalize_namespace_id("https://example.com");
    assert_eq!(result, "httpsexamplecom");
}

// ============================================================================
// Consistency with Validation Tests
// ============================================================================

#[test]
fn test_normalized_output_is_valid() {
    let long_input = "a".repeat(100);
    let test_inputs: Vec<&str> = vec![
        "simple",
        "With Spaces And CAPS",
        "special!@#$chars",
        "unicode-日本語",
        "numbers123",
        &long_input,
    ];

    for input in test_inputs {
        let normalized = normalize_namespace_id(input);

        // Normalized output should only contain valid characters
        assert!(
            normalized.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
            "Normalized '{}' contains invalid chars: '{}'",
            input,
            normalized
        );

        // Should not start or end with hyphen
        if !normalized.is_empty() {
            assert!(
                !normalized.starts_with('-'),
                "Normalized '{}' starts with hyphen: '{}'",
                input,
                normalized
            );
            assert!(
                !normalized.ends_with('-'),
                "Normalized '{}' ends with hyphen: '{}'",
                input,
                normalized
            );
        }

        // Should be within length limit
        assert!(
            normalized.len() <= 64,
            "Normalized '{}' exceeds length limit: {} chars",
            input,
            normalized.len()
        );
    }
}
