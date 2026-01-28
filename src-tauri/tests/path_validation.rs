//! Path validation integration tests
//!
//! Tests for home directory enforcement, sensitive directory blocking,
//! path traversal prevention, and auto-create functionality.

mod common;

use assistsupport_lib::validation::{validate_within_home, ValidationError};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ============================================================================
// Home Directory Enforcement Tests
// ============================================================================

#[test]
fn test_home_directory_valid_paths() {
    let home = dirs::home_dir().expect("Home directory should exist");

    // Documents should be valid (if exists)
    if home.join("Documents").exists() {
        let result = validate_within_home(&home.join("Documents"));
        assert!(result.is_ok(), "Documents should be allowed");
    }

    // Desktop should be valid (if exists)
    if home.join("Desktop").exists() {
        let result = validate_within_home(&home.join("Desktop"));
        assert!(result.is_ok(), "Desktop should be allowed");
    }

    // Downloads should be valid (if exists)
    if home.join("Downloads").exists() {
        let result = validate_within_home(&home.join("Downloads"));
        assert!(result.is_ok(), "Downloads should be allowed");
    }
}

#[test]
fn test_system_paths_blocked() {
    // /etc should be blocked
    let result = validate_within_home(Path::new("/etc"));
    assert!(
        matches!(
            result,
            Err(ValidationError::PathTraversal) | Err(ValidationError::PathNotFound(_))
        ),
        "/etc should be blocked"
    );

    // /var/log should be blocked
    let result = validate_within_home(Path::new("/var/log"));
    assert!(
        matches!(
            result,
            Err(ValidationError::PathTraversal) | Err(ValidationError::PathNotFound(_))
        ),
        "/var/log should be blocked"
    );

    // /usr/local should be blocked
    let result = validate_within_home(Path::new("/usr/local"));
    assert!(
        matches!(
            result,
            Err(ValidationError::PathTraversal) | Err(ValidationError::PathNotFound(_))
        ),
        "/usr/local should be blocked"
    );

    // /tmp should be blocked (not under home)
    let result = validate_within_home(Path::new("/tmp"));
    assert!(
        matches!(
            result,
            Err(ValidationError::PathTraversal) | Err(ValidationError::PathNotFound(_))
        ),
        "/tmp should be blocked"
    );
}

// ============================================================================
// Sensitive Directory Blocking Tests
// ============================================================================

#[test]
fn test_ssh_directory_blocked() {
    let home = dirs::home_dir().expect("Home directory should exist");
    let ssh_path = home.join(".ssh");

    if ssh_path.exists() {
        let result = validate_within_home(&ssh_path);
        assert!(
            matches!(result, Err(ValidationError::InvalidFormat(_))),
            ".ssh should be blocked: got {:?}",
            result
        );

        // Nested paths should also be blocked
        let nested = ssh_path.join("id_rsa");
        if nested.exists() {
            let result = validate_within_home(&nested);
            assert!(
                matches!(result, Err(ValidationError::InvalidFormat(_))),
                ".ssh/id_rsa should be blocked"
            );
        }
    }
}

#[test]
fn test_aws_directory_blocked() {
    let home = dirs::home_dir().expect("Home directory should exist");
    let aws_path = home.join(".aws");

    if aws_path.exists() {
        let result = validate_within_home(&aws_path);
        assert!(
            matches!(result, Err(ValidationError::InvalidFormat(_))),
            ".aws should be blocked"
        );

        // Credentials file should be blocked
        let creds = aws_path.join("credentials");
        if creds.exists() {
            let result = validate_within_home(&creds);
            assert!(
                matches!(result, Err(ValidationError::InvalidFormat(_))),
                ".aws/credentials should be blocked"
            );
        }
    }
}

#[test]
fn test_gnupg_directory_blocked() {
    let home = dirs::home_dir().expect("Home directory should exist");
    let gnupg_path = home.join(".gnupg");

    if gnupg_path.exists() {
        let result = validate_within_home(&gnupg_path);
        assert!(
            matches!(result, Err(ValidationError::InvalidFormat(_))),
            ".gnupg should be blocked"
        );
    }
}

#[test]
fn test_config_directory_blocked() {
    let home = dirs::home_dir().expect("Home directory should exist");
    let config_path = home.join(".config");

    if config_path.exists() {
        let result = validate_within_home(&config_path);
        assert!(
            matches!(result, Err(ValidationError::InvalidFormat(_))),
            ".config should be blocked"
        );
    }
}

#[test]
fn test_keychains_directory_blocked() {
    let home = dirs::home_dir().expect("Home directory should exist");
    let keychains_path = home.join("Library").join("Keychains");

    if keychains_path.exists() {
        let result = validate_within_home(&keychains_path);
        assert!(
            matches!(result, Err(ValidationError::InvalidFormat(_))),
            "Library/Keychains should be blocked"
        );
    }
}

// ============================================================================
// Path Traversal Prevention Tests
// ============================================================================

#[test]
fn test_path_traversal_with_dotdot() {
    let home = dirs::home_dir().expect("Home directory should exist");

    // Try to escape via Documents/../../../etc
    let traversal_path = home
        .join("Documents")
        .join("..")
        .join("..")
        .join("..")
        .join("etc");
    let result = validate_within_home(&traversal_path);
    assert!(
        matches!(
            result,
            Err(ValidationError::PathTraversal) | Err(ValidationError::PathNotFound(_))
        ),
        "Path traversal should be blocked"
    );
}

#[test]
fn test_symlink_traversal_blocked() {
    // Create a temp dir and a symlink to /etc
    let temp = TempDir::new().expect("Failed to create temp dir");
    let link_path = temp.path().join("etc_link");

    // Create symlink (may fail on Windows without admin, so skip if it fails)
    #[cfg(unix)]
    {
        if std::os::unix::fs::symlink("/etc", &link_path).is_ok() {
            let result = validate_within_home(&link_path);
            // Should either fail as path traversal or not found (if /etc symlink doesn't resolve within home)
            assert!(
                result.is_err(),
                "Symlink to /etc should be blocked or not resolve to home"
            );
        }
    }
}

// ============================================================================
// Auto-create Directory Tests
// ============================================================================

#[test]
fn test_auto_create_new_directory() {
    let home = dirs::home_dir().expect("Home directory should exist");

    // Create a unique directory name under Documents (if it exists)
    if home.join("Documents").exists() {
        let unique_name = format!("test_kb_{}", uuid::Uuid::new_v4());
        let new_path = home.join("Documents").join(&unique_name);

        // Ensure it doesn't exist
        assert!(!new_path.exists(), "Test directory should not exist yet");

        // Validate should create it
        let result = validate_within_home(&new_path);
        assert!(result.is_ok(), "Should auto-create directory: {:?}", result);

        // Verify it was created
        assert!(new_path.exists(), "Directory should have been created");
        assert!(new_path.is_dir(), "Should be a directory");

        // Cleanup
        let _ = fs::remove_dir(&new_path);
    }
}

#[test]
fn test_auto_create_nested_fails_without_parent() {
    let home = dirs::home_dir().expect("Home directory should exist");

    // Try to create a deeply nested path where parent doesn't exist
    let unique_name = format!("nonexistent_parent_{}/nested/deep", uuid::Uuid::new_v4());
    let nested_path = home.join(&unique_name);

    // This should fail because parent doesn't exist
    let result = validate_within_home(&nested_path);
    assert!(
        result.is_err(),
        "Should fail when parent directory doesn't exist"
    );
}

#[test]
fn test_auto_create_in_sensitive_location_blocked() {
    let home = dirs::home_dir().expect("Home directory should exist");

    // Try to create a new directory inside .ssh
    if home.join(".ssh").exists() {
        let unique_name = format!("test_dir_{}", uuid::Uuid::new_v4());
        let sensitive_path = home.join(".ssh").join(&unique_name);

        // Should be blocked even though parent exists
        let result = validate_within_home(&sensitive_path);
        assert!(
            matches!(result, Err(ValidationError::InvalidFormat(_))),
            "Should block creation in sensitive directory"
        );
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_home_directory_itself() {
    let home = dirs::home_dir().expect("Home directory should exist");

    // Home directory itself should be valid
    let result = validate_within_home(&home);
    assert!(result.is_ok(), "Home directory itself should be valid");
}

#[test]
fn test_path_with_spaces() {
    let _ctx = common::TestContext::new().expect("Failed to create test context");

    // Note: This test uses temp dir which may not be in home, so we use a different approach
    // We'll just verify the function handles spaces in paths correctly
    let home = dirs::home_dir().expect("Home directory should exist");

    if home.join("Documents").exists() {
        let unique_name = format!("test folder with spaces {}", uuid::Uuid::new_v4());
        let spaced_path = home.join("Documents").join(&unique_name);

        let result = validate_within_home(&spaced_path);
        assert!(result.is_ok(), "Paths with spaces should be allowed");

        // Cleanup
        if spaced_path.exists() {
            let _ = fs::remove_dir(&spaced_path);
        }
    }
}

#[test]
fn test_case_sensitivity() {
    // macOS is typically case-insensitive but preserving
    let home = dirs::home_dir().expect("Home directory should exist");

    if home.join("Documents").exists() {
        // Try both cases
        let lower = home.join("documents");
        let upper = home.join("DOCUMENTS");

        // On case-insensitive systems, these should all work
        let result_lower = validate_within_home(&lower);
        let result_upper = validate_within_home(&upper);

        // At least one should succeed (depending on actual case)
        assert!(
            result_lower.is_ok() || result_upper.is_ok(),
            "At least one case variant should work"
        );
    }
}
