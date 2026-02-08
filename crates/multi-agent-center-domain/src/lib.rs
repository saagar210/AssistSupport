#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use memory_kernel_core::{ContextPackage, MemoryId, MemoryVersionId, RecordType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use ulid::Ulid;

pub type DateTimeUtc = OffsetDateTime;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RunId(pub Ulid);

impl RunId {
    #[must_use]
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for RunId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StepId(pub Ulid);

impl StepId {
    #[must_use]
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for StepId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for StepId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Rejected,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GateKind {
    Human,
    Trust,
    Policy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GateDecision {
    Approved,
    Rejected,
    Pruned,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TraceEventType {
    RunStarted,
    RunFinished,
    WorkflowNormalized,
    StepReady,
    StepStarted,
    StepInputPrepared,
    StepPermissionPruned,
    GateEvaluated,
    ProviderCalled,
    StepFinished,
    ProposedMemoryWrite,
    ReplayStarted,
    ReplayFinished,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProviderBinding {
    pub provider_name: String,
    pub model_id: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgentPermissions {
    #[serde(default)]
    pub allowed_record_types: Vec<RecordType>,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    pub max_context_items: Option<u32>,
    #[serde(default)]
    pub can_propose_memory_writes: bool,
    #[serde(default)]
    pub fail_on_permission_prune: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgentDefinition {
    pub agent_name: String,
    pub role: String,
    pub provider: ProviderBinding,
    #[serde(default)]
    pub permissions: AgentPermissions,
    #[serde(default)]
    pub default_instructions: Vec<String>,
    #[serde(default)]
    pub metadata: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StepConstraints {
    #[serde(default)]
    pub max_output_tokens: Option<u32>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GatePointDefinition {
    pub gate_name: String,
    pub gate_kind: GateKind,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowStepDefinition {
    pub step_key: String,
    pub agent_name: String,
    #[serde(default)]
    pub task: Value,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub gate_points: Vec<String>,
    #[serde(default)]
    pub constraints: StepConstraints,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowDefaults {
    #[serde(default)]
    pub non_interactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NormalizedWorkflow {
    pub workflow_name: String,
    pub workflow_version: String,
    pub normalization_version: u32,
    pub agents: Vec<AgentDefinition>,
    pub steps: Vec<WorkflowStepDefinition>,
    #[serde(default)]
    pub gates: Vec<GatePointDefinition>,
    #[serde(default)]
    pub defaults: WorkflowDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct NormalizedWorkflowEnvelope {
    pub source_format: String,
    pub source_yaml_hash: String,
    pub normalized_hash: String,
    pub normalized_workflow: NormalizedWorkflow,
    pub normalized_json: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextPackageEnvelope {
    pub package_slot: usize,
    pub source: String,
    pub context_package: ContextPackage,
    pub package_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrustGateAttachment {
    pub memory_id: MemoryId,
    pub version: u32,
    pub memory_version_id: MemoryVersionId,
    pub include: bool,
    pub trust_status: String,
    pub confidence_effective: f32,
    pub capped: bool,
    pub reason_codes: Vec<String>,
    pub ruleset_version: Option<u32>,
    pub evaluated_at: DateTimeUtc,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct EffectivePermissions {
    pub allowed_record_types: Vec<RecordType>,
    pub allowed_tools: Vec<String>,
    pub max_context_items: Option<u32>,
    pub can_propose_memory_writes: bool,
    pub fail_on_permission_prune: bool,
}

impl From<&AgentPermissions> for EffectivePermissions {
    fn from(value: &AgentPermissions) -> Self {
        Self {
            allowed_record_types: value.allowed_record_types.clone(),
            allowed_tools: value.allowed_tools.clone(),
            max_context_items: value.max_context_items,
            can_propose_memory_writes: value.can_propose_memory_writes,
            fail_on_permission_prune: value.fail_on_permission_prune,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepRequest {
    pub run_id: RunId,
    pub step_id: StepId,
    pub step_key: String,
    pub as_of: DateTimeUtc,
    pub agent: AgentDefinition,
    pub task_payload: Value,
    pub injected_context_packages: Vec<ContextPackageEnvelope>,
    pub trust_gate_attachments: Vec<TrustGateAttachment>,
    pub effective_permissions: EffectivePermissions,
    pub constraints: StepConstraints,
    pub input_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProposedMemoryWrite {
    pub proposal_index: usize,
    pub payload: Value,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepOutputEnvelope {
    pub message: String,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorEnvelope {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateDecisionRecord {
    pub gate_kind: GateKind,
    pub gate_name: String,
    pub subject_type: String,
    pub memory_id: Option<MemoryId>,
    pub version: Option<u32>,
    pub memory_version_id: Option<MemoryVersionId>,
    pub decision: GateDecision,
    pub reason_codes: Vec<String>,
    pub notes: Option<String>,
    pub decided_by: String,
    pub decided_at: DateTimeUtc,
    pub source_ruleset_version: Option<u32>,
    pub evidence_json: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderCallRecord {
    pub provider_call_id: Ulid,
    pub provider_name: String,
    pub adapter_version: String,
    pub model_id: String,
    pub request_json: Value,
    pub request_hash: String,
    pub response_json: Value,
    pub response_hash: String,
    pub latency_ms: Option<u64>,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub started_at: DateTimeUtc,
    pub ended_at: DateTimeUtc,
    pub status: String,
    pub error_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepResult {
    pub run_id: RunId,
    pub step_id: StepId,
    pub status: StepStatus,
    pub outputs: StepOutputEnvelope,
    pub proposed_memory_writes: Vec<ProposedMemoryWrite>,
    pub provider_calls: Vec<ProviderCallRecord>,
    pub gate_decisions: Vec<GateDecisionRecord>,
    pub output_hash: String,
    pub error: Option<ErrorEnvelope>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct TraceEvent {
    pub event_id: Ulid,
    pub run_id: RunId,
    pub step_id: Option<StepId>,
    pub event_type: TraceEventType,
    pub occurred_at: DateTimeUtc,
    pub recorded_at: DateTimeUtc,
    pub actor_type: String,
    pub actor_id: String,
    pub payload_json: Value,
    pub payload_hash: String,
    pub prev_event_hash: Option<String>,
    pub event_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct RunRecord {
    pub run_id: RunId,
    pub workflow_name: String,
    pub workflow_version: String,
    pub workflow_hash: String,
    pub as_of: DateTimeUtc,
    pub as_of_was_default: bool,
    pub started_at: DateTimeUtc,
    pub ended_at: Option<DateTimeUtc>,
    pub status: RunStatus,
    pub replay_of_run_id: Option<RunId>,
    pub external_correlation_id: Option<String>,
    pub engine_version: String,
    pub cli_args_json: Value,
    pub manifest_hash: Option<String>,
    pub manifest_signature: Option<String>,
    pub manifest_signature_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepRecord {
    pub step_id: StepId,
    pub run_id: RunId,
    pub step_index: usize,
    pub step_key: String,
    pub agent_name: String,
    pub status: StepStatus,
    pub started_at: Option<DateTimeUtc>,
    pub ended_at: Option<DateTimeUtc>,
    pub task_payload_json: Value,
    pub constraints_json: Value,
    pub permissions_json: Value,
    pub input_hash: String,
    pub output_hash: Option<String>,
    pub error_json: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct EventRow {
    pub event_seq: i64,
    pub event: TraceEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct WorkflowSnapshotRecord {
    pub workflow_hash: String,
    pub normalization_version: u32,
    pub source_format: String,
    pub source_yaml_hash: String,
    pub normalized_json: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepContextPackageRecord {
    pub step_key: String,
    pub envelope: ContextPackageEnvelope,
}

#[must_use]
pub fn now_utc() -> DateTimeUtc {
    OffsetDateTime::now_utc()
}

#[must_use]
pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Hash a JSON value with stable `serde_json` serialization + SHA-256.
///
/// # Errors
/// Returns an error if JSON serialization fails.
pub fn hash_json(value: &Value) -> Result<String> {
    let bytes = serde_json::to_vec(value)?;
    Ok(hash_bytes(&bytes))
}

/// Ensure a string field is non-empty after trimming.
///
/// # Errors
/// Returns an error when the provided value is empty/whitespace.
pub fn ensure_non_empty(field_name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{field_name} MUST be non-empty"));
    }
    Ok(())
}

/// Compute a deterministic hash for a fully formed step request envelope.
///
/// # Errors
/// Returns an error if the request cannot be serialized.
pub fn compute_step_request_hash(request: &StepRequest) -> Result<String> {
    let value = serde_json::to_value(request)?;
    hash_json(&value)
}

/// Compute a deterministic hash for a fully formed step result envelope.
///
/// # Errors
/// Returns an error if the result cannot be serialized.
pub fn compute_step_result_hash(result: &StepResult) -> Result<String> {
    let value = serde_json::to_value(result)?;
    hash_json(&value)
}

/// Compute a deterministic hash for a trace event envelope.
///
/// # Errors
/// Returns an error if the event cannot be serialized.
pub fn compute_event_hash(event: &TraceEvent) -> Result<String> {
    let value = serde_json::to_value(event)?;
    hash_json(&value)
}
