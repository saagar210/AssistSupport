#![forbid(unsafe_code)]

use anyhow::Result;
use multi_agent_center_domain::{
    ContextPackageEnvelope, EventRow, GateDecisionRecord, ProposedMemoryWrite, ProviderCallRecord,
    RunId, RunRecord, RunStatus, StepContextPackageRecord, StepId, StepRecord, StepStatus,
    TraceEvent, WorkflowSnapshotRecord,
};

pub trait TraceStore {
    #[allow(clippy::missing_errors_doc)]
    fn migrate(&self) -> Result<()>;

    #[allow(clippy::missing_errors_doc)]
    fn upsert_workflow_snapshot(
        &self,
        workflow_hash: &str,
        normalization_version: u32,
        source_format: &str,
        source_yaml_hash: &str,
        normalized_json: &serde_json::Value,
    ) -> Result<()>;

    #[allow(clippy::missing_errors_doc)]
    fn insert_run(&self, run: &RunRecord) -> Result<()>;

    #[allow(clippy::missing_errors_doc)]
    fn update_run_finished(&self, run_id: RunId, status: RunStatus) -> Result<()>;

    #[allow(clippy::missing_errors_doc)]
    fn update_run_manifest(
        &self,
        run_id: RunId,
        manifest_hash: &str,
        manifest_signature: Option<&str>,
        manifest_signature_status: &str,
    ) -> Result<()>;

    #[allow(clippy::missing_errors_doc)]
    fn insert_step(&self, step: &StepRecord) -> Result<()>;

    #[allow(clippy::missing_errors_doc)]
    fn update_step_status(
        &self,
        step_id: StepId,
        status: StepStatus,
        output_hash: Option<&str>,
        error_json: Option<&serde_json::Value>,
    ) -> Result<()>;

    #[allow(clippy::missing_errors_doc)]
    fn append_event(&self, event: &TraceEvent) -> Result<i64>;

    #[allow(clippy::missing_errors_doc)]
    fn append_context_package(
        &self,
        run_id: RunId,
        step_id: StepId,
        envelope: &ContextPackageEnvelope,
    ) -> Result<()>;

    #[allow(clippy::missing_errors_doc)]
    fn append_gate_decision(
        &self,
        run_id: RunId,
        step_id: StepId,
        decision: &GateDecisionRecord,
    ) -> Result<()>;

    #[allow(clippy::missing_errors_doc)]
    fn append_provider_call(
        &self,
        run_id: RunId,
        step_id: StepId,
        call: &ProviderCallRecord,
    ) -> Result<()>;

    #[allow(clippy::missing_errors_doc)]
    fn append_proposed_memory_write(
        &self,
        run_id: RunId,
        step_id: StepId,
        write: &ProposedMemoryWrite,
        disposition: &str,
        disposition_reason: Option<&str>,
    ) -> Result<()>;

    #[allow(clippy::missing_errors_doc)]
    fn list_runs(&self) -> Result<Vec<RunRecord>>;

    #[allow(clippy::missing_errors_doc)]
    fn list_events_for_run(&self, run_id: RunId) -> Result<Vec<EventRow>>;

    #[allow(clippy::missing_errors_doc)]
    fn get_run(&self, run_id: RunId) -> Result<Option<RunRecord>>;

    #[allow(clippy::missing_errors_doc)]
    fn get_step_records(&self, run_id: RunId) -> Result<Vec<StepRecord>>;

    #[allow(clippy::missing_errors_doc)]
    fn get_workflow_snapshot(&self, workflow_hash: &str) -> Result<Option<WorkflowSnapshotRecord>>;

    #[allow(clippy::missing_errors_doc)]
    fn get_step_context_packages(&self, run_id: RunId) -> Result<Vec<StepContextPackageRecord>>;
}
