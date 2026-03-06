mod common;

use assistsupport_lib::diagnostics::{
    get_database_maintenance_cadence, get_database_stats, run_database_maintenance,
    run_database_maintenance_if_due,
};

#[test]
fn database_stats_include_wal_observability_fields() {
    let ctx = common::TestContext::new().expect("test context");

    let stats = get_database_stats(&ctx.db, &ctx.db_path).expect("database stats");

    assert!(
        stats.journal_mode.eq_ignore_ascii_case("wal"),
        "journal mode should be WAL for local-first read/write behavior"
    );
    assert!(stats.wal_checkpoint_busy >= 0);
    assert!(stats.wal_log_frames >= 0);
    assert!(stats.wal_checkpointed_frames >= 0);
}

#[test]
fn run_database_maintenance_records_optimize_and_checkpoint_timestamps() {
    let ctx = common::TestContext::new().expect("test context");

    let result = run_database_maintenance(&ctx.db);
    assert!(result.success, "database maintenance should succeed in test context");

    let stats = get_database_stats(&ctx.db, &ctx.db_path).expect("database stats");
    assert!(
        stats.last_optimize.is_some(),
        "maintenance should record last_optimize timestamp"
    );
    assert!(
        stats.last_wal_checkpoint.is_some(),
        "maintenance should record last_wal_checkpoint timestamp"
    );
}

#[test]
fn maintenance_cadence_is_due_without_history() {
    let ctx = common::TestContext::new().expect("test context");
    let cadence = get_database_maintenance_cadence(&ctx.db);

    assert!(
        cadence.maintenance_due,
        "cadence should be due when no prior maintenance timestamps exist"
    );
    assert_eq!(cadence.maintenance_due_reason, "no_maintenance_history");
}

#[test]
fn maintenance_cadence_not_due_with_recent_optimize() {
    let ctx = common::TestContext::new().expect("test context");
    let now = chrono::Utc::now().to_rfc3339();
    ctx.db
        .conn()
        .execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('last_optimize', ?)",
            [&now],
        )
        .expect("set last_optimize");

    let cadence = get_database_maintenance_cadence(&ctx.db);
    assert!(
        !cadence.maintenance_due,
        "cadence should not be due immediately after optimize timestamp"
    );
    assert_eq!(cadence.maintenance_due_reason, "within_interval");
}

#[test]
fn run_if_due_skips_when_not_due_and_runs_when_due() {
    let ctx = common::TestContext::new().expect("test context");

    let now = chrono::Utc::now().to_rfc3339();
    ctx.db
        .conn()
        .execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('last_optimize', ?)",
            [&now],
        )
        .expect("set fresh optimize");

    let skipped = run_database_maintenance_if_due(&ctx.db);
    assert!(skipped.is_none(), "maintenance should skip when cadence is not due");

    let stale = (chrono::Utc::now() - chrono::Duration::hours(48)).to_rfc3339();
    ctx.db
        .conn()
        .execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('last_optimize', ?)",
            [&stale],
        )
        .expect("set stale optimize");

    let triggered = run_database_maintenance_if_due(&ctx.db);
    assert!(triggered.is_some(), "maintenance should run when cadence is due");
}
