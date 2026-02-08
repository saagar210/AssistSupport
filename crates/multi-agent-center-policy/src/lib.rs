#![forbid(unsafe_code)]

use anyhow::Result;
use memory_kernel_core::{ContextItem, ContextPackage};
use multi_agent_center_domain::{hash_json, ContextPackageEnvelope, EffectivePermissions};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PrunedReference {
    pub package_slot: usize,
    pub memory_version_id: String,
    pub memory_id: String,
    pub version: u32,
    pub record_type: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PermissionPruneResult {
    pub packages: Vec<ContextPackageEnvelope>,
    pub pruned_references: Vec<PrunedReference>,
}

/// Apply agent permissions to context packages and return pruned artifacts.
///
/// # Errors
/// Returns an error if package hashing/serialization fails while rebuilding packages.
pub fn apply_context_permissions(
    packages: &[ContextPackageEnvelope],
    permissions: &EffectivePermissions,
) -> Result<PermissionPruneResult> {
    let mut out_packages = Vec::with_capacity(packages.len());
    let mut pruned = Vec::new();

    let type_filter_enabled = !permissions.allowed_record_types.is_empty();
    let mut total_selected = 0_u32;

    for package in packages {
        let mut selected_items: Vec<ContextItem> = Vec::new();
        for item in &package.context_package.selected_items {
            if type_filter_enabled && !permissions.allowed_record_types.contains(&item.record_type)
            {
                pruned.push(PrunedReference {
                    package_slot: package.package_slot,
                    memory_version_id: item.memory_version_id.to_string(),
                    memory_id: item.memory_id.to_string(),
                    version: item.version,
                    record_type: item.record_type.as_str().to_string(),
                    reason: "record_type_not_allowed".to_string(),
                });
                continue;
            }

            if let Some(limit) = permissions.max_context_items {
                if total_selected >= limit {
                    pruned.push(PrunedReference {
                        package_slot: package.package_slot,
                        memory_version_id: item.memory_version_id.to_string(),
                        memory_id: item.memory_id.to_string(),
                        version: item.version,
                        record_type: item.record_type.as_str().to_string(),
                        reason: "max_context_items_exceeded".to_string(),
                    });
                    continue;
                }
            }

            selected_items.push(item.clone());
            total_selected += 1;
        }

        let mut excluded_items: Vec<ContextItem> = Vec::new();
        for item in &package.context_package.excluded_items {
            if type_filter_enabled && !permissions.allowed_record_types.contains(&item.record_type)
            {
                pruned.push(PrunedReference {
                    package_slot: package.package_slot,
                    memory_version_id: item.memory_version_id.to_string(),
                    memory_id: item.memory_id.to_string(),
                    version: item.version,
                    record_type: item.record_type.as_str().to_string(),
                    reason: "excluded_record_type_not_allowed".to_string(),
                });
                continue;
            }
            excluded_items.push(item.clone());
        }

        let context_package = ContextPackage {
            context_package_id: package.context_package.context_package_id.clone(),
            generated_at: package.context_package.generated_at,
            query: package.context_package.query.clone(),
            determinism: package.context_package.determinism.clone(),
            answer: package.context_package.answer.clone(),
            selected_items,
            excluded_items,
            ordering_trace: package.context_package.ordering_trace.clone(),
        };

        let package_hash = hash_json(&serde_json::to_value(&context_package)?)?;

        out_packages.push(ContextPackageEnvelope {
            package_slot: package.package_slot,
            source: package.source.clone(),
            context_package,
            package_hash,
        });
    }

    Ok(PermissionPruneResult {
        packages: out_packages,
        pruned_references: pruned,
    })
}

#[cfg(test)]
mod tests {
    use super::apply_context_permissions;
    use memory_kernel_core::{
        Answer, AnswerResult, Authority, ContextItem, ContextPackage, DeterminismMetadata,
        MemoryId, MemoryVersionId, QueryRequest, RecordType, TruthStatus, Why,
    };
    use multi_agent_center_domain::{ContextPackageEnvelope, EffectivePermissions};

    fn fixture_package() -> ContextPackageEnvelope {
        let now = time::OffsetDateTime::now_utc();
        let selected_items = vec![
            ContextItem {
                rank: 1,
                memory_version_id: MemoryVersionId::new(),
                memory_id: MemoryId::new(),
                record_type: RecordType::Constraint,
                version: 1,
                truth_status: TruthStatus::Asserted,
                confidence: Some(0.8),
                authority: Authority::Authoritative,
                why: Why {
                    included: true,
                    reasons: vec!["fixture".to_string()],
                    rule_scores: None,
                },
            },
            ContextItem {
                rank: 2,
                memory_version_id: MemoryVersionId::new(),
                memory_id: MemoryId::new(),
                record_type: RecordType::Decision,
                version: 1,
                truth_status: TruthStatus::Observed,
                confidence: Some(0.6),
                authority: Authority::Derived,
                why: Why {
                    included: true,
                    reasons: vec!["fixture".to_string()],
                    rule_scores: None,
                },
            },
            ContextItem {
                rank: 3,
                memory_version_id: MemoryVersionId::new(),
                memory_id: MemoryId::new(),
                record_type: RecordType::Event,
                version: 1,
                truth_status: TruthStatus::Observed,
                confidence: Some(0.55),
                authority: Authority::Derived,
                why: Why {
                    included: true,
                    reasons: vec!["fixture".to_string()],
                    rule_scores: None,
                },
            },
        ];

        let context_package = ContextPackage {
            context_package_id: "fixture-package".to_string(),
            generated_at: now,
            query: QueryRequest {
                text: "fixture".to_string(),
                actor: "actor".to_string(),
                action: "action".to_string(),
                resource: "resource".to_string(),
                as_of: now,
            },
            determinism: DeterminismMetadata {
                ruleset_version: "memory_kernel.v1".to_string(),
                snapshot_id: "snapshot".to_string(),
                tie_breakers: vec!["fixture".to_string()],
            },
            answer: Answer {
                result: AnswerResult::Allow,
                why: "fixture".to_string(),
            },
            selected_items,
            excluded_items: Vec::new(),
            ordering_trace: vec!["fixture".to_string()],
        };

        ContextPackageEnvelope {
            package_slot: 0,
            source: "test".to_string(),
            package_hash: "hash".to_string(),
            context_package,
        }
    }

    #[test]
    fn prunes_disallowed_types_and_excess_items() {
        let packages = vec![fixture_package()];
        let permissions = EffectivePermissions {
            allowed_record_types: vec![RecordType::Constraint, RecordType::Event],
            allowed_tools: Vec::new(),
            max_context_items: Some(1),
            can_propose_memory_writes: false,
            fail_on_permission_prune: false,
        };

        let result = apply_context_permissions(&packages, &permissions);
        assert!(result.is_ok());
        let result = result.unwrap_or_else(|_| unreachable!());

        assert_eq!(result.packages.len(), 1);
        assert_eq!(result.packages[0].context_package.selected_items.len(), 1);
        assert_eq!(
            result.packages[0].context_package.selected_items[0].record_type,
            RecordType::Constraint
        );

        let reasons: Vec<String> = result
            .pruned_references
            .iter()
            .map(|item| item.reason.clone())
            .collect();
        assert!(reasons.contains(&"record_type_not_allowed".to_string()));
        assert!(reasons.contains(&"max_context_items_exceeded".to_string()));
    }

    #[test]
    fn empty_allowed_types_allows_all_records() {
        let packages = vec![fixture_package()];
        let permissions = EffectivePermissions {
            allowed_record_types: Vec::new(),
            allowed_tools: Vec::new(),
            max_context_items: None,
            can_propose_memory_writes: false,
            fail_on_permission_prune: false,
        };

        let result = apply_context_permissions(&packages, &permissions);
        assert!(result.is_ok());
        let result = result.unwrap_or_else(|_| unreachable!());

        assert_eq!(result.packages[0].context_package.selected_items.len(), 3);
        assert!(result.pruned_references.is_empty());
        assert!(!result.packages[0].package_hash.is_empty());
    }
}
