#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use memory_kernel_api::{AskRequest, MemoryKernelApi, RecallRequest};
use memory_kernel_core::{
    build_context_package, build_recall_context_package, default_recall_record_types, MemoryRecord,
    QueryRequest, RecordType,
};
use memory_kernel_outcome_core::{
    apply_as_of_decay, gate_memory, parse_rfc3339_utc, GateDecision as OutcomeGateDecision,
    MemoryKey, MemoryTrust, OutcomeRuleset, RetrievalMode, TrustStatus,
};
use memory_kernel_store_sqlite::SqliteStore as MemoryKernelSqliteStore;
use multi_agent_center_domain::{
    compute_step_request_hash, compute_step_result_hash, hash_json, now_utc, AgentDefinition,
    ContextPackageEnvelope, EffectivePermissions, EventRow, GateDecision, GateDecisionRecord,
    GateKind, NormalizedWorkflowEnvelope, ProposedMemoryWrite, RunId, RunRecord, RunStatus, StepId,
    StepRecord, StepRequest, StepResult, StepStatus, TraceEvent, TraceEventType,
    TrustGateAttachment,
};
use multi_agent_center_policy::{apply_context_permissions, PermissionPruneResult};
use multi_agent_center_provider::{
    HttpJsonProvider, MockProvider, ProviderAdapter, ProviderInvocation,
};
use multi_agent_center_trace_core::TraceStore;
use rusqlite::OptionalExtension;
use serde_json::{json, Map, Value};
use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub run_id: Option<RunId>,
    pub as_of: Option<time::OffsetDateTime>,
    pub replay_of_run_id: Option<RunId>,
    pub external_correlation_id: Option<String>,
    pub non_interactive: bool,
    pub cli_args_json: Value,
    pub engine_version: String,
    pub apply_proposed_writes: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            run_id: None,
            as_of: None,
            replay_of_run_id: None,
            external_correlation_id: None,
            non_interactive: false,
            cli_args_json: Value::Object(Map::default()),
            engine_version: "multi-agent-center.v0".to_string(),
            apply_proposed_writes: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunExecutionSummary {
    pub run_id: RunId,
    pub status: RunStatus,
    pub steps_total: usize,
    pub steps_succeeded: usize,
    pub steps_failed_or_rejected: usize,
}

#[derive(Debug, Clone)]
pub struct ReplayReport {
    pub run_id: RunId,
    pub events: usize,
    pub chain_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRef {
    pub memory_id: memory_kernel_core::MemoryId,
    pub version: u32,
    pub memory_version_id: memory_kernel_core::MemoryVersionId,
}

pub trait ContextPackageSource {
    #[allow(clippy::missing_errors_doc)]
    fn packages_for_step(
        &self,
        run_id: RunId,
        step: &multi_agent_center_domain::WorkflowStepDefinition,
        as_of: time::OffsetDateTime,
    ) -> Result<Vec<ContextPackageEnvelope>>;
}

pub trait TrustGateSource {
    #[allow(clippy::missing_errors_doc)]
    fn evaluate(
        &self,
        run_id: RunId,
        step_id: StepId,
        step_key: &str,
        as_of: time::OffsetDateTime,
        refs: &[ContextRef],
    ) -> Result<Vec<TrustGateAttachment>>;
}

pub trait HumanGateDecider {
    #[allow(clippy::missing_errors_doc)]
    fn decide(&self, request: &HumanGateRequest) -> Result<HumanGateResponse>;
}

pub trait ProposedWriteApplier {
    #[allow(clippy::missing_errors_doc)]
    fn apply(
        &self,
        run_id: RunId,
        step_id: StepId,
        write: &ProposedMemoryWrite,
    ) -> Result<WriteApplyResult>;
}

#[derive(Debug, Clone)]
pub struct HumanGateRequest {
    pub run_id: RunId,
    pub step_id: StepId,
    pub step_key: String,
    pub gate_name: String,
    pub required: bool,
    pub non_interactive: bool,
}

#[derive(Debug, Clone)]
pub struct HumanGateResponse {
    pub approved: bool,
    pub notes: Option<String>,
    pub decided_by: String,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WriteApplyResult {
    pub disposition: String,
    pub disposition_reason: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct StaticContextPackageSource {
    by_step: BTreeMap<String, Vec<ContextPackageEnvelope>>,
}

impl StaticContextPackageSource {
    #[must_use]
    pub fn with_step_packages(by_step: BTreeMap<String, Vec<ContextPackageEnvelope>>) -> Self {
        Self { by_step }
    }
}

impl ContextPackageSource for StaticContextPackageSource {
    fn packages_for_step(
        &self,
        _run_id: RunId,
        step: &multi_agent_center_domain::WorkflowStepDefinition,
        _as_of: time::OffsetDateTime,
    ) -> Result<Vec<ContextPackageEnvelope>> {
        Ok(self
            .by_step
            .get(&step.step_key)
            .cloned()
            .unwrap_or_default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMemoryKernelContextSource {
    pub records_by_step: BTreeMap<String, Vec<MemoryRecord>>,
}

impl ContextPackageSource for InMemoryMemoryKernelContextSource {
    fn packages_for_step(
        &self,
        run_id: RunId,
        step: &multi_agent_center_domain::WorkflowStepDefinition,
        as_of: time::OffsetDateTime,
    ) -> Result<Vec<ContextPackageEnvelope>> {
        let records = self
            .records_by_step
            .get(&step.step_key)
            .cloned()
            .unwrap_or_default();
        build_context_packages_from_records(
            &records,
            run_id,
            step,
            as_of,
            "memory_kernel.in_memory",
        )
    }
}

#[derive(Debug, Clone)]
pub struct SqliteMemoryKernelContextSource {
    db_path: PathBuf,
}

impl SqliteMemoryKernelContextSource {
    #[must_use]
    pub fn new(db_path: &Path) -> Self {
        Self {
            db_path: db_path.to_path_buf(),
        }
    }
}

impl ContextPackageSource for SqliteMemoryKernelContextSource {
    fn packages_for_step(
        &self,
        run_id: RunId,
        step: &multi_agent_center_domain::WorkflowStepDefinition,
        as_of: time::OffsetDateTime,
    ) -> Result<Vec<ContextPackageEnvelope>> {
        let mut store = MemoryKernelSqliteStore::open(&self.db_path)?;
        store.migrate()?;
        let mut records = store.list_records()?;
        records.retain(|record| record.effective_at <= as_of);
        build_context_packages_from_records(&records, run_id, step, as_of, "memory_kernel.sqlite")
    }
}

#[derive(Debug, Clone)]
pub struct ApiMemoryKernelContextSource {
    api: MemoryKernelApi,
}

impl ApiMemoryKernelContextSource {
    #[must_use]
    pub fn new(db_path: &Path) -> Self {
        Self {
            api: MemoryKernelApi::new(db_path.to_path_buf()),
        }
    }
}

impl ContextPackageSource for ApiMemoryKernelContextSource {
    fn packages_for_step(
        &self,
        run_id: RunId,
        step: &multi_agent_center_domain::WorkflowStepDefinition,
        as_of: time::OffsetDateTime,
    ) -> Result<Vec<ContextPackageEnvelope>> {
        let queries = resolve_step_context_queries(step, as_of)?;
        let mut envelopes = Vec::with_capacity(queries.len());

        for (package_slot, query) in queries.into_iter().enumerate() {
            let package = match query {
                StepContextQuery::Policy(request) => self.api.query_ask(AskRequest {
                    text: request.text,
                    actor: request.actor,
                    action: request.action,
                    resource: request.resource,
                    as_of: Some(request.as_of),
                })?,
                StepContextQuery::Recall { text, record_types } => {
                    self.api.query_recall(RecallRequest {
                        text,
                        record_types,
                        as_of: Some(as_of),
                    })?
                }
            };

            let package_json = serde_json::to_value(&package)?;
            let package_hash = hash_json(&package_json)?;
            envelopes.push(ContextPackageEnvelope {
                package_slot,
                source: "memory_kernel.api".to_string(),
                context_package: package,
                package_hash,
            });
        }

        Self::rewrite_snapshot_ids(run_id, step, envelopes)
    }
}

impl ApiMemoryKernelContextSource {
    fn rewrite_snapshot_ids(
        run_id: RunId,
        step: &multi_agent_center_domain::WorkflowStepDefinition,
        mut envelopes: Vec<ContextPackageEnvelope>,
    ) -> Result<Vec<ContextPackageEnvelope>> {
        // Keep a stable orchestrator-local hash chain regardless of upstream snapshot generation.
        for envelope in &mut envelopes {
            envelope.context_package.determinism.snapshot_id =
                format!("{run_id}:{}:{}", step.step_key, envelope.package_slot);
            let package_json = serde_json::to_value(&envelope.context_package)?;
            envelope.package_hash = hash_json(&package_json)?;
        }
        Ok(envelopes)
    }
}

#[derive(Debug, Clone, Default)]
pub struct AllowAllTrustGateSource;

impl TrustGateSource for AllowAllTrustGateSource {
    fn evaluate(
        &self,
        _run_id: RunId,
        _step_id: StepId,
        _step_key: &str,
        as_of: time::OffsetDateTime,
        refs: &[ContextRef],
    ) -> Result<Vec<TrustGateAttachment>> {
        Ok(refs
            .iter()
            .map(|item| TrustGateAttachment {
                memory_id: item.memory_id,
                version: item.version,
                memory_version_id: item.memory_version_id,
                include: true,
                trust_status: "active".to_string(),
                confidence_effective: 1.0,
                capped: false,
                reason_codes: vec!["included.no_trust_gating_configured".to_string()],
                ruleset_version: None,
                evaluated_at: as_of,
                source: "trust.none".to_string(),
            })
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct OutcomeMemoryTrustGateSource {
    db_path: PathBuf,
    mode: RetrievalMode,
}

impl OutcomeMemoryTrustGateSource {
    #[must_use]
    pub fn new(db_path: &Path, mode: RetrievalMode) -> Self {
        Self {
            db_path: db_path.to_path_buf(),
            mode,
        }
    }
}

impl TrustGateSource for OutcomeMemoryTrustGateSource {
    fn evaluate(
        &self,
        run_id: RunId,
        _step_id: StepId,
        step_key: &str,
        as_of: time::OffsetDateTime,
        refs: &[ContextRef],
    ) -> Result<Vec<TrustGateAttachment>> {
        let conn = rusqlite::Connection::open(&self.db_path).with_context(|| {
            format!(
                "failed to open OutcomeMemory sqlite at {}",
                self.db_path.display()
            )
        })?;

        let candidates: Vec<MemoryKey> = refs
            .iter()
            .map(|item| MemoryKey {
                memory_id: item.memory_id,
                version: item.version,
            })
            .collect();

        let rulesets = load_outcome_rulesets(&conn)?;
        let context_id = format!("{run_id}:{step_key}");
        let mut decisions: Vec<(OutcomeGateDecision, Option<u32>)> =
            Vec::with_capacity(candidates.len());
        for candidate in &candidates {
            let Some((trust, last_ruleset_version)) =
                get_memory_trust_and_ruleset(&conn, candidate.memory_id, candidate.version)?
            else {
                decisions.push((
                    OutcomeGateDecision {
                        memory_id: candidate.memory_id,
                        version: candidate.version,
                        include: false,
                        confidence_effective: 0.0,
                        trust_status: TrustStatus::Active,
                        capped: false,
                        reason_codes: vec!["excluded.no_trust_snapshot".to_string()],
                    },
                    None,
                ));
                continue;
            };

            let ruleset = rulesets
                .get(&last_ruleset_version)
                .ok_or_else(|| anyhow!("missing outcome ruleset {last_ruleset_version}"))?;
            let trust_with_decay = apply_as_of_decay(&trust, ruleset, as_of);
            decisions.push((
                gate_memory(&trust_with_decay, self.mode, Some(&context_id), ruleset),
                Some(last_ruleset_version),
            ));
        }

        let memory_version_by_key: BTreeMap<(String, u32), memory_kernel_core::MemoryVersionId> =
            refs.iter()
                .map(|item| {
                    (
                        (item.memory_id.to_string(), item.version),
                        item.memory_version_id,
                    )
                })
                .collect();

        decisions
            .into_iter()
            .map(|(decision, ruleset_version)| {
                let memory_version_id = *memory_version_by_key
                    .get(&(decision.memory_id.to_string(), decision.version))
                    .ok_or_else(|| {
                        anyhow!(
                            "missing memory_version_id for trust decision {}:{}",
                            decision.memory_id,
                            decision.version
                        )
                    })?;

                Ok(TrustGateAttachment {
                    memory_id: decision.memory_id,
                    version: decision.version,
                    memory_version_id,
                    include: decision.include,
                    trust_status: decision.trust_status.as_str().to_string(),
                    confidence_effective: decision.confidence_effective,
                    capped: decision.capped,
                    reason_codes: decision.reason_codes,
                    ruleset_version,
                    evaluated_at: as_of,
                    source: "outcome_memory.live".to_string(),
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct DefaultHumanGateDecider;

impl HumanGateDecider for DefaultHumanGateDecider {
    fn decide(&self, request: &HumanGateRequest) -> Result<HumanGateResponse> {
        if request.non_interactive {
            return Ok(HumanGateResponse {
                approved: false,
                notes: Some("non-interactive auto-reject".to_string()),
                decided_by: "system.non_interactive".to_string(),
                reason_codes: vec!["rejected.non_interactive".to_string()],
            });
        }

        Ok(HumanGateResponse {
            approved: true,
            notes: Some("default auto-approve".to_string()),
            decided_by: "system.default".to_string(),
            reason_codes: vec!["approved.default".to_string()],
        })
    }
}

#[derive(Debug, Clone)]
pub struct NoopProposedWriteApplier;

impl ProposedWriteApplier for NoopProposedWriteApplier {
    fn apply(
        &self,
        _run_id: RunId,
        _step_id: StepId,
        _write: &ProposedMemoryWrite,
    ) -> Result<WriteApplyResult> {
        Ok(WriteApplyResult {
            disposition: "not_applied".to_string(),
            disposition_reason: Some("apply_path_not_configured".to_string()),
        })
    }
}

pub struct Orchestrator<'a> {
    trace_store: &'a dyn TraceStore,
    context_source: &'a dyn ContextPackageSource,
    trust_source: &'a dyn TrustGateSource,
    human_gate: &'a dyn HumanGateDecider,
    write_applier: &'a dyn ProposedWriteApplier,
}

impl<'a> Orchestrator<'a> {
    #[must_use]
    pub fn new(
        trace_store: &'a dyn TraceStore,
        context_source: &'a dyn ContextPackageSource,
        trust_source: &'a dyn TrustGateSource,
        human_gate: &'a dyn HumanGateDecider,
        write_applier: &'a dyn ProposedWriteApplier,
    ) -> Self {
        Self {
            trace_store,
            context_source,
            trust_source,
            human_gate,
            write_applier,
        }
    }

    /// Execute a normalized workflow and persist full trace artifacts.
    ///
    /// # Errors
    /// Returns an error when trace persistence, context retrieval, gate evaluation,
    /// provider invocation routing, or workflow dependency resolution fails.
    #[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
    pub fn execute_workflow(
        &self,
        workflow: &NormalizedWorkflowEnvelope,
        config: RunConfig,
    ) -> Result<RunExecutionSummary> {
        self.trace_store.migrate()?;

        let run_id = config.run_id.unwrap_or_default();
        let as_of = config.as_of.unwrap_or_else(now_utc);
        let as_of_was_default = config.as_of.is_none();

        self.trace_store.upsert_workflow_snapshot(
            &workflow.normalized_hash,
            workflow.normalized_workflow.normalization_version,
            &workflow.source_format,
            &workflow.source_yaml_hash,
            &workflow.normalized_json,
        )?;

        let run = RunRecord {
            run_id,
            workflow_name: workflow.normalized_workflow.workflow_name.clone(),
            workflow_version: workflow.normalized_workflow.workflow_version.clone(),
            workflow_hash: workflow.normalized_hash.clone(),
            as_of,
            as_of_was_default,
            started_at: now_utc(),
            ended_at: None,
            status: RunStatus::Running,
            replay_of_run_id: config.replay_of_run_id,
            external_correlation_id: config.external_correlation_id.clone(),
            engine_version: config.engine_version.clone(),
            cli_args_json: config.cli_args_json.clone(),
            manifest_hash: None,
            manifest_signature: None,
            manifest_signature_status: "unsigned".to_string(),
        };
        self.trace_store.insert_run(&run)?;

        let run_manifest_payload = json!({
            "schema": "run_manifest.v1",
            "run_id": run_id.to_string(),
            "workflow_hash": workflow.normalized_hash,
            "source_yaml_hash": workflow.source_yaml_hash,
            "normalization_version": workflow.normalized_workflow.normalization_version,
            "workflow_name": workflow.normalized_workflow.workflow_name,
            "workflow_version": workflow.normalized_workflow.workflow_version,
            "as_of": format_rfc3339(as_of)?,
            "as_of_was_default": as_of_was_default,
            "replay_of_run_id": config.replay_of_run_id.map(|id| id.to_string()),
            "external_correlation_id": config.external_correlation_id,
            "engine_version": config.engine_version,
            "cli_args_json": config.cli_args_json,
        });
        let run_manifest_hash = hash_json(&run_manifest_payload)?;
        self.trace_store
            .update_run_manifest(run_id, &run_manifest_hash, None, "unsigned")?;

        let mut chain = EventChain::default();
        self.emit_event(
            run_id,
            None,
            TraceEventType::WorkflowNormalized,
            "system",
            "orchestrator",
            json!({
                "workflow_hash": workflow.normalized_hash,
                "source_yaml_hash": workflow.source_yaml_hash,
                "normalization_version": workflow.normalized_workflow.normalization_version,
            }),
            &mut chain,
        )?;
        self.emit_event(
            run_id,
            None,
            TraceEventType::RunStarted,
            "system",
            "orchestrator",
            json!({
                "as_of": format_rfc3339(as_of)?,
                "as_of_was_default": as_of_was_default,
            }),
            &mut chain,
        )?;

        let agents: BTreeMap<&str, &AgentDefinition> = workflow
            .normalized_workflow
            .agents
            .iter()
            .map(|agent| (agent.agent_name.as_str(), agent))
            .collect();

        let steps = &workflow.normalized_workflow.steps;
        let total_steps = steps.len();
        let mut step_by_key: BTreeMap<&str, usize> = BTreeMap::new();
        for (index, step) in steps.iter().enumerate() {
            step_by_key.insert(step.step_key.as_str(), index);
        }

        let mut statuses: Vec<StepStatus> = vec![StepStatus::Pending; total_steps];
        let step_ids: Vec<StepId> = (0..total_steps).map(|_| StepId::new()).collect();
        let mut inserted_steps = BTreeSet::new();

        loop {
            if statuses
                .iter()
                .all(|status| !matches!(status, StepStatus::Pending | StepStatus::Running))
            {
                break;
            }

            let mut ready: Vec<usize> = Vec::new();
            let mut blocked: Vec<usize> = Vec::new();

            for (idx, step) in steps.iter().enumerate() {
                if statuses[idx] != StepStatus::Pending {
                    continue;
                }

                let mut has_non_success_dependency = false;
                let mut all_done = true;
                for dep in &step.depends_on {
                    let dep_idx = step_by_key
                        .get(dep.as_str())
                        .ok_or_else(|| anyhow!("unknown dependency {dep}"))?;
                    match statuses[*dep_idx] {
                        StepStatus::Succeeded => {}
                        StepStatus::Pending | StepStatus::Running => {
                            all_done = false;
                        }
                        StepStatus::Failed | StepStatus::Rejected | StepStatus::Skipped => {
                            has_non_success_dependency = true;
                        }
                    }
                }

                if has_non_success_dependency {
                    blocked.push(idx);
                } else if all_done {
                    ready.push(idx);
                }
            }

            ready.sort_unstable();
            blocked.sort_unstable();

            for idx in blocked {
                statuses[idx] = StepStatus::Skipped;
                let step = &steps[idx];
                let step_id = step_ids[idx];
                let step_record = StepRecord {
                    step_id,
                    run_id,
                    step_index: idx,
                    step_key: step.step_key.clone(),
                    agent_name: step.agent_name.clone(),
                    status: StepStatus::Skipped,
                    started_at: Some(now_utc()),
                    ended_at: Some(now_utc()),
                    task_payload_json: step.task.clone(),
                    constraints_json: serde_json::to_value(&step.constraints)?,
                    permissions_json: Value::Object(Map::default()),
                    input_hash: "skipped".to_string(),
                    output_hash: Some("skipped".to_string()),
                    error_json: Some(json!({"reason": "dependency_not_satisfied"})),
                };
                if !inserted_steps.contains(&idx) {
                    self.trace_store.insert_step(&step_record)?;
                    inserted_steps.insert(idx);
                }
                self.emit_event(
                    run_id,
                    Some(step_id),
                    TraceEventType::StepFinished,
                    "system",
                    "scheduler",
                    json!({"step_key": step.step_key, "status": "skipped"}),
                    &mut chain,
                )?;
            }

            if ready.is_empty() {
                if statuses
                    .iter()
                    .all(|status| !matches!(status, StepStatus::Pending | StepStatus::Running))
                {
                    break;
                }
                return Err(anyhow!(
                    "no ready steps found while pending steps remain; check workflow dependencies"
                ));
            }

            for idx in ready {
                let step = &steps[idx];
                let step_id = step_ids[idx];
                statuses[idx] = StepStatus::Running;

                let agent = agents
                    .get(step.agent_name.as_str())
                    .ok_or_else(|| anyhow!("unknown agent {}", step.agent_name))?;
                let effective_permissions = EffectivePermissions::from(&agent.permissions);

                let packages = self
                    .context_source
                    .packages_for_step(run_id, step, as_of)
                    .with_context(|| {
                        format!(
                            "failed to obtain context packages for step {}",
                            step.step_key
                        )
                    })?;

                let PermissionPruneResult {
                    packages: permission_packages,
                    pruned_references,
                } = apply_context_permissions(&packages, &effective_permissions)?;

                let refs: Vec<ContextRef> = permission_packages
                    .iter()
                    .flat_map(|package| {
                        package
                            .context_package
                            .selected_items
                            .iter()
                            .map(|item| ContextRef {
                                memory_id: item.memory_id,
                                version: item.version,
                                memory_version_id: item.memory_version_id,
                            })
                    })
                    .collect();

                let trust_attachments =
                    self.trust_source
                        .evaluate(run_id, step_id, &step.step_key, as_of, &refs)?;

                let trust_map: BTreeMap<(String, u32), bool> = trust_attachments
                    .iter()
                    .map(|item| ((item.memory_id.to_string(), item.version), item.include))
                    .collect();
                let trust_included = trust_attachments.iter().filter(|item| item.include).count();
                let trust_excluded = trust_attachments.len().saturating_sub(trust_included);

                let gated_packages = apply_trust_filter(&permission_packages, &trust_map)?;

                let mut step_request = StepRequest {
                    run_id,
                    step_id,
                    step_key: step.step_key.clone(),
                    as_of,
                    agent: (*agent).clone(),
                    task_payload: step.task.clone(),
                    injected_context_packages: gated_packages,
                    trust_gate_attachments: trust_attachments,
                    effective_permissions: effective_permissions.clone(),
                    constraints: step.constraints.clone(),
                    input_hash: String::new(),
                };
                step_request.input_hash = compute_step_request_hash(&step_request)?;

                let step_record = StepRecord {
                    step_id,
                    run_id,
                    step_index: idx,
                    step_key: step.step_key.clone(),
                    agent_name: step.agent_name.clone(),
                    status: StepStatus::Running,
                    started_at: Some(now_utc()),
                    ended_at: None,
                    task_payload_json: step.task.clone(),
                    constraints_json: serde_json::to_value(&step.constraints)?,
                    permissions_json: serde_json::to_value(&effective_permissions)?,
                    input_hash: step_request.input_hash.clone(),
                    output_hash: None,
                    error_json: None,
                };
                if !inserted_steps.contains(&idx) {
                    self.trace_store.insert_step(&step_record)?;
                    inserted_steps.insert(idx);
                }

                for attachment in &step_request.trust_gate_attachments {
                    let trust_decision = if attachment.include {
                        GateDecision::Approved
                    } else {
                        GateDecision::Rejected
                    };
                    self.trace_store.append_gate_decision(
                        run_id,
                        step_id,
                        &GateDecisionRecord {
                            gate_kind: GateKind::Trust,
                            gate_name: "trust_gate".to_string(),
                            subject_type: "memory_ref".to_string(),
                            memory_id: Some(attachment.memory_id),
                            version: Some(attachment.version),
                            memory_version_id: Some(attachment.memory_version_id),
                            decision: trust_decision,
                            reason_codes: attachment.reason_codes.clone(),
                            notes: Some(format!(
                                "status={} confidence_effective={:.6} capped={}",
                                attachment.trust_status,
                                attachment.confidence_effective,
                                attachment.capped
                            )),
                            decided_by: attachment.source.clone(),
                            decided_at: attachment.evaluated_at,
                            source_ruleset_version: attachment.ruleset_version,
                            evidence_json: Some(json!({
                                "trust_status": attachment.trust_status,
                                "confidence_effective": attachment.confidence_effective,
                                "capped": attachment.capped,
                                "source": attachment.source,
                            })),
                        },
                    )?;
                }

                self.emit_event(
                    run_id,
                    Some(step_id),
                    TraceEventType::StepReady,
                    "system",
                    "scheduler",
                    json!({"step_key": step.step_key, "step_index": idx}),
                    &mut chain,
                )?;

                for package in &step_request.injected_context_packages {
                    self.trace_store
                        .append_context_package(run_id, step_id, package)?;
                }

                self.emit_event(
                    run_id,
                    Some(step_id),
                    TraceEventType::StepInputPrepared,
                    "system",
                    "orchestrator",
                    json!({
                        "step_key": step.step_key,
                        "context_packages": step_request.injected_context_packages.len(),
                        "context_refs": refs.len(),
                        "trust_attachments": step_request.trust_gate_attachments.len(),
                    }),
                    &mut chain,
                )?;

                if trust_included + trust_excluded > 0 {
                    self.emit_event(
                        run_id,
                        Some(step_id),
                        TraceEventType::GateEvaluated,
                        "system",
                        "trust_gate",
                        json!({
                            "gate_kind": "trust",
                            "gate_name": "trust_gate",
                            "step_key": step.step_key,
                            "included": trust_included,
                            "excluded": trust_excluded,
                        }),
                        &mut chain,
                    )?;
                }

                if !pruned_references.is_empty() {
                    self.emit_event(
                        run_id,
                        Some(step_id),
                        TraceEventType::StepPermissionPruned,
                        "system",
                        "policy",
                        json!({
                            "count": pruned_references.len(),
                            "items": pruned_references,
                        }),
                        &mut chain,
                    )?;

                    self.emit_event(
                        run_id,
                        Some(step_id),
                        TraceEventType::Warning,
                        "system",
                        "policy",
                        json!({
                            "warning_code": "context_pruned",
                            "count": pruned_references.len(),
                            "continue_execution": true,
                        }),
                        &mut chain,
                    )?;

                    if effective_permissions.fail_on_permission_prune {
                        self.emit_event(
                            run_id,
                            Some(step_id),
                            TraceEventType::Warning,
                            "system",
                            "policy",
                            json!({
                                "warning_code": "fail_on_permission_prune_ignored",
                                "reason": "locked_decision_continue_on_prune",
                            }),
                            &mut chain,
                        )?;
                    }

                    self.trace_store.append_gate_decision(
                        run_id,
                        step_id,
                        &GateDecisionRecord {
                            gate_kind: GateKind::Policy,
                            gate_name: "context_permission".to_string(),
                            subject_type: "context_items".to_string(),
                            memory_id: None,
                            version: None,
                            memory_version_id: None,
                            decision: GateDecision::Pruned,
                            reason_codes: vec!["context_items_pruned".to_string()],
                            notes: Some(format!("{} item(s) pruned", pruned_references.len())),
                            decided_by: "policy.engine".to_string(),
                            decided_at: now_utc(),
                            source_ruleset_version: None,
                            evidence_json: None,
                        },
                    )?;
                }

                self.emit_event(
                    run_id,
                    Some(step_id),
                    TraceEventType::StepStarted,
                    "system",
                    "orchestrator",
                    json!({"step_key": step.step_key}),
                    &mut chain,
                )?;

                let mut rejected_by_human_gate = false;
                for gate_name in &step.gate_points {
                    let gate = workflow
                        .normalized_workflow
                        .gates
                        .iter()
                        .find(|candidate| candidate.gate_name == *gate_name)
                        .ok_or_else(|| anyhow!("missing gate {gate_name}"))?;

                    if gate.gate_kind != GateKind::Human {
                        continue;
                    }

                    let decision = self.human_gate.decide(&HumanGateRequest {
                        run_id,
                        step_id,
                        step_key: step.step_key.clone(),
                        gate_name: gate_name.clone(),
                        required: gate.required,
                        non_interactive: config.non_interactive,
                    })?;

                    let gate_decision = if decision.approved {
                        GateDecision::Approved
                    } else {
                        GateDecision::Rejected
                    };

                    self.trace_store.append_gate_decision(
                        run_id,
                        step_id,
                        &GateDecisionRecord {
                            gate_kind: GateKind::Human,
                            gate_name: gate_name.clone(),
                            subject_type: "step".to_string(),
                            memory_id: None,
                            version: None,
                            memory_version_id: None,
                            decision: gate_decision.clone(),
                            reason_codes: decision.reason_codes.clone(),
                            notes: decision.notes.clone(),
                            decided_by: decision.decided_by.clone(),
                            decided_at: now_utc(),
                            source_ruleset_version: None,
                            evidence_json: None,
                        },
                    )?;

                    self.emit_event(
                        run_id,
                        Some(step_id),
                        TraceEventType::GateEvaluated,
                        "human",
                        &decision.decided_by,
                        json!({
                            "gate_kind": "human",
                            "gate_name": gate_name,
                            "decision": match gate_decision {
                                GateDecision::Approved => "approved",
                                GateDecision::Rejected => "rejected",
                                GateDecision::Pruned => "pruned",
                            },
                            "required": gate.required,
                            "reason_codes": decision.reason_codes,
                            "notes": decision.notes,
                        }),
                        &mut chain,
                    )?;

                    if gate.required && !decision.approved {
                        rejected_by_human_gate = true;
                    }
                }

                let result = if rejected_by_human_gate {
                    StepResult {
                        run_id,
                        step_id,
                        status: StepStatus::Rejected,
                        outputs: multi_agent_center_domain::StepOutputEnvelope {
                            message: "step rejected by human gate".to_string(),
                            payload: json!({"rejected": true}),
                        },
                        proposed_memory_writes: Vec::new(),
                        provider_calls: Vec::new(),
                        gate_decisions: Vec::new(),
                        output_hash: String::new(),
                        error: None,
                    }
                } else {
                    match route_provider_call(&step_request) {
                        Ok(invocation) => {
                            self.persist_provider_call(run_id, step_id, &invocation, &mut chain)?;
                            build_step_result_from_provider(run_id, step_id, invocation)
                        }
                        Err(err) => {
                            self.emit_event(
                                run_id,
                                Some(step_id),
                                TraceEventType::Error,
                                "provider",
                                "router",
                                json!({
                                    "step_key": step.step_key,
                                    "error": err.to_string(),
                                }),
                                &mut chain,
                            )?;
                            StepResult {
                                run_id,
                                step_id,
                                status: StepStatus::Failed,
                                outputs: multi_agent_center_domain::StepOutputEnvelope {
                                    message: "provider invocation failed".to_string(),
                                    payload: json!({"failed": true}),
                                },
                                proposed_memory_writes: Vec::new(),
                                provider_calls: Vec::new(),
                                gate_decisions: Vec::new(),
                                output_hash: String::new(),
                                error: Some(multi_agent_center_domain::ErrorEnvelope {
                                    code: "provider_invocation_failed".to_string(),
                                    message: err.to_string(),
                                }),
                            }
                        }
                    }
                };

                let mut result = result;
                result.output_hash = compute_step_result_hash(&result)?;

                if config.apply_proposed_writes && !result.proposed_memory_writes.is_empty() {
                    for proposal in &result.proposed_memory_writes {
                        let apply = self.write_applier.apply(run_id, step_id, proposal)?;
                        self.trace_store.append_proposed_memory_write(
                            run_id,
                            step_id,
                            proposal,
                            &apply.disposition,
                            apply.disposition_reason.as_deref(),
                        )?;
                    }
                } else {
                    for proposal in &result.proposed_memory_writes {
                        self.trace_store.append_proposed_memory_write(
                            run_id,
                            step_id,
                            proposal,
                            "not_applied",
                            Some("apply_proposed_writes_disabled"),
                        )?;
                    }
                }

                self.trace_store.update_step_status(
                    step_id,
                    result.status.clone(),
                    Some(&result.output_hash),
                    result
                        .error
                        .as_ref()
                        .map(serde_json::to_value)
                        .transpose()?
                        .as_ref(),
                )?;

                self.emit_event(
                    run_id,
                    Some(step_id),
                    TraceEventType::StepFinished,
                    "system",
                    "orchestrator",
                    json!({
                        "step_key": step.step_key,
                        "status": step_status_to_text(&result.status),
                        "output_hash": result.output_hash,
                    }),
                    &mut chain,
                )?;

                statuses[idx] = result.status;
            }
        }

        let mut succeeded = 0_usize;
        let mut failed_or_rejected = 0_usize;
        for status in &statuses {
            match status {
                StepStatus::Succeeded => succeeded += 1,
                StepStatus::Failed | StepStatus::Rejected => failed_or_rejected += 1,
                StepStatus::Pending | StepStatus::Running | StepStatus::Skipped => {}
            }
        }

        let run_status = if statuses
            .iter()
            .any(|status| matches!(status, StepStatus::Rejected))
        {
            RunStatus::Rejected
        } else if statuses
            .iter()
            .any(|status| matches!(status, StepStatus::Failed))
        {
            RunStatus::Failed
        } else {
            RunStatus::Succeeded
        };

        self.trace_store
            .update_run_finished(run_id, run_status.clone())?;

        self.emit_event(
            run_id,
            None,
            TraceEventType::RunFinished,
            "system",
            "orchestrator",
            json!({
                "status": run_status_to_text(&run_status),
                "steps_total": total_steps,
                "steps_succeeded": succeeded,
                "steps_failed_or_rejected": failed_or_rejected,
            }),
            &mut chain,
        )?;

        Ok(RunExecutionSummary {
            run_id,
            status: run_status,
            steps_total: total_steps,
            steps_succeeded: succeeded,
            steps_failed_or_rejected: failed_or_rejected,
        })
    }

    /// Reconstruct and verify the event hash chain for a recorded run.
    ///
    /// # Errors
    /// Returns an error when trace rows cannot be read.
    pub fn replay_audit(&self, run_id: RunId) -> Result<ReplayReport> {
        let events = self.trace_store.list_events_for_run(run_id)?;
        let mut prev: Option<String> = None;
        for row in &events {
            if row.event.prev_event_hash != prev {
                return Ok(ReplayReport {
                    run_id,
                    events: events.len(),
                    chain_valid: false,
                });
            }
            prev = Some(row.event.event_hash.clone());
        }

        Ok(ReplayReport {
            run_id,
            events: events.len(),
            chain_valid: true,
        })
    }

    fn persist_provider_call(
        &self,
        run_id: RunId,
        step_id: StepId,
        invocation: &ProviderInvocation,
        chain: &mut EventChain,
    ) -> Result<()> {
        self.trace_store
            .append_provider_call(run_id, step_id, &invocation.provider_call)?;
        self.emit_event(
            run_id,
            Some(step_id),
            TraceEventType::ProviderCalled,
            "provider",
            &invocation.provider_call.provider_name,
            json!({
                "provider_call_id": invocation.provider_call.provider_call_id,
                "provider_name": invocation.provider_call.provider_name,
                "model_id": invocation.provider_call.model_id,
                "request_hash": invocation.provider_call.request_hash,
                "response_hash": invocation.provider_call.response_hash,
                "latency_ms": invocation.provider_call.latency_ms,
            }),
            chain,
        )?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_event(
        &self,
        run_id: RunId,
        step_id: Option<StepId>,
        event_type: TraceEventType,
        actor_type: &str,
        actor_id: &str,
        payload_json: Value,
        chain: &mut EventChain,
    ) -> Result<EventRow> {
        let occurred_at = now_utc();
        let recorded_at = now_utc();
        let payload_hash = hash_json(&payload_json)?;
        let event_id = Ulid::new();

        let material = json!({
            "event_id": event_id,
            "run_id": run_id,
            "step_id": step_id,
            "event_type": event_type,
            "occurred_at": format_rfc3339(occurred_at)?,
            "recorded_at": format_rfc3339(recorded_at)?,
            "actor_type": actor_type,
            "actor_id": actor_id,
            "payload_hash": payload_hash,
            "prev_event_hash": chain.prev_event_hash,
        });
        let event_hash = hash_json(&material)?;

        let event = TraceEvent {
            event_id,
            run_id,
            step_id,
            event_type,
            occurred_at,
            recorded_at,
            actor_type: actor_type.to_string(),
            actor_id: actor_id.to_string(),
            payload_json,
            payload_hash,
            prev_event_hash: chain.prev_event_hash.clone(),
            event_hash: event_hash.clone(),
        };

        let event_seq = self.trace_store.append_event(&event)?;
        chain.prev_event_hash = Some(event_hash);

        Ok(EventRow { event_seq, event })
    }
}

fn route_provider_call(request: &StepRequest) -> Result<ProviderInvocation> {
    match request.agent.provider.provider_name.as_str() {
        "mock" => MockProvider::new().invoke(request),
        "http_json" => HttpJsonProvider::new().invoke(request),
        other => Err(anyhow!(
            "unsupported provider adapter '{other}'; supported providers are 'mock' and 'http_json'"
        )),
    }
}

fn build_step_result_from_provider(
    run_id: RunId,
    step_id: StepId,
    invocation: ProviderInvocation,
) -> StepResult {
    StepResult {
        run_id,
        step_id,
        status: StepStatus::Succeeded,
        outputs: invocation.output,
        proposed_memory_writes: invocation.proposed_memory_writes,
        provider_calls: vec![invocation.provider_call],
        gate_decisions: Vec::new(),
        output_hash: String::new(),
        error: None,
    }
}

fn apply_trust_filter(
    packages: &[ContextPackageEnvelope],
    trust_map: &BTreeMap<(String, u32), bool>,
) -> Result<Vec<ContextPackageEnvelope>> {
    let mut out = Vec::with_capacity(packages.len());

    for package in packages {
        let mut package_copy = package.clone();
        package_copy.context_package.selected_items.retain(|item| {
            trust_map
                .get(&(item.memory_id.to_string(), item.version))
                .copied()
                .unwrap_or(true)
        });
        package_copy.package_hash =
            hash_json(&serde_json::to_value(&package_copy.context_package)?)?;
        out.push(package_copy);
    }

    Ok(out)
}

#[derive(Debug, Default)]
struct EventChain {
    prev_event_hash: Option<String>,
}

fn format_rfc3339(value: time::OffsetDateTime) -> Result<String> {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|err| anyhow!("invalid RFC3339 value: {err}"))
}

fn step_status_to_text(status: &StepStatus) -> &'static str {
    match status {
        StepStatus::Pending => "pending",
        StepStatus::Running => "running",
        StepStatus::Succeeded => "succeeded",
        StepStatus::Failed => "failed",
        StepStatus::Rejected => "rejected",
        StepStatus::Skipped => "skipped",
    }
}

fn run_status_to_text(status: &RunStatus) -> &'static str {
    match status {
        RunStatus::Pending => "pending",
        RunStatus::Running => "running",
        RunStatus::Succeeded => "succeeded",
        RunStatus::Failed => "failed",
        RunStatus::Rejected => "rejected",
    }
}

enum StepContextQuery {
    Policy(QueryRequest),
    Recall {
        text: String,
        record_types: Vec<RecordType>,
    },
}

fn resolve_step_context_queries(
    step: &multi_agent_center_domain::WorkflowStepDefinition,
    as_of: time::OffsetDateTime,
) -> Result<Vec<StepContextQuery>> {
    let mut queries: Vec<StepContextQuery> = Vec::new();

    if let Some(raw) = step.task.get("context_queries") {
        let array = raw
            .as_array()
            .ok_or_else(|| anyhow!("task.context_queries must be an array"))?;
        for entry in array {
            let object = entry
                .as_object()
                .ok_or_else(|| anyhow!("task.context_queries entries must be objects"))?;
            let text = object
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or(step.step_key.as_str())
                .to_string();
            let actor = object
                .get("actor")
                .and_then(Value::as_str)
                .unwrap_or("*")
                .to_string();
            let action = object
                .get("action")
                .and_then(Value::as_str)
                .unwrap_or("*")
                .to_string();
            let resource = object
                .get("resource")
                .and_then(Value::as_str)
                .unwrap_or("*")
                .to_string();
            let mode = object
                .get("mode")
                .and_then(Value::as_str)
                .map(str::trim)
                .map_or_else(|| "policy".to_string(), str::to_ascii_lowercase);

            match mode.as_str() {
                "policy" => queries.push(StepContextQuery::Policy(QueryRequest {
                    text,
                    actor,
                    action,
                    resource,
                    as_of,
                })),
                "recall" => {
                    let record_types = parse_recall_record_types(object.get("record_types"))?;
                    queries.push(StepContextQuery::Recall { text, record_types });
                }
                _ => {
                    return Err(anyhow!(
                        "task.context_queries mode must be one of: policy, recall"
                    ));
                }
            }
        }
    } else {
        let text = step
            .task
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or(step.step_key.as_str())
            .to_string();
        let actor = step
            .task
            .get("actor")
            .and_then(Value::as_str)
            .unwrap_or("*")
            .to_string();
        let action = step
            .task
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("*")
            .to_string();
        let resource = step
            .task
            .get("resource")
            .and_then(Value::as_str)
            .unwrap_or("*")
            .to_string();
        queries.push(StepContextQuery::Policy(QueryRequest {
            text,
            actor,
            action,
            resource,
            as_of,
        }));
    }

    Ok(queries)
}

fn build_context_packages_from_records(
    records: &[MemoryRecord],
    run_id: RunId,
    step: &multi_agent_center_domain::WorkflowStepDefinition,
    as_of: time::OffsetDateTime,
    source: &str,
) -> Result<Vec<ContextPackageEnvelope>> {
    if records.is_empty() {
        return Ok(Vec::new());
    }
    let queries = resolve_step_context_queries(step, as_of)?;

    let mut envelopes = Vec::with_capacity(queries.len());
    for (package_slot, query) in queries.into_iter().enumerate() {
        let package = match query {
            StepContextQuery::Policy(request) => build_context_package(
                records,
                request,
                &format!("{}:{}:{}", run_id, step.step_key, package_slot),
            ),
            StepContextQuery::Recall { text, record_types } => build_recall_context_package(
                records,
                QueryRequest {
                    text,
                    actor: "*".to_string(),
                    action: "*".to_string(),
                    resource: "*".to_string(),
                    as_of,
                },
                &format!("{}:{}:{}", run_id, step.step_key, package_slot),
                &record_types,
            ),
        }
        .map_err(|err| anyhow!("memory kernel context package build failed: {err}"))?;
        let package_json = serde_json::to_value(&package)?;
        let package_hash = hash_json(&package_json)?;
        envelopes.push(ContextPackageEnvelope {
            package_slot,
            source: source.to_string(),
            context_package: package,
            package_hash,
        });
    }

    Ok(envelopes)
}

fn parse_recall_record_types(raw: Option<&Value>) -> Result<Vec<RecordType>> {
    let Some(value) = raw else {
        return Ok(default_recall_record_types());
    };

    let array = value
        .as_array()
        .ok_or_else(|| anyhow!("task.context_queries[].record_types must be an array"))?;
    if array.is_empty() {
        return Ok(default_recall_record_types());
    }

    let mut record_types = Vec::with_capacity(array.len());
    for item in array {
        let raw_record_type = item
            .as_str()
            .ok_or_else(|| anyhow!("task.context_queries[].record_types entries must be strings"))?
            .trim()
            .to_ascii_lowercase();
        let record_type = RecordType::parse(&raw_record_type).ok_or_else(|| {
            anyhow!(
                "unsupported recall record type `{raw_record_type}`; expected one of constraint|decision|preference|event|outcome"
            )
        })?;
        record_types.push(record_type);
    }

    Ok(record_types)
}

fn load_outcome_rulesets(conn: &rusqlite::Connection) -> Result<BTreeMap<u32, OutcomeRuleset>> {
    let mut stmt = conn.prepare(
        "SELECT ruleset_version, ruleset_json
         FROM outcome_rulesets
         ORDER BY ruleset_version ASC",
    )?;
    let mut rows = stmt.query([])?;

    let mut out = BTreeMap::new();
    while let Some(row) = rows.next()? {
        let version_i64: i64 = row.get(0)?;
        let version = u32::try_from(version_i64)
            .map_err(|_| anyhow!("invalid outcome ruleset_version: {version_i64}"))?;
        let ruleset_json: String = row.get(1)?;
        let value: Value =
            serde_json::from_str(&ruleset_json).context("invalid outcome_rulesets.ruleset_json")?;
        let ruleset = OutcomeRuleset::from_json(&value)
            .map_err(|err| anyhow!("invalid outcome ruleset {version}: {err}"))?;
        out.insert(version, ruleset);
    }

    if out.is_empty() {
        out.insert(1, OutcomeRuleset::v1());
    }

    Ok(out)
}

fn get_memory_trust_and_ruleset(
    conn: &rusqlite::Connection,
    memory_id: memory_kernel_core::MemoryId,
    version: u32,
) -> Result<Option<(MemoryTrust, u32)>> {
    let mut stmt = conn.prepare(
        "SELECT
            confidence_raw, confidence_effective, baseline_confidence,
            trust_status, contradiction_cap_active, cap_value, manual_override_active,
            wins_last5, failures_last5, last_event_seq, last_ruleset_version,
            last_scored_at, updated_at
         FROM memory_trust
         WHERE memory_id = ?1 AND version = ?2",
    )?;

    stmt.query_row(
        rusqlite::params![memory_id.to_string(), i64::from(version)],
        |row| {
            let trust_status_raw: String = row.get(3)?;
            let trust_status = TrustStatus::parse(&trust_status_raw).ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid trust_status: {trust_status_raw}"),
                    )),
                )
            })?;

            let last_ruleset_version_i64: i64 = row.get(10)?;
            let last_ruleset_version = u32::try_from(last_ruleset_version_i64).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    10,
                    rusqlite::types::Type::Integer,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid ruleset version: {last_ruleset_version_i64}"),
                    )),
                )
            })?;

            let last_scored_at = row
                .get::<_, Option<String>>(11)?
                .as_deref()
                .map(|value| parse_rfc3339_utc(value).map_err(|err| to_sql_error(&err)))
                .transpose()?;
            let updated_at_raw: String = row.get(12)?;
            let updated_at =
                parse_rfc3339_utc(&updated_at_raw).map_err(|err| to_sql_error(&err))?;

            let wins_last5_i64: i64 = row.get(7)?;
            let failures_last5_i64: i64 = row.get(8)?;
            let wins_last5 = u8::try_from(wins_last5_i64).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    7,
                    rusqlite::types::Type::Integer,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid wins_last5: {wins_last5_i64}"),
                    )),
                )
            })?;
            let failures_last5 = u8::try_from(failures_last5_i64).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Integer,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("invalid failures_last5: {failures_last5_i64}"),
                    )),
                )
            })?;

            let trust = MemoryTrust {
                memory_id,
                version,
                confidence_raw: row.get(0)?,
                confidence_effective: row.get(1)?,
                baseline_confidence: row.get(2)?,
                trust_status,
                contradiction_cap_active: row.get::<_, i64>(4)? == 1,
                cap_value: row.get(5)?,
                manual_override_active: row.get::<_, i64>(6)? == 1,
                wins_last5,
                failures_last5,
                last_event_seq: row.get(9)?,
                last_scored_at,
                updated_at,
            };
            Ok((trust, last_ruleset_version))
        },
    )
    .optional()
    .map_err(anyhow::Error::from)
}

fn to_sql_error(err: &memory_kernel_outcome_core::OutcomeError) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        rusqlite::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            err.to_string(),
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        AllowAllTrustGateSource, ApiMemoryKernelContextSource, ContextRef, DefaultHumanGateDecider,
        HumanGateDecider, HumanGateRequest, HumanGateResponse, InMemoryMemoryKernelContextSource,
        NoopProposedWriteApplier, Orchestrator, RunConfig, TrustGateAttachment, TrustGateSource,
    };
    use memory_kernel_api::{AddConstraintRequest, AddSummaryRequest, MemoryKernelApi};
    use memory_kernel_core::{
        default_recall_record_types, Answer, AnswerResult, Authority, ConstraintEffect,
        ConstraintPayload, ConstraintScope, ContextItem, ContextPackage, DecisionPayload,
        DeterminismMetadata, MemoryId, MemoryPayload, MemoryRecord, MemoryVersionId, QueryRequest,
        RecordType, TruthStatus, Why,
    };
    use multi_agent_center_domain::{ContextPackageEnvelope, StepId, StepStatus};
    use multi_agent_center_trace_core::TraceStore;
    use multi_agent_center_trace_sqlite::SqliteTraceStore;
    use multi_agent_center_workflow::normalize_workflow_yaml;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn temp_db_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "mac-orchestrator-test-{}-{}.sqlite",
            name,
            ulid::Ulid::new()
        ))
    }

    fn fixture_memory_record() -> MemoryRecord {
        let now = time::OffsetDateTime::now_utc();
        MemoryRecord {
            memory_version_id: MemoryVersionId::new(),
            memory_id: MemoryId::new(),
            version: 1,
            created_at: now,
            effective_at: now,
            truth_status: TruthStatus::Asserted,
            authority: Authority::Authoritative,
            confidence: Some(0.9),
            writer: "test".to_string(),
            justification: "fixture".to_string(),
            provenance: memory_kernel_core::Provenance {
                source_uri: "test://fixture".to_string(),
                source_hash: None,
                evidence: Vec::new(),
            },
            supersedes: Vec::new(),
            contradicts: Vec::new(),
            payload: MemoryPayload::Constraint(ConstraintPayload {
                scope: ConstraintScope {
                    actor: "*".to_string(),
                    action: "*".to_string(),
                    resource: "*".to_string(),
                },
                effect: ConstraintEffect::Allow,
                note: None,
            }),
        }
    }

    fn fixture_summary_record(record_type: RecordType, summary: &str) -> MemoryRecord {
        let now = time::OffsetDateTime::now_utc();
        let payload = match record_type {
            RecordType::Decision => MemoryPayload::Decision(DecisionPayload {
                summary: summary.to_string(),
            }),
            RecordType::Preference => {
                MemoryPayload::Preference(memory_kernel_core::PreferencePayload {
                    summary: summary.to_string(),
                })
            }
            RecordType::Event => MemoryPayload::Event(memory_kernel_core::EventPayload {
                summary: summary.to_string(),
            }),
            RecordType::Outcome => MemoryPayload::Outcome(memory_kernel_core::OutcomePayload {
                summary: summary.to_string(),
            }),
            RecordType::Constraint => panic!("fixture_summary_record does not support constraint"),
        };

        MemoryRecord {
            memory_version_id: MemoryVersionId::new(),
            memory_id: MemoryId::new(),
            version: 1,
            created_at: now,
            effective_at: now,
            truth_status: TruthStatus::Observed,
            authority: Authority::Derived,
            confidence: Some(0.8),
            writer: "test".to_string(),
            justification: "fixture".to_string(),
            provenance: memory_kernel_core::Provenance {
                source_uri: "test://fixture".to_string(),
                source_hash: None,
                evidence: Vec::new(),
            },
            supersedes: Vec::new(),
            contradicts: Vec::new(),
            payload,
        }
    }

    fn fixture_context_package(step_key: &str) -> ContextPackageEnvelope {
        let now = time::OffsetDateTime::now_utc();
        let selected_items = vec![
            ContextItem {
                rank: 1,
                memory_version_id: MemoryVersionId::new(),
                memory_id: MemoryId::new(),
                record_type: RecordType::Constraint,
                version: 1,
                truth_status: TruthStatus::Asserted,
                confidence: Some(0.9),
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
                record_type: RecordType::Constraint,
                version: 2,
                truth_status: TruthStatus::Asserted,
                confidence: Some(0.7),
                authority: Authority::Authoritative,
                why: Why {
                    included: true,
                    reasons: vec!["fixture".to_string()],
                    rule_scores: None,
                },
            },
        ];
        let package = ContextPackage {
            context_package_id: format!("pkg-{step_key}"),
            generated_at: now,
            query: QueryRequest {
                text: "t".to_string(),
                actor: "a".to_string(),
                action: "act".to_string(),
                resource: "res".to_string(),
                as_of: now,
            },
            determinism: DeterminismMetadata {
                ruleset_version: "mk.v1".to_string(),
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
            package_hash: "fixture-hash".to_string(),
            context_package: package,
        }
    }

    struct ApproveHumanGate;

    impl HumanGateDecider for ApproveHumanGate {
        fn decide(&self, _request: &HumanGateRequest) -> anyhow::Result<HumanGateResponse> {
            Ok(HumanGateResponse {
                approved: true,
                notes: None,
                decided_by: "test".to_string(),
                reason_codes: vec!["approved.test".to_string()],
            })
        }
    }

    struct SelectiveTrustGate;

    impl TrustGateSource for SelectiveTrustGate {
        fn evaluate(
            &self,
            _run_id: multi_agent_center_domain::RunId,
            _step_id: StepId,
            _step_key: &str,
            as_of: time::OffsetDateTime,
            refs: &[ContextRef],
        ) -> anyhow::Result<Vec<TrustGateAttachment>> {
            Ok(refs
                .iter()
                .map(|item| TrustGateAttachment {
                    memory_id: item.memory_id,
                    version: item.version,
                    memory_version_id: item.memory_version_id,
                    include: item.version % 2 == 1,
                    trust_status: "active".to_string(),
                    confidence_effective: 0.8,
                    capped: false,
                    reason_codes: vec!["fixture".to_string()],
                    ruleset_version: Some(1),
                    evaluated_at: as_of,
                    source: "test.trust".to_string(),
                })
                .collect())
        }
    }

    #[test]
    fn provider_failure_marks_step_failed_instead_of_aborting_run() {
        let trace_db = temp_db_path("provider-failure");
        let trace_store = SqliteTraceStore::open(&trace_db);
        assert!(trace_store.is_ok());
        let trace_store = trace_store.unwrap_or_else(|_| unreachable!());
        assert!(trace_store.migrate().is_ok());

        let workflow_yaml = r#"
workflow_name: wf
workflow_version: v1
normalization_version: 0
agents:
  - agent_name: planner
    role: planning
    provider:
      provider_name: unsupported_provider
      model_id: x
steps:
  - step_key: step_a
    agent_name: planner
    task: { text: "a" }
    depends_on: []
    gate_points: []
  - step_key: step_b
    agent_name: planner
    task: { text: "b" }
    depends_on: [step_a]
    gate_points: []
gates: []
defaults:
  non_interactive: true
"#;
        let workflow = normalize_workflow_yaml(workflow_yaml);
        assert!(workflow.is_ok());
        let workflow = workflow.unwrap_or_else(|_| unreachable!());

        let context_source = super::StaticContextPackageSource::default();
        let trust_source = AllowAllTrustGateSource;
        let human_gate = DefaultHumanGateDecider;
        let write_applier = NoopProposedWriteApplier;

        let summary = Orchestrator::new(
            &trace_store,
            &context_source,
            &trust_source,
            &human_gate,
            &write_applier,
        )
        .execute_workflow(
            &workflow,
            RunConfig {
                non_interactive: true,
                ..RunConfig::default()
            },
        );
        let summary = match summary {
            Ok(value) => value,
            Err(err) => panic!("workflow execution failed: {err:#}"),
        };
        assert_eq!(summary.status, multi_agent_center_domain::RunStatus::Failed);
        let run_record = trace_store.get_run(summary.run_id);
        assert!(run_record.is_ok());
        let run_record = run_record
            .unwrap_or_else(|_| unreachable!())
            .unwrap_or_else(|| unreachable!());
        assert!(run_record.manifest_hash.is_some());
        assert_eq!(run_record.manifest_signature_status, "unsigned");

        let steps = trace_store.get_step_records(summary.run_id);
        assert!(steps.is_ok());
        let steps = steps.unwrap_or_else(|_| unreachable!());
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].status, StepStatus::Failed);
        assert_eq!(steps[1].status, StepStatus::Skipped);
    }

    #[test]
    fn step_supports_multiple_context_packages() {
        let trace_db = temp_db_path("multi-packages");
        let trace_store = SqliteTraceStore::open(&trace_db);
        assert!(trace_store.is_ok());
        let trace_store = trace_store.unwrap_or_else(|_| unreachable!());
        assert!(trace_store.migrate().is_ok());

        let workflow_yaml = r#"
workflow_name: wf
workflow_version: v1
normalization_version: 0
agents:
  - agent_name: planner
    role: planning
    provider:
      provider_name: mock
      model_id: x
steps:
  - step_key: step_a
    agent_name: planner
    task:
      context_queries:
        - { text: "q1", actor: "a", action: "x", resource: "r1" }
        - { text: "q2", actor: "a", action: "x", resource: "r2" }
    depends_on: []
    gate_points: []
gates: []
defaults:
  non_interactive: true
"#;
        let workflow = normalize_workflow_yaml(workflow_yaml);
        assert!(workflow.is_ok());
        let workflow = workflow.unwrap_or_else(|_| unreachable!());

        let mut records_by_step = BTreeMap::new();
        records_by_step.insert("step_a".to_string(), vec![fixture_memory_record()]);
        let context_source = InMemoryMemoryKernelContextSource { records_by_step };

        let trust_source = AllowAllTrustGateSource;
        let human_gate = DefaultHumanGateDecider;
        let write_applier = NoopProposedWriteApplier;
        let summary = Orchestrator::new(
            &trace_store,
            &context_source,
            &trust_source,
            &human_gate,
            &write_applier,
        )
        .execute_workflow(
            &workflow,
            RunConfig {
                non_interactive: true,
                ..RunConfig::default()
            },
        );
        let summary = match summary {
            Ok(value) => value,
            Err(err) => panic!("workflow execution failed: {err:#}"),
        };
        assert_eq!(
            summary.status,
            multi_agent_center_domain::RunStatus::Succeeded
        );

        let packages = trace_store.get_step_context_packages(summary.run_id);
        assert!(packages.is_ok());
        let packages = packages.unwrap_or_else(|_| unreachable!());
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].envelope.package_slot, 0);
        assert_eq!(packages[1].envelope.package_slot, 1);
    }

    #[test]
    fn step_context_query_recall_mode_uses_recall_resolver_rules() {
        let trace_db = temp_db_path("recall-mode");
        let trace_store = SqliteTraceStore::open(&trace_db);
        assert!(trace_store.is_ok());
        let trace_store = trace_store.unwrap_or_else(|_| unreachable!());
        assert!(trace_store.migrate().is_ok());

        let workflow_yaml = r#"
workflow_name: wf
workflow_version: v1
normalization_version: 0
agents:
  - agent_name: planner
    role: planning
    provider:
      provider_name: mock
      model_id: x
steps:
  - step_key: step_a
    agent_name: planner
    task:
      context_queries:
        - { mode: "recall", text: "usb project notes", record_types: ["decision", "outcome"] }
    depends_on: []
    gate_points: []
gates: []
defaults:
  non_interactive: true
"#;
        let workflow = normalize_workflow_yaml(workflow_yaml);
        assert!(workflow.is_ok());
        let workflow = workflow.unwrap_or_else(|_| unreachable!());

        let mut records_by_step = BTreeMap::new();
        records_by_step.insert(
            "step_a".to_string(),
            vec![
                fixture_memory_record(),
                fixture_summary_record(RecordType::Decision, "usb project notes decision"),
                fixture_summary_record(RecordType::Outcome, "usb project notes outcome"),
            ],
        );
        let context_source = InMemoryMemoryKernelContextSource { records_by_step };

        let trust_source = AllowAllTrustGateSource;
        let human_gate = DefaultHumanGateDecider;
        let write_applier = NoopProposedWriteApplier;
        let summary = Orchestrator::new(
            &trace_store,
            &context_source,
            &trust_source,
            &human_gate,
            &write_applier,
        )
        .execute_workflow(
            &workflow,
            RunConfig {
                non_interactive: true,
                ..RunConfig::default()
            },
        );
        let summary = match summary {
            Ok(value) => value,
            Err(err) => panic!("workflow execution failed: {err:#}"),
        };
        assert_eq!(
            summary.status,
            multi_agent_center_domain::RunStatus::Succeeded
        );

        let packages = trace_store.get_step_context_packages(summary.run_id);
        assert!(packages.is_ok());
        let packages = packages.unwrap_or_else(|_| unreachable!());
        assert_eq!(packages.len(), 1);
        let selected = &packages[0].envelope.context_package.selected_items;
        assert!(selected
            .iter()
            .all(|item| item.record_type == RecordType::Decision
                || item.record_type == RecordType::Outcome));
    }

    #[test]
    fn step_context_query_recall_missing_record_types_defaults_scope() {
        let trace_db = temp_db_path("recall-default-missing");
        let trace_store = SqliteTraceStore::open(&trace_db);
        assert!(trace_store.is_ok());
        let trace_store = trace_store.unwrap_or_else(|_| unreachable!());
        assert!(trace_store.migrate().is_ok());

        let workflow_yaml = r#"
workflow_name: wf
workflow_version: v1
normalization_version: 0
agents:
  - agent_name: planner
    role: planning
    provider:
      provider_name: mock
      model_id: x
steps:
  - step_key: step_a
    agent_name: planner
    task:
      context_queries:
        - { mode: "recall", text: "repo policy decision" }
    depends_on: []
    gate_points: []
gates: []
defaults:
  non_interactive: true
"#;
        let workflow = normalize_workflow_yaml(workflow_yaml);
        assert!(workflow.is_ok());
        let workflow = workflow.unwrap_or_else(|_| unreachable!());

        let mut records_by_step = BTreeMap::new();
        records_by_step.insert(
            "step_a".to_string(),
            vec![
                fixture_memory_record(),
                fixture_summary_record(RecordType::Decision, "repo policy decision"),
                fixture_summary_record(RecordType::Outcome, "repo policy outcome"),
            ],
        );
        let context_source = InMemoryMemoryKernelContextSource { records_by_step };

        let summary = Orchestrator::new(
            &trace_store,
            &context_source,
            &AllowAllTrustGateSource,
            &DefaultHumanGateDecider,
            &NoopProposedWriteApplier,
        )
        .execute_workflow(
            &workflow,
            RunConfig {
                non_interactive: true,
                ..RunConfig::default()
            },
        );
        let summary = match summary {
            Ok(value) => value,
            Err(err) => panic!("workflow execution failed: {err:#}"),
        };
        assert_eq!(
            summary.status,
            multi_agent_center_domain::RunStatus::Succeeded
        );

        let packages = trace_store.get_step_context_packages(summary.run_id);
        assert!(packages.is_ok());
        let packages = packages.unwrap_or_else(|_| unreachable!());
        assert_eq!(packages.len(), 1);
        let selected = &packages[0].envelope.context_package.selected_items;
        assert!(!selected.is_empty());
        let default_types = default_recall_record_types();
        assert!(selected
            .iter()
            .all(|item| default_types.contains(&item.record_type)));
    }

    #[test]
    fn step_context_query_recall_empty_record_types_defaults_scope() {
        let trace_db = temp_db_path("recall-default-empty");
        let trace_store = SqliteTraceStore::open(&trace_db);
        assert!(trace_store.is_ok());
        let trace_store = trace_store.unwrap_or_else(|_| unreachable!());
        assert!(trace_store.migrate().is_ok());

        let workflow_yaml = r#"
workflow_name: wf
workflow_version: v1
normalization_version: 0
agents:
  - agent_name: planner
    role: planning
    provider:
      provider_name: mock
      model_id: x
steps:
  - step_key: step_a
    agent_name: planner
    task:
      context_queries:
        - { mode: "recall", text: "repo policy outcome", record_types: [] }
    depends_on: []
    gate_points: []
gates: []
defaults:
  non_interactive: true
"#;
        let workflow = normalize_workflow_yaml(workflow_yaml);
        assert!(workflow.is_ok());
        let workflow = workflow.unwrap_or_else(|_| unreachable!());

        let mut records_by_step = BTreeMap::new();
        records_by_step.insert(
            "step_a".to_string(),
            vec![
                fixture_memory_record(),
                fixture_summary_record(RecordType::Decision, "repo policy decision"),
                fixture_summary_record(RecordType::Outcome, "repo policy outcome"),
            ],
        );
        let context_source = InMemoryMemoryKernelContextSource { records_by_step };

        let summary = Orchestrator::new(
            &trace_store,
            &context_source,
            &AllowAllTrustGateSource,
            &DefaultHumanGateDecider,
            &NoopProposedWriteApplier,
        )
        .execute_workflow(
            &workflow,
            RunConfig {
                non_interactive: true,
                ..RunConfig::default()
            },
        );
        let summary = match summary {
            Ok(value) => value,
            Err(err) => panic!("workflow execution failed: {err:#}"),
        };
        assert_eq!(
            summary.status,
            multi_agent_center_domain::RunStatus::Succeeded
        );

        let packages = trace_store.get_step_context_packages(summary.run_id);
        assert!(packages.is_ok());
        let packages = packages.unwrap_or_else(|_| unreachable!());
        assert_eq!(packages.len(), 1);
        let selected = &packages[0].envelope.context_package.selected_items;
        assert!(!selected.is_empty());
        let default_types = default_recall_record_types();
        assert!(selected
            .iter()
            .all(|item| default_types.contains(&item.record_type)));
    }

    #[test]
    fn step_context_query_recall_invalid_record_type_fails() {
        let trace_db = temp_db_path("recall-invalid-type");
        let trace_store = SqliteTraceStore::open(&trace_db);
        assert!(trace_store.is_ok());
        let trace_store = trace_store.unwrap_or_else(|_| unreachable!());
        assert!(trace_store.migrate().is_ok());

        let workflow_yaml = r#"
workflow_name: wf
workflow_version: v1
normalization_version: 0
agents:
  - agent_name: planner
    role: planning
    provider:
      provider_name: mock
      model_id: x
steps:
  - step_key: step_a
    agent_name: planner
    task:
      context_queries:
        - { mode: "recall", text: "repo policy decision", record_types: ["constraint", "invalid_type"] }
    depends_on: []
    gate_points: []
gates: []
defaults:
  non_interactive: true
"#;
        let workflow = normalize_workflow_yaml(workflow_yaml);
        assert!(workflow.is_ok());
        let workflow = workflow.unwrap_or_else(|_| unreachable!());

        let mut records_by_step = BTreeMap::new();
        records_by_step.insert("step_a".to_string(), vec![fixture_memory_record()]);
        let context_source = InMemoryMemoryKernelContextSource { records_by_step };

        let run = Orchestrator::new(
            &trace_store,
            &context_source,
            &AllowAllTrustGateSource,
            &DefaultHumanGateDecider,
            &NoopProposedWriteApplier,
        )
        .execute_workflow(
            &workflow,
            RunConfig {
                non_interactive: true,
                ..RunConfig::default()
            },
        );
        assert!(run.is_err());
    }

    #[test]
    fn parse_recall_record_types_invalid_value_errors() {
        let raw = json!(["decision", "invalid_type"]);
        let parsed = super::parse_recall_record_types(Some(&raw));
        assert!(parsed.is_err());
        let err = parsed.err().unwrap_or_else(|| unreachable!());
        let message = err.to_string();
        assert!(message.contains("unsupported recall record type"));
        assert!(message.contains("invalid_type"));
    }

    #[test]
    fn step_context_query_default_mode_is_policy() {
        let trace_db = temp_db_path("default-policy-mode");
        let trace_store = SqliteTraceStore::open(&trace_db);
        assert!(trace_store.is_ok());
        let trace_store = trace_store.unwrap_or_else(|_| unreachable!());
        assert!(trace_store.migrate().is_ok());

        let workflow_yaml = r#"
workflow_name: wf
workflow_version: v1
normalization_version: 0
agents:
  - agent_name: planner
    role: planning
    provider:
      provider_name: mock
      model_id: x
steps:
  - step_key: step_a
    agent_name: planner
    task:
      context_queries:
        - { text: "Can dev read repo?", actor: "dev", action: "read", resource: "repo" }
    depends_on: []
    gate_points: []
gates: []
defaults:
  non_interactive: true
"#;
        let workflow = normalize_workflow_yaml(workflow_yaml);
        assert!(workflow.is_ok());
        let workflow = workflow.unwrap_or_else(|_| unreachable!());

        let mut records_by_step = BTreeMap::new();
        records_by_step.insert("step_a".to_string(), vec![fixture_memory_record()]);
        let context_source = InMemoryMemoryKernelContextSource { records_by_step };

        let summary = Orchestrator::new(
            &trace_store,
            &context_source,
            &AllowAllTrustGateSource,
            &DefaultHumanGateDecider,
            &NoopProposedWriteApplier,
        )
        .execute_workflow(
            &workflow,
            RunConfig {
                non_interactive: true,
                ..RunConfig::default()
            },
        );
        let summary = match summary {
            Ok(value) => value,
            Err(err) => panic!("workflow execution failed: {err:#}"),
        };
        assert_eq!(
            summary.status,
            multi_agent_center_domain::RunStatus::Succeeded
        );

        let packages = trace_store.get_step_context_packages(summary.run_id);
        assert!(packages.is_ok());
        let packages = packages.unwrap_or_else(|_| unreachable!());
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].envelope.context_package.query.actor, "dev");
        assert_eq!(packages[0].envelope.context_package.query.action, "read");
        assert_eq!(packages[0].envelope.context_package.query.resource, "repo");
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn api_memory_kernel_context_source_builds_step_packages() {
        let trace_db = temp_db_path("api-context-source");
        let memory_db = temp_db_path("api-memory-db");

        let api = MemoryKernelApi::new(memory_db.clone());
        assert!(api.migrate(false).is_ok());
        assert!(api
            .add_constraint(AddConstraintRequest {
                actor: "dev".to_string(),
                action: "read".to_string(),
                resource: "repo".to_string(),
                effect: ConstraintEffect::Allow,
                note: None,
                memory_id: None,
                version: 1,
                writer: "test".to_string(),
                justification: "seed constraint".to_string(),
                source_uri: "file:///constraint.md".to_string(),
                source_hash: None,
                evidence: Vec::new(),
                confidence: Some(0.9),
                truth_status: TruthStatus::Observed,
                authority: Authority::Authoritative,
                created_at: None,
                effective_at: None,
                supersedes: Vec::new(),
                contradicts: Vec::new(),
            })
            .is_ok());
        assert!(api
            .add_summary(AddSummaryRequest {
                record_type: RecordType::Decision,
                summary: "repo policy decision".to_string(),
                memory_id: None,
                version: 1,
                writer: "test".to_string(),
                justification: "seed decision".to_string(),
                source_uri: "file:///decision.md".to_string(),
                source_hash: None,
                evidence: Vec::new(),
                confidence: Some(0.8),
                truth_status: TruthStatus::Observed,
                authority: Authority::Derived,
                created_at: None,
                effective_at: None,
                supersedes: Vec::new(),
                contradicts: Vec::new(),
            })
            .is_ok());

        let trace_store = SqliteTraceStore::open(&trace_db);
        assert!(trace_store.is_ok());
        let trace_store = trace_store.unwrap_or_else(|_| unreachable!());
        assert!(trace_store.migrate().is_ok());

        let workflow_yaml = r#"
workflow_name: wf
workflow_version: v1
normalization_version: 0
agents:
  - agent_name: planner
    role: planning
    provider:
      provider_name: mock
      model_id: x
steps:
  - step_key: step_a
    agent_name: planner
    task:
      context_queries:
        - { mode: "policy", text: "Can dev read repo?", actor: "dev", action: "read", resource: "repo" }
        - { mode: "recall", text: "repo policy decision", record_types: ["decision"] }
    depends_on: []
    gate_points: []
gates: []
defaults:
  non_interactive: true
"#;
        let workflow = normalize_workflow_yaml(workflow_yaml);
        assert!(workflow.is_ok());
        let workflow = workflow.unwrap_or_else(|_| unreachable!());

        let context_source = ApiMemoryKernelContextSource::new(&memory_db);
        let trust_source = AllowAllTrustGateSource;
        let human_gate = DefaultHumanGateDecider;
        let write_applier = NoopProposedWriteApplier;
        let summary = Orchestrator::new(
            &trace_store,
            &context_source,
            &trust_source,
            &human_gate,
            &write_applier,
        )
        .execute_workflow(
            &workflow,
            RunConfig {
                non_interactive: true,
                ..RunConfig::default()
            },
        );
        let summary = match summary {
            Ok(value) => value,
            Err(err) => panic!("workflow execution failed: {err:#}"),
        };
        assert_eq!(
            summary.status,
            multi_agent_center_domain::RunStatus::Succeeded
        );

        let packages = trace_store.get_step_context_packages(summary.run_id);
        assert!(packages.is_ok());
        let packages = packages.unwrap_or_else(|_| unreachable!());
        assert_eq!(packages.len(), 2);
        assert_eq!(
            packages[0].envelope.context_package.query.actor, "dev",
            "first query should run policy mode ask() semantics"
        );
        assert_eq!(packages[0].envelope.context_package.query.action, "read");
        assert_eq!(packages[0].envelope.context_package.query.resource, "repo");
        assert!(packages[0]
            .envelope
            .context_package
            .selected_items
            .iter()
            .any(|item| item.record_type == RecordType::Constraint));
        assert!(packages[1]
            .envelope
            .context_package
            .selected_items
            .iter()
            .all(|item| item.record_type == RecordType::Decision));
    }

    #[test]
    fn trust_gate_decisions_are_persisted_per_memory_ref() {
        let trace_db = temp_db_path("trust-gates");
        let trace_store = SqliteTraceStore::open(&trace_db);
        assert!(trace_store.is_ok());
        let trace_store = trace_store.unwrap_or_else(|_| unreachable!());
        assert!(trace_store.migrate().is_ok());

        let workflow_yaml = r#"
workflow_name: wf
workflow_version: v1
normalization_version: 0
agents:
  - agent_name: planner
    role: planning
    provider:
      provider_name: mock
      model_id: x
steps:
  - step_key: step_a
    agent_name: planner
    task: { text: "a" }
    depends_on: []
    gate_points: []
gates: []
defaults:
  non_interactive: true
"#;
        let workflow = normalize_workflow_yaml(workflow_yaml);
        assert!(workflow.is_ok());
        let workflow = workflow.unwrap_or_else(|_| unreachable!());

        let mut by_step = BTreeMap::new();
        by_step.insert(
            "step_a".to_string(),
            vec![fixture_context_package("step_a")],
        );
        let context_source = super::StaticContextPackageSource::with_step_packages(by_step);
        let trust_source = SelectiveTrustGate;
        let human_gate = ApproveHumanGate;
        let write_applier = NoopProposedWriteApplier;
        let summary = Orchestrator::new(
            &trace_store,
            &context_source,
            &trust_source,
            &human_gate,
            &write_applier,
        )
        .execute_workflow(
            &workflow,
            RunConfig {
                non_interactive: true,
                ..RunConfig::default()
            },
        );
        let summary = match summary {
            Ok(value) => value,
            Err(err) => panic!("workflow execution failed: {err:#}"),
        };

        let conn = rusqlite::Connection::open(&trace_db).unwrap_or_else(|_| unreachable!());
        let trust_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM step_gate_decisions WHERE run_id = ?1 AND gate_kind = ?2",
                rusqlite::params![summary.run_id.to_string(), "trust"],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| unreachable!());
        let rejected_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM step_gate_decisions
                 WHERE run_id = ?1 AND gate_kind = ?2 AND decision = ?3",
                rusqlite::params![summary.run_id.to_string(), "trust", "rejected"],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(trust_rows, 2);
        assert_eq!(rejected_rows, 1);
    }

    #[test]
    fn dag_with_parallel_ready_steps_executes_successfully() {
        let trace_db = temp_db_path("parallel-ready");
        let trace_store = SqliteTraceStore::open(&trace_db);
        assert!(trace_store.is_ok());
        let trace_store = trace_store.unwrap_or_else(|_| unreachable!());
        assert!(trace_store.migrate().is_ok());

        let workflow_yaml = r#"
workflow_name: wf
workflow_version: v1
normalization_version: 0
agents:
  - agent_name: planner
    role: planning
    provider:
      provider_name: mock
      model_id: x
steps:
  - step_key: step_a
    agent_name: planner
    task: { text: "a" }
    depends_on: []
    gate_points: []
  - step_key: step_b
    agent_name: planner
    task: { text: "b" }
    depends_on: []
    gate_points: []
  - step_key: step_c
    agent_name: planner
    task: { text: "c" }
    depends_on: [step_a, step_b]
    gate_points: []
gates: []
defaults:
  non_interactive: true
"#;
        let workflow = normalize_workflow_yaml(workflow_yaml);
        assert!(workflow.is_ok());
        let workflow = workflow.unwrap_or_else(|_| unreachable!());

        let context_source = super::StaticContextPackageSource::default();
        let trust_source = AllowAllTrustGateSource;
        let human_gate = DefaultHumanGateDecider;
        let write_applier = NoopProposedWriteApplier;
        let summary = Orchestrator::new(
            &trace_store,
            &context_source,
            &trust_source,
            &human_gate,
            &write_applier,
        )
        .execute_workflow(
            &workflow,
            RunConfig {
                non_interactive: true,
                ..RunConfig::default()
            },
        );
        assert!(summary.is_ok());
        let summary = summary.unwrap_or_else(|_| unreachable!());
        assert_eq!(
            summary.status,
            multi_agent_center_domain::RunStatus::Succeeded
        );
        assert_eq!(summary.steps_total, 3);
        assert_eq!(summary.steps_succeeded, 3);
    }
}
