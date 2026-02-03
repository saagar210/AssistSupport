//! Integration tests for encrypted database read/write round-trips.

use assistsupport_lib::db::Database;
use assistsupport_lib::security::MasterKey;
use tempfile::TempDir;

#[test]
fn encrypted_database_roundtrip_persists_data_with_correct_key() {
    let temp_dir = TempDir::new().expect("temp dir");
    let db_path = temp_dir.path().join("roundtrip.db");
    let key = MasterKey::generate();
    let setting_key = "integration-test-key";
    let setting_value = "secret-value-123";

    {
        let db = Database::open(&db_path, &key).expect("open db");
        db.initialize().expect("initialize db");
        db.conn()
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)",
                rusqlite::params![setting_key, setting_value],
            )
            .expect("insert setting");
    }

    let reopened = Database::open(&db_path, &key).expect("reopen db with same key");
    let loaded: String = reopened
        .conn()
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![setting_key],
            |row| row.get(0),
        )
        .expect("load inserted setting");
    assert_eq!(loaded, setting_value);

    let raw = std::fs::read(&db_path).expect("read db bytes");
    let raw_text = String::from_utf8_lossy(&raw);
    assert!(
        !raw_text.contains(setting_value),
        "plaintext should not appear on disk"
    );
}

#[test]
fn encrypted_database_rejects_wrong_key() {
    let temp_dir = TempDir::new().expect("temp dir");
    let db_path = temp_dir.path().join("wrong-key.db");
    let key_a = MasterKey::generate();
    let key_b = MasterKey::generate();

    {
        let db = Database::open(&db_path, &key_a).expect("open db");
        db.initialize().expect("initialize db");
    }

    let reopened = Database::open(&db_path, &key_b);
    assert!(reopened.is_err(), "opening with wrong key must fail");
}
