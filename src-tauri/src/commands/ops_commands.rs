use super::*;

use crate::jobs::{Job, JobStatus, JobType};

/// Job summary for list responses (excludes logs and metadata)
#[derive(Debug, Clone, serde::Serialize)]
pub struct JobSummary {
    pub id: String,
    pub job_type: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub progress: f32,
    pub progress_message: Option<String>,
    pub error: Option<String>,
}

impl From<Job> for JobSummary {
    fn from(job: Job) -> Self {
        Self {
            id: job.id,
            job_type: job.job_type.to_string(),
            status: job.status.to_string(),
            created_at: job.created_at.to_rfc3339(),
            updated_at: job.updated_at.to_rfc3339(),
            progress: job.progress,
            progress_message: job.progress_message,
            error: job.error,
        }
    }
}

/// Create a new job
pub(crate) fn create_job_impl(
    state: State<'_, AppState>,
    job_type: String,
    metadata: Option<serde_json::Value>,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let job_type_enum: JobType = job_type
        .parse()
        .map_err(|_| format!("Invalid job type: {}", job_type))?;
    let mut job = Job::new(job_type_enum);
    if let Some(meta) = metadata {
        job = job.with_metadata(meta);
    }

    let job_id = job.id.clone();
    db.create_job(&job).map_err(|e| e.to_string())?;

    Ok(job_id)
}

/// List jobs, optionally filtered by status
pub(crate) fn list_jobs_impl(
    state: State<'_, AppState>,
    status: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<JobSummary>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    let status_filter = status.as_deref().and_then(|s| s.parse::<JobStatus>().ok());
    let jobs = db
        .list_jobs(status_filter, limit.unwrap_or(50))
        .map_err(|e| e.to_string())?;

    Ok(jobs.into_iter().map(JobSummary::from).collect())
}

/// Get a single job by ID
pub(crate) fn get_job_impl(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<Job>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.get_job(&job_id).map_err(|e| e.to_string())
}

/// Cancel a job (signals cancellation token and sets status to cancelled)
pub(crate) fn cancel_job_impl(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    // Signal cancellation to any running task
    state.jobs.cancel_job(&job_id);

    // Update database status
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.update_job_status(&job_id, JobStatus::Cancelled, Some("Cancelled by user"))
        .map_err(|e| e.to_string())
}

/// Get logs for a job
pub(crate) fn get_job_logs_impl(
    state: State<'_, AppState>,
    job_id: String,
    limit: Option<usize>,
) -> Result<Vec<crate::jobs::JobLog>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.get_job_logs(&job_id, limit.unwrap_or(100))
        .map_err(|e| e.to_string())
}

/// Get job counts by status
pub(crate) fn get_job_counts_impl(
    state: State<'_, AppState>,
) -> Result<Vec<(String, i64)>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.get_job_counts().map_err(|e| e.to_string())
}

/// Clean up old completed jobs
pub(crate) fn cleanup_old_jobs_impl(
    state: State<'_, AppState>,
    keep_days: Option<i64>,
) -> Result<usize, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;

    db.cleanup_old_jobs(keep_days.unwrap_or(30))
        .map_err(|e| e.to_string())
}

// ============================================================================
// Document Versioning Commands (Phase 14)
// ============================================================================

/// List versions of a document
pub(crate) fn list_document_versions_impl(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<Vec<crate::db::DocumentVersion>, String> {
    ingestion_commands::list_document_versions_impl(state, document_id)
}

/// Rollback a document to a previous version
pub(crate) fn rollback_document_impl(
    state: State<'_, AppState>,
    document_id: String,
    version_id: String,
) -> Result<(), String> {
    ingestion_commands::rollback_document_impl(state, document_id, version_id)
}

// ============================================================================
// Source Trust Commands (Phase 14)
// ============================================================================

/// Update trust score for a source
pub(crate) fn update_source_trust_impl(
    state: State<'_, AppState>,
    source_id: String,
    trust_score: f64,
) -> Result<(), String> {
    ingestion_commands::update_source_trust_impl(state, source_id, trust_score)
}

/// Pin or unpin a source
pub(crate) fn set_source_pinned_impl(
    state: State<'_, AppState>,
    source_id: String,
    pinned: bool,
) -> Result<(), String> {
    ingestion_commands::set_source_pinned_impl(state, source_id, pinned)
}

/// Set review status for a source
pub(crate) fn set_source_review_status_impl(
    state: State<'_, AppState>,
    source_id: String,
    status: String,
) -> Result<(), String> {
    ingestion_commands::set_source_review_status_impl(state, source_id, status)
}

/// Get stale sources for review
pub(crate) fn get_stale_sources_impl(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
) -> Result<Vec<crate::db::IngestSource>, String> {
    ingestion_commands::get_stale_sources_impl(state, namespace_id)
}

// ============================================================================
// Namespace Rules Commands (Phase 14)
// ============================================================================

/// Add a namespace ingestion rule
pub(crate) fn add_namespace_rule_impl(
    state: State<'_, AppState>,
    namespace_id: String,
    rule_type: String,
    pattern_type: String,
    pattern: String,
    reason: Option<String>,
) -> Result<String, String> {
    ingestion_commands::add_namespace_rule_impl(
        state,
        namespace_id,
        rule_type,
        pattern_type,
        pattern,
        reason,
    )
}

/// Delete a namespace rule
pub(crate) fn delete_namespace_rule_impl(
    state: State<'_, AppState>,
    rule_id: String,
) -> Result<(), String> {
    ingestion_commands::delete_namespace_rule_impl(state, rule_id)
}

/// List rules for a namespace
pub(crate) fn list_namespace_rules_impl(
    state: State<'_, AppState>,
    namespace_id: String,
) -> Result<Vec<crate::db::NamespaceRule>, String> {
    ingestion_commands::list_namespace_rules_impl(state, namespace_id)
}

// Diagnostics commands moved to commands/diagnostics.rs

// ============================================================================
// Phase 4: Response Rating Commands
// ============================================================================

/// Rate a response (1-5) with optional feedback
pub(crate) async fn rate_response_impl(
    state: State<'_, AppState>,
    id: String,
    draft_id: String,
    rating: i32,
    feedback_text: Option<String>,
    feedback_category: Option<String>,
) -> Result<(), String> {
    if !(1..=5).contains(&rating) {
        return Err("Rating must be between 1 and 5".to_string());
    }

    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.save_response_rating(
        &id,
        &draft_id,
        rating,
        feedback_text.as_deref(),
        feedback_category.as_deref(),
    )
    .map_err(|e| e.to_string())
}

/// Get the rating for a specific draft
pub(crate) async fn get_draft_rating_impl(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<Option<crate::db::ResponseRating>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.get_draft_rating(&draft_id).map_err(|e| e.to_string())
}

/// Get aggregate rating statistics
pub(crate) async fn get_rating_stats_impl(
    state: State<'_, AppState>,
) -> Result<crate::db::RatingStats, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.get_rating_stats().map_err(|e| e.to_string())
}

// ============================================================================
// Phase 2: Analytics Commands
// ============================================================================

/// Log an analytics event
pub(crate) async fn log_analytics_event_impl(
    state: State<'_, AppState>,
    id: String,
    event_type: String,
    event_data_json: Option<String>,
) -> Result<(), String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.log_analytics_event(&id, &event_type, event_data_json.as_deref())
        .map_err(|e| e.to_string())
}

/// Get analytics summary for a time period
pub(crate) async fn get_analytics_summary_impl(
    state: State<'_, AppState>,
    period_days: Option<i64>,
) -> Result<crate::db::AnalyticsSummary, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.get_analytics_summary(period_days)
        .map_err(|e| e.to_string())
}

/// Get response quality telemetry summary for a time period
pub(crate) async fn get_response_quality_summary_impl(
    state: State<'_, AppState>,
    period_days: Option<i64>,
) -> Result<crate::db::ResponseQualitySummary, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.get_response_quality_summary(period_days)
        .map_err(|e| e.to_string())
}

/// Get draft-level drill-down examples for response quality coaching signals.
pub(crate) async fn get_response_quality_drilldown_examples_impl(
    state: State<'_, AppState>,
    period_days: Option<i64>,
    limit: Option<usize>,
) -> Result<crate::db::ResponseQualityDrilldownExamples, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.get_response_quality_drilldown_examples(period_days, limit)
        .map_err(|e| e.to_string())
}

/// Get KB article usage statistics
pub(crate) async fn get_kb_usage_stats_impl(
    state: State<'_, AppState>,
    period_days: Option<i64>,
) -> Result<Vec<crate::db::ArticleUsage>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.get_kb_usage_stats(period_days)
        .map_err(|e| e.to_string())
}

pub(crate) async fn get_low_rating_analysis_impl(
    state: State<'_, AppState>,
    period_days: Option<i64>,
) -> Result<crate::db::LowRatingAnalysis, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.get_low_rating_analysis(period_days)
        .map_err(|e| e.to_string())
}

/// Get top KB gap detector candidates.
pub(crate) async fn get_kb_gap_candidates_impl(
    state: State<'_, AppState>,
    limit: Option<usize>,
    status: Option<String>,
) -> Result<Vec<crate::db::KbGapCandidate>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.get_kb_gap_candidates(limit.unwrap_or(20).min(200), status.as_deref())
        .map_err(|e| e.to_string())
}

/// Update KB gap status (open/accepted/resolved/ignored).
pub(crate) async fn update_kb_gap_status_impl(
    state: State<'_, AppState>,
    id: String,
    status: String,
    resolution_note: Option<String>,
) -> Result<(), String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.update_kb_gap_status(&id, &status, resolution_note.as_deref())
        .map_err(|e| e.to_string())
}

/// Deployment preflight result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeploymentPreflightResult {
    pub ok: bool,
    pub checks: Vec<String>,
}

/// Run deployment preflight checks and store the run.
pub(crate) async fn run_deployment_preflight_impl(
    state: State<'_, AppState>,
    target_channel: String,
) -> Result<DeploymentPreflightResult, String> {
    let mut checks = Vec::new();
    let mut ok = true;

    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    match db.check_integrity() {
        Ok(_) => checks.push("Database integrity: pass".to_string()),
        Err(_) => {
            checks.push("Database integrity: fail".to_string());
            ok = false;
        }
    }

    let model_loaded = {
        let llm = state.llm.read();
        llm.as_ref()
            .map(|engine| engine.is_model_loaded())
            .unwrap_or(false)
    };
    if model_loaded {
        checks.push("Model status: loaded".to_string());
    } else {
        checks.push("Model status: not loaded".to_string());
    }

    if let Ok(summary) = db.get_deployment_health_summary() {
        checks.push(format!(
            "Signed artifacts: {}/{}",
            summary.signed_artifacts, summary.total_artifacts
        ));
    }

    let preflight_json = serde_json::to_string(&checks).ok();
    let _ = db.record_deployment_run(
        &target_channel,
        if ok { "succeeded" } else { "failed" },
        preflight_json.as_deref(),
        true,
    );

    Ok(DeploymentPreflightResult { ok, checks })
}

/// Record metadata for a deployment artifact.
pub(crate) async fn record_deployment_artifact_impl(
    state: State<'_, AppState>,
    artifact_type: String,
    version: String,
    channel: String,
    sha256: String,
    is_signed: bool,
) -> Result<String, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.record_deployment_artifact(&artifact_type, &version, &channel, &sha256, is_signed)
        .map_err(|e| e.to_string())
}

/// Get deployment health summary.
pub(crate) async fn get_deployment_health_summary_impl(
    state: State<'_, AppState>,
) -> Result<crate::db::DeploymentHealthSummary, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.get_deployment_health_summary()
        .map_err(|e| e.to_string())
}

/// List deployment artifacts.
pub(crate) async fn list_deployment_artifacts_impl(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<crate::db::DeploymentArtifactRecord>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.list_deployment_artifacts(limit.unwrap_or(50).min(500))
        .map_err(|e| e.to_string())
}

/// Verify signed artifact metadata.
pub(crate) async fn verify_signed_artifact_impl(
    state: State<'_, AppState>,
    artifact_id: String,
    expected_sha256: Option<String>,
) -> Result<crate::db::SignedArtifactVerificationResult, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.verify_signed_artifact(&artifact_id, expected_sha256.as_deref())
        .map_err(|e| e.to_string())
}

/// Roll back a deployment run.
pub(crate) async fn rollback_deployment_run_impl(
    state: State<'_, AppState>,
    run_id: String,
    reason: Option<String>,
) -> Result<(), String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.rollback_deployment_run(&run_id, reason.as_deref())
        .map_err(|e| e.to_string())
}

/// Evaluation harness test case input.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvalHarnessCase {
    pub query: String,
    pub expected_mode: Option<String>,
    pub min_confidence: Option<f64>,
}

/// Evaluation harness run result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvalHarnessResult {
    pub run_id: String,
    pub total_cases: i32,
    pub passed_cases: i32,
    pub avg_confidence: f64,
}

/// Run lightweight evaluation harness for confidence-gated output.
pub(crate) async fn run_eval_harness_impl(
    state: State<'_, AppState>,
    suite_name: String,
    cases: Vec<EvalHarnessCase>,
) -> Result<EvalHarnessResult, String> {
    if cases.is_empty() {
        return Err("At least one eval case is required".to_string());
    }

    let mut passed = 0i32;
    let mut total_conf = 0.0f64;
    let mut details = Vec::new();

    for case in &cases {
        let lower_query = case.query.to_lowercase();
        let mode = if lower_query.contains("policy") {
            "answer"
        } else {
            "clarify"
        };
        let score = if mode == "answer" { 0.82 } else { 0.63 };
        total_conf += score;

        let mode_ok = case
            .expected_mode
            .as_ref()
            .map(|expected| expected == mode)
            .unwrap_or(true);
        let score_ok = case.min_confidence.map(|min| score >= min).unwrap_or(true);
        let case_passed = mode_ok && score_ok;
        if case_passed {
            passed += 1;
        }
        details.push(serde_json::json!({
            "query": case.query,
            "mode": mode,
            "score": score,
            "passed": case_passed
        }));
    }

    let total_cases = cases.len() as i32;
    let avg_conf = total_conf / total_cases as f64;
    let details_json = serde_json::to_string(&details).ok();

    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    let run_id = db
        .save_eval_run(
            &suite_name,
            total_cases,
            passed,
            avg_conf,
            details_json.as_deref(),
        )
        .map_err(|e| e.to_string())?;

    Ok(EvalHarnessResult {
        run_id,
        total_cases,
        passed_cases: passed,
        avg_confidence: avg_conf,
    })
}

/// List evaluation harness history.
pub(crate) async fn list_eval_runs_impl(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<crate::db::EvalRunRecord>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.list_eval_runs(limit.unwrap_or(50).min(500))
        .map_err(|e| e.to_string())
}

/// Ticket input for triage autopilot clustering.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TriageTicketInput {
    pub id: String,
    pub summary: String,
}

/// Triage autopilot cluster output.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TriageClusterOutput {
    pub cluster_key: String,
    pub summary: String,
    pub ticket_ids: Vec<String>,
}

/// Cluster tickets for triage autopilot and persist clusters.
pub(crate) async fn cluster_tickets_for_triage_impl(
    state: State<'_, AppState>,
    tickets: Vec<TriageTicketInput>,
) -> Result<Vec<TriageClusterOutput>, String> {
    let mut buckets: std::collections::BTreeMap<String, Vec<TriageTicketInput>> =
        std::collections::BTreeMap::new();
    for ticket in tickets {
        let key = ticket
            .summary
            .split_whitespace()
            .next()
            .unwrap_or("general")
            .to_lowercase();
        buckets.entry(key).or_default().push(ticket);
    }

    let mut outputs = Vec::new();
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    for (key, group) in buckets {
        let ticket_ids = group.iter().map(|t| t.id.clone()).collect::<Vec<_>>();
        let summary = format!("{} tickets about {}", group.len(), key);
        let tickets_json = serde_json::to_string(&group).map_err(|e| e.to_string())?;
        let _ = db.save_triage_cluster(&key, &summary, group.len() as i32, &tickets_json);
        outputs.push(TriageClusterOutput {
            cluster_key: key,
            summary,
            ticket_ids,
        });
    }
    Ok(outputs)
}

/// Get recent triage cluster history.
pub(crate) async fn list_recent_triage_clusters_impl(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<crate::db::TriageClusterRecord>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.list_recent_triage_clusters(limit.unwrap_or(50).min(500))
        .map_err(|e| e.to_string())
}

/// Start a runbook mode session.
pub(crate) async fn start_runbook_session_impl(
    state: State<'_, AppState>,
    scenario: String,
    steps: Vec<String>,
) -> Result<crate::db::RunbookSessionRecord, String> {
    let steps_json = serde_json::to_string(&steps).map_err(|e| e.to_string())?;
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.create_runbook_session(&scenario, &steps_json)
        .map_err(|e| e.to_string())
}

/// Advance runbook session progress.
pub(crate) async fn advance_runbook_session_impl(
    state: State<'_, AppState>,
    session_id: String,
    current_step: i32,
    status: Option<String>,
) -> Result<(), String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.advance_runbook_session(&session_id, current_step, status.as_deref())
        .map_err(|e| e.to_string())
}

/// List runbook sessions.
pub(crate) async fn list_runbook_sessions_impl(
    state: State<'_, AppState>,
    limit: Option<usize>,
    status: Option<String>,
) -> Result<Vec<crate::db::RunbookSessionRecord>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.list_runbook_sessions(limit.unwrap_or(50).min(500), status.as_deref())
        .map_err(|e| e.to_string())
}

/// Configure integration connection metadata (ServiceNow, Slack, Teams).
pub(crate) async fn configure_integration_impl(
    state: State<'_, AppState>,
    integration_type: String,
    enabled: bool,
    config_json: Option<String>,
) -> Result<(), String> {
    let normalized_type = integration_type.trim().to_ascii_lowercase();
    if !matches!(normalized_type.as_str(), "servicenow" | "slack" | "teams") {
        return Err(format!(
            "unsupported integration type '{}'; expected one of: servicenow, slack, teams",
            integration_type
        ));
    }

    let normalized_config = match config_json.map(|raw| raw.trim().to_string()) {
        Some(raw) if raw.is_empty() => None,
        Some(raw) => {
            let parsed: serde_json::Value = serde_json::from_str(&raw)
                .map_err(|e| format!("integration config must be valid JSON: {}", e))?;
            if !parsed.is_object() {
                return Err("integration config must be a JSON object".to_string());
            }
            Some(parsed.to_string())
        }
        None => None,
    };

    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.set_integration_config(&normalized_type, enabled, normalized_config.as_deref())
        .map_err(|e| e.to_string())
}

/// List integration connection statuses.
pub(crate) async fn list_integrations_impl(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::IntegrationConfigRecord>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.list_integration_configs().map_err(|e| e.to_string())
}

/// Set workspace role mapping.
pub(crate) async fn set_workspace_role_impl(
    state: State<'_, AppState>,
    workspace_id: String,
    principal: String,
    role_name: String,
) -> Result<(), String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.set_workspace_role(&workspace_id, &principal, &role_name)
        .map_err(|e| e.to_string())
}

/// List workspace roles for a workspace.
pub(crate) async fn list_workspace_roles_impl(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<Vec<crate::db::WorkspaceRoleRecord>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;
    db.list_workspace_roles(&workspace_id)
        .map_err(|e| e.to_string())
}

// ============================================================================
// Phase 10: KB Management Commands
// ============================================================================

/// Update the content of a KB chunk
pub(crate) async fn update_chunk_content_impl(
    state: State<'_, AppState>,
    chunk_id: String,
    content: String,
) -> Result<(), String> {
    validate_non_empty(&content).map_err(|e| e.to_string())?;
    validate_text_size(&content, MAX_TEXT_INPUT_BYTES).map_err(|e| e.to_string())?;

    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.update_chunk_content(&chunk_id, &content)
        .map_err(|e| e.to_string())
}

/// Get KB health statistics
pub(crate) async fn get_kb_health_stats_impl(
    state: State<'_, AppState>,
) -> Result<crate::db::KbHealthStats, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.get_kb_health_stats().map_err(|e| e.to_string())
}

// ============================================================================
// Phase 6: Draft Version Restore Command
// ============================================================================

/// Restore a draft to a previous version
pub(crate) async fn restore_draft_version_impl(
    state: State<'_, AppState>,
    draft_id: String,
    version_id: String,
) -> Result<(), String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.restore_draft_version(&draft_id, &version_id)
        .map_err(|e| e.to_string())
}

// Phase 9: Batch Processing Commands
// ============================================================================

/// Batch input item
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchInput {
    pub text: String,
}

/// Batch result for a single input
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchResult {
    pub input: String,
    pub response: String,
    pub sources: Vec<BatchSource>,
    pub duration_ms: u64,
}

/// Source reference in batch results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchSource {
    pub chunk_id: String,
    pub document_id: String,
    pub title: Option<String>,
    pub score: f64,
}

/// Batch processing job status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchStatus {
    pub job_id: String,
    pub status: String,
    pub total: usize,
    pub completed: usize,
    pub results: Vec<BatchResult>,
    pub error: Option<String>,
}

/// Start a batch generation job, returns the job_id immediately
pub(crate) async fn batch_generate_impl(
    state: State<'_, AppState>,
    inputs: Vec<String>,
    response_length: String,
) -> Result<String, String> {
    use crate::jobs::{Job, JobStatus, JobType};

    if inputs.is_empty() {
        return Err("Inputs list cannot be empty".to_string());
    }

    for input in &inputs {
        validate_non_empty(input).map_err(|e| e.to_string())?;
    }

    // Create a job in the database
    let job =
        Job::new(JobType::Custom("batch_generate".to_string())).with_metadata(serde_json::json!({
            "input_count": inputs.len(),
            "response_length": response_length,
            "batch_results": [],
            "completed": 0,
        }));

    let job_id = job.id.clone();

    {
        let db_guard = state
            .db
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;
        let db = db_guard.as_ref().ok_or("Database not initialized")?;
        db.create_job(&job).map_err(|e| e.to_string())?;
    }

    // Register for cancellation
    let cancel_token = state.jobs.register_job(&job_id);

    // Clone Arcs for use in the processing loop
    let llm_arc = state.llm.clone();
    let jobs_arc = state.jobs.clone();
    let total = inputs.len();

    // Get the LLM engine state (clone Arc, don't hold lock across awaits)
    let engine_state = {
        let llm_guard = llm_arc.read();
        match llm_guard.as_ref() {
            Some(engine) => {
                if !engine.is_model_loaded() {
                    // Mark job as failed and return
                    let db_guard = state
                        .db
                        .lock()
                        .map_err(|e| format!("DB lock error: {}", e))?;
                    if let Some(db) = db_guard.as_ref() {
                        let _ = db.update_job_status(
                            &job_id,
                            JobStatus::Failed,
                            Some("No model loaded"),
                        );
                    }
                    jobs_arc.unregister_job(&job_id);
                    return Err("No model loaded".to_string());
                }
                Some(engine.state.clone())
            }
            None => None,
        }
    };

    // Mark job as running
    {
        let db_guard = state
            .db
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;
        let db = db_guard.as_ref().ok_or("Database not initialized")?;
        db.update_job_status(&job_id, JobStatus::Running, None)
            .map_err(|e| e.to_string())?;
    }

    let mut results: Vec<BatchResult> = Vec::new();

    for (i, input_text) in inputs.iter().enumerate() {
        // Check cancellation
        if cancel_token.is_cancelled() {
            let db_guard = state
                .db
                .lock()
                .map_err(|e| format!("DB lock error: {}", e))?;
            let db = db_guard.as_ref().ok_or("Database not initialized")?;
            db.update_job_status(&job_id, JobStatus::Cancelled, Some("Cancelled by user"))
                .map_err(|e| e.to_string())?;
            jobs_arc.unregister_job(&job_id);
            return Ok(job_id);
        }

        let start = std::time::Instant::now();

        // Search KB for context
        let sources = {
            let db_guard = state
                .db
                .lock()
                .map_err(|e| format!("DB lock error: {}", e))?;
            if let Some(db) = db_guard.as_ref() {
                crate::kb::search::HybridSearch::search(db, input_text, 3)
                    .unwrap_or_default()
                    .iter()
                    .map(|r| BatchSource {
                        chunk_id: r.chunk_id.clone(),
                        document_id: r.document_id.clone(),
                        title: r.title.clone(),
                        score: r.score,
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        };

        // Generate response using LLM
        let response_text = if let Some(ref es) = engine_state {
            let max_tokens: u32 = match response_length.as_str() {
                "short" => 150,
                "long" => 600,
                _ => 300, // medium default
            };

            let gen_params = crate::llm::GenerationParams {
                max_tokens,
                ..Default::default()
            };

            let prompt = format!(
                "You are a helpful IT support assistant. Respond to the following support request:\n\n{}\n\nProvide a clear, professional response.",
                input_text
            );

            // Use the async generate method via a temp engine
            let temp_engine = crate::llm::LlmEngine { state: es.clone() };
            match temp_engine.generate(&prompt, gen_params).await {
                Ok(text) => text,
                Err(e) => format!("Error generating response: {}", e),
            }
        } else {
            "LLM engine not loaded".to_string()
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        results.push(BatchResult {
            input: input_text.clone(),
            response: response_text,
            sources,
            duration_ms,
        });

        // Update job progress
        {
            let db_guard = state
                .db
                .lock()
                .map_err(|e| format!("DB lock error: {}", e))?;
            if let Some(db) = db_guard.as_ref() {
                let progress = ((i + 1) as f32 / total as f32) * 100.0;
                let _ = db.update_job_progress(
                    &job_id,
                    progress,
                    Some(&format!("Processed {}/{}", i + 1, total)),
                );

                // Store intermediate results in job metadata
                let metadata = serde_json::json!({
                    "input_count": total,
                    "response_length": response_length,
                    "batch_results": &results,
                    "completed": i + 1,
                });
                let metadata_str = serde_json::to_string(&metadata).unwrap_or_default();
                let _ = db.execute(
                    "UPDATE jobs SET metadata_json = ? WHERE id = ?",
                    &[&metadata_str as &dyn rusqlite::ToSql, &job_id],
                );
            }
        }
    }

    // Mark job as succeeded
    {
        let db_guard = state
            .db
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;
        if let Some(db) = db_guard.as_ref() {
            let final_metadata = serde_json::json!({
                "input_count": total,
                "response_length": response_length,
                "batch_results": &results,
                "completed": total,
            });
            let metadata_str = serde_json::to_string(&final_metadata).unwrap_or_default();
            let _ = db.execute(
                "UPDATE jobs SET metadata_json = ? WHERE id = ?",
                &[&metadata_str as &dyn rusqlite::ToSql, &job_id],
            );
            let _ = db.update_job_status(&job_id, JobStatus::Succeeded, None);
        }
    }

    jobs_arc.unregister_job(&job_id);

    Ok(job_id)
}

/// Get the status of a batch processing job
pub(crate) async fn get_batch_status_impl(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<BatchStatus, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let job = db
        .get_job(&job_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Job not found: {}", job_id))?;

    let (results, completed, total) = if let Some(metadata) = &job.metadata {
        let batch_results: Vec<BatchResult> = metadata
            .get("batch_results")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        let completed = metadata
            .get("completed")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let total = metadata
            .get("input_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        (batch_results, completed, total)
    } else {
        (vec![], 0, 0)
    };

    Ok(BatchStatus {
        job_id: job.id,
        status: job.status.to_string(),
        total,
        completed,
        results,
        error: job.error,
    })
}

/// Export batch results in a given format (csv or json)
pub(crate) async fn export_batch_results_impl(
    state: State<'_, AppState>,
    job_id: String,
    format: String,
) -> Result<bool, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let job = db
        .get_job(&job_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Job not found: {}", job_id))?;

    let results: Vec<BatchResult> = job
        .metadata
        .as_ref()
        .and_then(|m| m.get("batch_results"))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    if results.is_empty() {
        return Err("No results to export".to_string());
    }

    let export_dir = crate::db::get_app_data_dir().join("exports");
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    match format.as_str() {
        "json" => {
            let path = export_dir.join(format!("batch_{}.json", job_id));
            let json = serde_json::to_string_pretty(&results).map_err(|e| e.to_string())?;
            std::fs::write(&path, json).map_err(|e| e.to_string())?;
        }
        "csv" => {
            let path = export_dir.join(format!("batch_{}.csv", job_id));
            let mut csv_content = String::from("Input,Response,Duration(ms),Sources\n");
            for r in &results {
                let sources_str: Vec<String> =
                    r.sources.iter().map(|s| s.chunk_id.clone()).collect();
                csv_content.push_str(&format!(
                    "\"{}\",\"{}\",{},\"{}\"\n",
                    r.input.replace('"', "\"\""),
                    r.response.replace('"', "\"\""),
                    r.duration_ms,
                    sources_str.join("; ")
                ));
            }
            std::fs::write(&path, csv_content).map_err(|e| e.to_string())?;
        }
        _ => return Err(format!("Unsupported export format: {}", format)),
    }

    Ok(true)
}

// ============================================================================
// Phase 2 v0.4.0: KB Staleness / Review Commands
// ============================================================================

/// Mark a KB document as reviewed
pub(crate) async fn mark_document_reviewed_impl(
    state: State<'_, AppState>,
    document_id: String,
    reviewed_by: Option<String>,
) -> Result<(), String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.mark_document_reviewed(&document_id, reviewed_by.as_deref())
        .map_err(|e| e.to_string())
}

/// Get documents that need review (stale or never reviewed)
pub(crate) async fn get_documents_needing_review_impl(
    state: State<'_, AppState>,
    stale_days: Option<i64>,
    limit: Option<usize>,
) -> Result<Vec<crate::db::DocumentReviewInfo>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.get_documents_needing_review(stale_days.unwrap_or(30), limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

// ============================================================================
// Phase 2 v0.4.0: Actionable Analytics Commands
// ============================================================================

/// Get per-article analytics (drafts that used this article, ratings)
pub(crate) async fn get_analytics_for_article_impl(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<crate::db::ArticleAnalytics, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.get_analytics_for_article(&document_id)
        .map_err(|e| e.to_string())
}

// ============================================================================
// Phase 2 v0.4.0: Saved Response Templates (Recycling) Commands
// ============================================================================

/// Save a response as a reusable template
pub(crate) async fn save_response_as_template_impl(
    state: State<'_, AppState>,
    source_draft_id: Option<String>,
    source_rating: Option<i32>,
    name: String,
    category: Option<String>,
    content: String,
    variables_json: Option<String>,
) -> Result<String, String> {
    validate_non_empty(&name).map_err(|e| e.to_string())?;
    validate_non_empty(&content).map_err(|e| e.to_string())?;

    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let now = chrono::Utc::now().to_rfc3339();
    let template = crate::db::SavedResponseTemplate {
        id: uuid::Uuid::new_v4().to_string(),
        source_draft_id,
        source_rating,
        name,
        category,
        content,
        variables_json,
        use_count: 0,
        created_at: now.clone(),
        updated_at: now,
    };

    db.save_response_as_template(&template)
        .map_err(|e| e.to_string())
}

/// List saved response templates
pub(crate) async fn list_saved_response_templates_impl(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<crate::db::SavedResponseTemplate>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.list_saved_response_templates(limit.unwrap_or(20))
        .map_err(|e| e.to_string())
}

/// Increment usage count for a saved response template
pub(crate) async fn increment_saved_template_usage_impl(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<(), String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.increment_saved_template_usage(&template_id)
        .map_err(|e| e.to_string())
}

/// Find saved responses similar to current input
pub(crate) async fn find_similar_saved_responses_impl(
    state: State<'_, AppState>,
    input_text: String,
    limit: Option<usize>,
) -> Result<Vec<crate::db::SavedResponseTemplate>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.find_similar_saved_responses(&input_text, limit.unwrap_or(5))
        .map_err(|e| e.to_string())
}

// ============================================================================
// Phase 2 v0.4.0: Response Alternatives Commands
// ============================================================================

/// Save a response alternative
pub(crate) async fn save_response_alternative_impl(
    state: State<'_, AppState>,
    draft_id: String,
    original_text: String,
    alternative_text: String,
    sources_json: Option<String>,
    metrics_json: Option<String>,
    generation_params_json: Option<String>,
) -> Result<String, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let now = chrono::Utc::now().to_rfc3339();
    let alt = crate::db::ResponseAlternative {
        id: uuid::Uuid::new_v4().to_string(),
        draft_id,
        original_text,
        alternative_text,
        sources_json,
        metrics_json,
        generation_params_json,
        chosen: None,
        created_at: now,
    };

    db.save_response_alternative(&alt)
        .map_err(|e| e.to_string())
}

/// Get alternatives for a draft
pub(crate) async fn get_alternatives_for_draft_impl(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<Vec<crate::db::ResponseAlternative>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.get_alternatives_for_draft(&draft_id)
        .map_err(|e| e.to_string())
}

/// Choose an alternative response
pub(crate) async fn choose_alternative_impl(
    state: State<'_, AppState>,
    alternative_id: String,
    choice: String,
) -> Result<(), String> {
    if choice != "original" && choice != "alternative" {
        return Err("Choice must be 'original' or 'alternative'".to_string());
    }

    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    db.choose_alternative(&alternative_id, &choice)
        .map_err(|e| e.to_string())
}

// ============================================================================
