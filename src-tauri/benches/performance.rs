//! Performance benchmarks for AssistSupport
//!
//! Run with: cargo bench
//!
//! These benchmarks establish baseline metrics for:
//! - Encryption throughput (MB/second)
//! - Key derivation (operations/second)
//! - Database operations (operations/second)
//! - FTS search latency (milliseconds)

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use std::time::Duration;

// Import from library
use assistsupport_lib::db::Database;
use assistsupport_lib::security::{Crypto, MasterKey};
use rusqlite::params;

fn setup_test_db() -> (Database, MasterKey, tempfile::TempDir) {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("bench.db");
    let key = MasterKey::generate();
    let db = Database::open(&db_path, &key).expect("Failed to open database");
    db.initialize().expect("Failed to initialize database");
    (db, key, temp_dir)
}

/// Benchmark encryption throughput
fn bench_encryption(c: &mut Criterion) {
    let key = MasterKey::generate();

    // Small data (1KB)
    let small_data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();

    // Medium data (64KB)
    let medium_data: Vec<u8> = (0..65536).map(|i| (i % 256) as u8).collect();

    // Large data (1MB)
    let large_data: Vec<u8> = (0..1_048_576).map(|i| (i % 256) as u8).collect();

    let mut group = c.benchmark_group("encryption");

    group.throughput(Throughput::Bytes(1024));
    group.bench_function("encrypt_1kb", |b| {
        b.iter(|| {
            Crypto::encrypt(key.as_bytes(), black_box(&small_data)).unwrap()
        })
    });

    group.throughput(Throughput::Bytes(65536));
    group.bench_function("encrypt_64kb", |b| {
        b.iter(|| {
            Crypto::encrypt(key.as_bytes(), black_box(&medium_data)).unwrap()
        })
    });

    group.throughput(Throughput::Bytes(1_048_576));
    group.bench_function("encrypt_1mb", |b| {
        b.iter(|| {
            Crypto::encrypt(key.as_bytes(), black_box(&large_data)).unwrap()
        })
    });

    group.finish();
}

/// Benchmark decryption throughput
fn bench_decryption(c: &mut Criterion) {
    let key = MasterKey::generate();

    let small_data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let medium_data: Vec<u8> = (0..65536).map(|i| (i % 256) as u8).collect();
    let large_data: Vec<u8> = (0..1_048_576).map(|i| (i % 256) as u8).collect();

    let encrypted_small = Crypto::encrypt(key.as_bytes(), &small_data).unwrap();
    let encrypted_medium = Crypto::encrypt(key.as_bytes(), &medium_data).unwrap();
    let encrypted_large = Crypto::encrypt(key.as_bytes(), &large_data).unwrap();

    let mut group = c.benchmark_group("decryption");

    group.throughput(Throughput::Bytes(1024));
    group.bench_function("decrypt_1kb", |b| {
        b.iter(|| {
            Crypto::decrypt(key.as_bytes(), black_box(&encrypted_small)).unwrap()
        })
    });

    group.throughput(Throughput::Bytes(65536));
    group.bench_function("decrypt_64kb", |b| {
        b.iter(|| {
            Crypto::decrypt(key.as_bytes(), black_box(&encrypted_medium)).unwrap()
        })
    });

    group.throughput(Throughput::Bytes(1_048_576));
    group.bench_function("decrypt_1mb", |b| {
        b.iter(|| {
            Crypto::decrypt(key.as_bytes(), black_box(&encrypted_large)).unwrap()
        })
    });

    group.finish();
}

/// Benchmark key derivation (intentionally slow for security)
fn bench_key_derivation(c: &mut Criterion) {
    let master_key = MasterKey::generate();
    let passphrase = "benchmark-passphrase-12345";

    let mut group = c.benchmark_group("key_derivation");
    group.sample_size(10); // Fewer samples since key derivation is slow by design
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("wrap_key_argon2", |b| {
        b.iter(|| {
            Crypto::wrap_key(black_box(&master_key), black_box(passphrase)).unwrap()
        })
    });

    let wrapped = Crypto::wrap_key(&master_key, passphrase).unwrap();

    group.bench_function("unwrap_key_argon2", |b| {
        b.iter(|| {
            Crypto::unwrap_key(black_box(&wrapped), black_box(passphrase)).unwrap()
        })
    });

    group.finish();
}

/// Benchmark FTS search performance
fn bench_fts_search(c: &mut Criterion) {
    let (db, _key, _temp_dir) = setup_test_db();

    // Insert test documents using raw SQL
    let conn = db.conn();

    // Insert test documents and chunks directly
    for i in 0..100 {
        let doc_id = format!("bench_doc_{}", i);
        conn.execute(
            "INSERT INTO kb_documents (id, file_path, file_hash, title, indexed_at, namespace_id)
             VALUES (?, ?, ?, ?, datetime('now'), 'default')",
            params![
                doc_id,
                format!("test_{}.md", i),
                format!("hash_{}", i),
                format!("Test Document {}", i)
            ],
        ).unwrap();

        // Insert chunks
        for j in 0..5 {
            let chunk_id = format!("{}_chunk_{}", doc_id, j);
            conn.execute(
                "INSERT INTO kb_chunks (id, document_id, chunk_index, heading_path, content, word_count)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    chunk_id,
                    doc_id,
                    j,
                    format!("Section {}", j),
                    format!("Chunk {} of document {}: This chunk contains information about IT support, troubleshooting, and technical documentation. The content covers various topics.", j, i),
                    20
                ],
            ).unwrap();
        }
    }

    let mut group = c.benchmark_group("fts_search");

    group.bench_function("search_simple", |b| {
        b.iter(|| {
            db.fts_search(black_box("troubleshooting"), 10).unwrap()
        })
    });

    group.bench_function("search_multi_word", |b| {
        b.iter(|| {
            db.fts_search(black_box("IT support technical"), 10).unwrap()
        })
    });

    group.bench_function("search_phrase", |b| {
        b.iter(|| {
            db.fts_search(black_box("\"technical documentation\""), 10).unwrap()
        })
    });

    group.finish();
}

/// Benchmark database operations
fn bench_db_operations(c: &mut Criterion) {
    let (db, _key, _temp_dir) = setup_test_db();

    let mut group = c.benchmark_group("db_operations");

    // Benchmark settings read/write
    group.bench_function("read_vector_consent", |b| {
        b.iter(|| {
            db.get_vector_consent().unwrap()
        })
    });

    group.bench_function("write_vector_consent", |b| {
        b.iter(|| {
            db.set_vector_consent(black_box(true), black_box(true)).unwrap()
        })
    });

    // Benchmark integrity check
    group.bench_function("check_integrity", |b| {
        b.iter(|| {
            db.check_integrity().unwrap()
        })
    });

    group.finish();
}

/// Benchmark master key generation
fn bench_key_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_generation");

    group.bench_function("generate_master_key", |b| {
        b.iter(|| {
            MasterKey::generate()
        })
    });

    group.finish();
}

/// Benchmark database open with encryption
fn bench_db_open(c: &mut Criterion) {
    let mut group = c.benchmark_group("db_open");
    group.sample_size(20); // Database open is relatively slow

    group.bench_function("open_and_init", |b| {
        b.iter_with_setup(
            || {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let key = MasterKey::generate();
                (temp_dir, key)
            },
            |(temp_dir, key)| {
                let db_path = temp_dir.path().join("bench.db");
                let db = Database::open(&db_path, &key).unwrap();
                db.initialize().unwrap();
                black_box(db)
            }
        )
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_encryption,
    bench_decryption,
    bench_key_generation,
    bench_fts_search,
    bench_db_operations,
    bench_db_open,
);

// Key derivation is slow by design, so we run it separately
criterion_group! {
    name = slow_benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(20));
    targets = bench_key_derivation
}

criterion_main!(benches, slow_benches);
