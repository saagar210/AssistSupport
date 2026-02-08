use criterion::{criterion_group, criterion_main, Criterion};
use memory_kernel_core::{
    build_context_package, build_recall_context_package, default_recall_record_types, Authority,
    ConstraintEffect, ConstraintPayload, ConstraintScope, DecisionPayload, EventPayload, MemoryId,
    MemoryPayload, MemoryRecord, MemoryVersionId, PreferencePayload, QueryRequest, TruthStatus,
};
use time::OffsetDateTime;

fn mk_constraint(index: usize) -> MemoryRecord {
    let effect = if index % 2 == 0 { ConstraintEffect::Deny } else { ConstraintEffect::Allow };
    MemoryRecord {
        memory_version_id: MemoryVersionId::new(),
        memory_id: MemoryId::new(),
        version: 1,
        created_at: OffsetDateTime::UNIX_EPOCH,
        effective_at: OffsetDateTime::UNIX_EPOCH,
        truth_status: TruthStatus::Asserted,
        authority: Authority::Authoritative,
        confidence: Some(0.8),
        writer: "bench".to_string(),
        justification: "benchmark fixture".to_string(),
        provenance: memory_kernel_core::Provenance {
            source_uri: "file:///bench-policy.md".to_string(),
            source_hash: Some("sha256:abc123".to_string()),
            evidence: Vec::new(),
        },
        supersedes: Vec::new(),
        contradicts: Vec::new(),
        payload: MemoryPayload::Constraint(ConstraintPayload {
            scope: ConstraintScope {
                actor: "user".to_string(),
                action: "use".to_string(),
                resource: "usb_drive".to_string(),
            },
            effect,
            note: Some("policy benchmark fixture".to_string()),
        }),
    }
}

fn mk_summary(index: usize) -> MemoryRecord {
    let payload = match index % 4 {
        0 => MemoryPayload::Decision(DecisionPayload {
            summary: "Decision: USB media requires approval".to_string(),
        }),
        1 => MemoryPayload::Preference(PreferencePayload {
            summary: "Preference: avoid unencrypted USB devices".to_string(),
        }),
        2 => MemoryPayload::Event(EventPayload {
            summary: "Event: USB controls training completed".to_string(),
        }),
        _ => MemoryPayload::Outcome(memory_kernel_core::OutcomePayload {
            summary: "Outcome: USB compliance findings reduced".to_string(),
        }),
    };

    MemoryRecord {
        memory_version_id: MemoryVersionId::new(),
        memory_id: MemoryId::new(),
        version: 1,
        created_at: OffsetDateTime::UNIX_EPOCH,
        effective_at: OffsetDateTime::UNIX_EPOCH,
        truth_status: TruthStatus::Observed,
        authority: Authority::Authoritative,
        confidence: Some(0.85),
        writer: "bench".to_string(),
        justification: "benchmark fixture".to_string(),
        provenance: memory_kernel_core::Provenance {
            source_uri: "file:///bench-recall.md".to_string(),
            source_hash: Some("sha256:def456".to_string()),
            evidence: Vec::new(),
        },
        supersedes: Vec::new(),
        contradicts: Vec::new(),
        payload,
    }
}

fn bench_policy(c: &mut Criterion) {
    let records = (0..1_000).map(mk_constraint).collect::<Vec<_>>();
    let query = QueryRequest {
        text: "Am I allowed to use a USB drive?".to_string(),
        actor: "user".to_string(),
        action: "use".to_string(),
        resource: "usb_drive".to_string(),
        as_of: OffsetDateTime::UNIX_EPOCH,
    };

    c.bench_function("policy_context_package_1000_records", |b| {
        b.iter(|| {
            let package = build_context_package(&records, query.clone(), "bench_policy");
            if let Err(err) = package {
                panic!("policy benchmark context package failed: {err}");
            }
        });
    });
}

fn bench_recall(c: &mut Criterion) {
    let records = (0..1_000).map(mk_summary).collect::<Vec<_>>();
    let query = QueryRequest {
        text: "usb compliance outcome".to_string(),
        actor: "*".to_string(),
        action: "*".to_string(),
        resource: "*".to_string(),
        as_of: OffsetDateTime::UNIX_EPOCH,
    };
    let record_types = default_recall_record_types();

    c.bench_function("recall_context_package_1000_records", |b| {
        b.iter(|| {
            let package = build_recall_context_package(
                &records,
                query.clone(),
                "bench_recall",
                &record_types,
            );
            if let Err(err) = package {
                panic!("recall benchmark context package failed: {err}");
            }
        });
    });
}

criterion_group!(resolver_benches, bench_policy, bench_recall);
criterion_main!(resolver_benches);
