use super::*;

/// Create a new job
#[tauri::command]
pub fn create_job(
    state: State<'_, AppState>,
    job_type: String,
    metadata: Option<serde_json::Value>,
) -> Result<String, String> {
    ops_commands::create_job_impl(state, job_type, metadata)
}

/// List jobs, optionally filtered by status
#[tauri::command]
pub fn list_jobs(
    state: State<'_, AppState>,
    status: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<JobSummary>, String> {
    ops_commands::list_jobs_impl(state, status, limit)
}

/// Get a single job by ID
#[tauri::command]
pub fn get_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<crate::jobs::Job>, String> {
    ops_commands::get_job_impl(state, job_id)
}

/// Cancel a job (signals cancellation token and sets status to cancelled)
#[tauri::command]
pub fn cancel_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    ops_commands::cancel_job_impl(state, job_id)
}

/// Get logs for a job
#[tauri::command]
pub fn get_job_logs(
    state: State<'_, AppState>,
    job_id: String,
    limit: Option<usize>,
) -> Result<Vec<crate::jobs::JobLog>, String> {
    ops_commands::get_job_logs_impl(state, job_id, limit)
}

/// Get job counts by status
#[tauri::command]
pub fn get_job_counts(state: State<'_, AppState>) -> Result<Vec<(String, i64)>, String> {
    ops_commands::get_job_counts_impl(state)
}

/// Clean up old completed jobs
#[tauri::command]
pub fn cleanup_old_jobs(
    state: State<'_, AppState>,
    keep_days: Option<i64>,
) -> Result<usize, String> {
    ops_commands::cleanup_old_jobs_impl(state, keep_days)
}

// ============================================================================
// Document Versioning Commands (Phase 14)
// ============================================================================

/// List versions of a document
#[tauri::command]
pub fn list_document_versions(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<Vec<crate::db::DocumentVersion>, String> {
    ops_commands::list_document_versions_impl(state, document_id)
}

/// Rollback a document to a previous version
#[tauri::command]
pub fn rollback_document(
    state: State<'_, AppState>,
    document_id: String,
    version_id: String,
) -> Result<(), String> {
    ops_commands::rollback_document_impl(state, document_id, version_id)
}

// ============================================================================
// Source Trust Commands (Phase 14)
// ============================================================================

/// Update trust score for a source
#[tauri::command]
pub fn update_source_trust(
    state: State<'_, AppState>,
    source_id: String,
    trust_score: f64,
) -> Result<(), String> {
    ops_commands::update_source_trust_impl(state, source_id, trust_score)
}

/// Pin or unpin a source
#[tauri::command]
pub fn set_source_pinned(
    state: State<'_, AppState>,
    source_id: String,
    pinned: bool,
) -> Result<(), String> {
    ops_commands::set_source_pinned_impl(state, source_id, pinned)
}

/// Set review status for a source
#[tauri::command]
pub fn set_source_review_status(
    state: State<'_, AppState>,
    source_id: String,
    status: String,
) -> Result<(), String> {
    ops_commands::set_source_review_status_impl(state, source_id, status)
}

/// Get stale sources for review
#[tauri::command]
pub fn get_stale_sources(
    state: State<'_, AppState>,
    namespace_id: Option<String>,
) -> Result<Vec<crate::db::IngestSource>, String> {
    ops_commands::get_stale_sources_impl(state, namespace_id)
}

// ============================================================================
// Namespace Rules Commands (Phase 14)
// ============================================================================

/// Add a namespace ingestion rule
#[tauri::command]
pub fn add_namespace_rule(
    state: State<'_, AppState>,
    namespace_id: String,
    rule_type: String,
    pattern_type: String,
    pattern: String,
    reason: Option<String>,
) -> Result<String, String> {
    ops_commands::add_namespace_rule_impl(
        state,
        namespace_id,
        rule_type,
        pattern_type,
        pattern,
        reason,
    )
}

/// Delete a namespace rule
#[tauri::command]
pub fn delete_namespace_rule(state: State<'_, AppState>, rule_id: String) -> Result<(), String> {
    ops_commands::delete_namespace_rule_impl(state, rule_id)
}

/// List rules for a namespace
#[tauri::command]
pub fn list_namespace_rules(
    state: State<'_, AppState>,
    namespace_id: String,
) -> Result<Vec<crate::db::NamespaceRule>, String> {
    ops_commands::list_namespace_rules_impl(state, namespace_id)
}

// Diagnostics commands moved to commands/diagnostics.rs

// ============================================================================
// Phase 4: Response Rating Commands
// ============================================================================

/// Rate a response (1-5) with optional feedback
#[tauri::command]
pub async fn rate_response(
    state: State<'_, AppState>,
    id: String,
    draft_id: String,
    rating: i32,
    feedback_text: Option<String>,
    feedback_category: Option<String>,
) -> Result<(), String> {
    ops_commands::rate_response_impl(
        state,
        id,
        draft_id,
        rating,
        feedback_text,
        feedback_category,
    )
    .await
}

/// Get the rating for a specific draft
#[tauri::command]
pub async fn get_draft_rating(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<Option<crate::db::ResponseRating>, String> {
    ops_commands::get_draft_rating_impl(state, draft_id).await
}

/// Get aggregate rating statistics
#[tauri::command]
pub async fn get_rating_stats(
    state: State<'_, AppState>,
) -> Result<crate::db::RatingStats, String> {
    ops_commands::get_rating_stats_impl(state).await
}

// ============================================================================
// Phase 2: Analytics Commands
// ============================================================================

/// Log an analytics event
#[tauri::command]
pub async fn log_analytics_event(
    state: State<'_, AppState>,
    id: String,
    event_type: String,
    event_data_json: Option<String>,
) -> Result<(), String> {
    ops_commands::log_analytics_event_impl(state, id, event_type, event_data_json).await
}

/// Get analytics summary for a time period
#[tauri::command]
pub async fn get_analytics_summary(
    state: State<'_, AppState>,
    period_days: Option<i64>,
) -> Result<crate::db::AnalyticsSummary, String> {
    ops_commands::get_analytics_summary_impl(state, period_days).await
}

/// Get response quality telemetry summary for a time period
#[tauri::command]
pub async fn get_response_quality_summary(
    state: State<'_, AppState>,
    period_days: Option<i64>,
) -> Result<crate::db::ResponseQualitySummary, String> {
    ops_commands::get_response_quality_summary_impl(state, period_days).await
}

/// Get draft-level drill-down examples for response quality coaching signals.
#[tauri::command]
pub async fn get_response_quality_drilldown_examples(
    state: State<'_, AppState>,
    period_days: Option<i64>,
    limit: Option<usize>,
) -> Result<crate::db::ResponseQualityDrilldownExamples, String> {
    ops_commands::get_response_quality_drilldown_examples_impl(state, period_days, limit).await
}

/// Get KB article usage statistics
#[tauri::command]
pub async fn get_kb_usage_stats(
    state: State<'_, AppState>,
    period_days: Option<i64>,
) -> Result<Vec<crate::db::ArticleUsage>, String> {
    ops_commands::get_kb_usage_stats_impl(state, period_days).await
}

#[tauri::command]
pub async fn get_low_rating_analysis(
    state: State<'_, AppState>,
    period_days: Option<i64>,
) -> Result<crate::db::LowRatingAnalysis, String> {
    ops_commands::get_low_rating_analysis_impl(state, period_days).await
}

/// Get top KB gap detector candidates.
#[tauri::command]
pub async fn get_kb_gap_candidates(
    state: State<'_, AppState>,
    limit: Option<usize>,
    status: Option<String>,
) -> Result<Vec<crate::db::KbGapCandidate>, String> {
    ops_commands::get_kb_gap_candidates_impl(state, limit, status).await
}

/// Update KB gap status (open/accepted/resolved/ignored).
#[tauri::command]
pub async fn update_kb_gap_status(
    state: State<'_, AppState>,
    id: String,
    status: String,
    resolution_note: Option<String>,
) -> Result<(), String> {
    ops_commands::update_kb_gap_status_impl(state, id, status, resolution_note).await
}

/// Run deployment preflight checks and store the run.
#[tauri::command]
pub async fn run_deployment_preflight(
    state: State<'_, AppState>,
    target_channel: String,
) -> Result<DeploymentPreflightResult, String> {
    ops_commands::run_deployment_preflight_impl(state, target_channel).await
}

/// Record metadata for a deployment artifact.
#[tauri::command]
pub async fn record_deployment_artifact(
    state: State<'_, AppState>,
    artifact_type: String,
    version: String,
    channel: String,
    sha256: String,
    is_signed: bool,
) -> Result<String, String> {
    ops_commands::record_deployment_artifact_impl(
        state,
        artifact_type,
        version,
        channel,
        sha256,
        is_signed,
    )
    .await
}

/// Get deployment health summary.
#[tauri::command]
pub async fn get_deployment_health_summary(
    state: State<'_, AppState>,
) -> Result<crate::db::DeploymentHealthSummary, String> {
    ops_commands::get_deployment_health_summary_impl(state).await
}

/// List deployment artifacts.
#[tauri::command]
pub async fn list_deployment_artifacts(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<crate::db::DeploymentArtifactRecord>, String> {
    ops_commands::list_deployment_artifacts_impl(state, limit).await
}

/// Verify signed artifact metadata.
#[tauri::command]
pub async fn verify_signed_artifact(
    state: State<'_, AppState>,
    artifact_id: String,
    expected_sha256: Option<String>,
) -> Result<crate::db::SignedArtifactVerificationResult, String> {
    ops_commands::verify_signed_artifact_impl(state, artifact_id, expected_sha256).await
}

/// Roll back a deployment run.
#[tauri::command]
pub async fn rollback_deployment_run(
    state: State<'_, AppState>,
    run_id: String,
    reason: Option<String>,
) -> Result<(), String> {
    ops_commands::rollback_deployment_run_impl(state, run_id, reason).await
}

/// Run lightweight evaluation harness for confidence-gated output.
#[tauri::command]
pub async fn run_eval_harness(
    state: State<'_, AppState>,
    suite_name: String,
    cases: Vec<EvalHarnessCase>,
) -> Result<EvalHarnessResult, String> {
    ops_commands::run_eval_harness_impl(state, suite_name, cases).await
}

/// List evaluation harness history.
#[tauri::command]
pub async fn list_eval_runs(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<crate::db::EvalRunRecord>, String> {
    ops_commands::list_eval_runs_impl(state, limit).await
}

/// Cluster tickets for triage autopilot and persist clusters.
#[tauri::command]
pub async fn cluster_tickets_for_triage(
    state: State<'_, AppState>,
    tickets: Vec<TriageTicketInput>,
) -> Result<Vec<TriageClusterOutput>, String> {
    ops_commands::cluster_tickets_for_triage_impl(state, tickets).await
}

/// Get recent triage cluster history.
#[tauri::command]
pub async fn list_recent_triage_clusters(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<crate::db::TriageClusterRecord>, String> {
    ops_commands::list_recent_triage_clusters_impl(state, limit).await
}

/// Start a runbook mode session.
#[tauri::command]
pub async fn start_runbook_session(
    state: State<'_, AppState>,
    scenario: String,
    steps: Vec<String>,
) -> Result<crate::db::RunbookSessionRecord, String> {
    ops_commands::start_runbook_session_impl(state, scenario, steps).await
}

/// Advance runbook session progress.
#[tauri::command]
pub async fn advance_runbook_session(
    state: State<'_, AppState>,
    session_id: String,
    current_step: i32,
    status: Option<String>,
) -> Result<(), String> {
    ops_commands::advance_runbook_session_impl(state, session_id, current_step, status).await
}

/// List runbook sessions.
#[tauri::command]
pub async fn list_runbook_sessions(
    state: State<'_, AppState>,
    limit: Option<usize>,
    status: Option<String>,
) -> Result<Vec<crate::db::RunbookSessionRecord>, String> {
    ops_commands::list_runbook_sessions_impl(state, limit, status).await
}

/// Configure integration connection metadata (ServiceNow, Slack, Teams).
#[tauri::command]
pub async fn configure_integration(
    state: State<'_, AppState>,
    integration_type: String,
    enabled: bool,
    config_json: Option<String>,
) -> Result<(), String> {
    ops_commands::configure_integration_impl(state, integration_type, enabled, config_json).await
}

/// List integration connection statuses.
#[tauri::command]
pub async fn list_integrations(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::IntegrationConfigRecord>, String> {
    ops_commands::list_integrations_impl(state).await
}

/// Set workspace role mapping.
#[tauri::command]
pub async fn set_workspace_role(
    state: State<'_, AppState>,
    workspace_id: String,
    principal: String,
    role_name: String,
) -> Result<(), String> {
    ops_commands::set_workspace_role_impl(state, workspace_id, principal, role_name).await
}

/// List workspace roles for a workspace.
#[tauri::command]
pub async fn list_workspace_roles(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<Vec<crate::db::WorkspaceRoleRecord>, String> {
    ops_commands::list_workspace_roles_impl(state, workspace_id).await
}

// ============================================================================
// Phase 10: KB Management Commands
// ============================================================================

/// Update the content of a KB chunk
#[tauri::command]
pub async fn update_chunk_content(
    state: State<'_, AppState>,
    chunk_id: String,
    content: String,
) -> Result<(), String> {
    ops_commands::update_chunk_content_impl(state, chunk_id, content).await
}

/// Get KB health statistics
#[tauri::command]
pub async fn get_kb_health_stats(
    state: State<'_, AppState>,
) -> Result<crate::db::KbHealthStats, String> {
    ops_commands::get_kb_health_stats_impl(state).await
}

// ============================================================================
// Phase 6: Draft Version Restore Command
// ============================================================================

/// Restore a draft to a previous version
#[tauri::command]
pub async fn restore_draft_version(
    state: State<'_, AppState>,
    draft_id: String,
    version_id: String,
) -> Result<(), String> {
    ops_commands::restore_draft_version_impl(state, draft_id, version_id).await
}

// ============================================================================
// Phase 9: Batch Processing Commands
// ============================================================================

/// Start a batch generation job, returns the job_id immediately
#[tauri::command]
pub async fn batch_generate(
    state: State<'_, AppState>,
    inputs: Vec<String>,
    response_length: String,
) -> Result<String, String> {
    ops_commands::batch_generate_impl(state, inputs, response_length).await
}

/// Get the status of a batch processing job
#[tauri::command]
pub async fn get_batch_status(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<BatchStatus, String> {
    ops_commands::get_batch_status_impl(state, job_id).await
}

/// Export batch results in a given format (csv or json)
#[tauri::command]
pub async fn export_batch_results(
    state: State<'_, AppState>,
    job_id: String,
    format: String,
) -> Result<bool, String> {
    ops_commands::export_batch_results_impl(state, job_id, format).await
}

// ============================================================================
// Phase 2 v0.4.0: KB Staleness / Review Commands
// ============================================================================

/// Mark a KB document as reviewed
#[tauri::command]
pub async fn mark_document_reviewed(
    state: State<'_, AppState>,
    document_id: String,
    reviewed_by: Option<String>,
) -> Result<(), String> {
    ops_commands::mark_document_reviewed_impl(state, document_id, reviewed_by).await
}

/// Get documents that need review (stale or never reviewed)
#[tauri::command]
pub async fn get_documents_needing_review(
    state: State<'_, AppState>,
    stale_days: Option<i64>,
    limit: Option<usize>,
) -> Result<Vec<crate::db::DocumentReviewInfo>, String> {
    ops_commands::get_documents_needing_review_impl(state, stale_days, limit).await
}

// ============================================================================
// Phase 2 v0.4.0: Actionable Analytics Commands
// ============================================================================

/// Get per-article analytics (drafts that used this article, ratings)
#[tauri::command]
pub async fn get_analytics_for_article(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<crate::db::ArticleAnalytics, String> {
    ops_commands::get_analytics_for_article_impl(state, document_id).await
}

// ============================================================================
// Phase 2 v0.4.0: Saved Response Templates (Recycling) Commands
// ============================================================================

/// Save a response as a reusable template
#[tauri::command]
pub async fn save_response_as_template(
    state: State<'_, AppState>,
    source_draft_id: Option<String>,
    source_rating: Option<i32>,
    name: String,
    category: Option<String>,
    content: String,
    variables_json: Option<String>,
) -> Result<String, String> {
    ops_commands::save_response_as_template_impl(
        state,
        source_draft_id,
        source_rating,
        name,
        category,
        content,
        variables_json,
    )
    .await
}

/// List saved response templates
#[tauri::command]
pub async fn list_saved_response_templates(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<crate::db::SavedResponseTemplate>, String> {
    ops_commands::list_saved_response_templates_impl(state, limit).await
}

/// Increment usage count for a saved response template
#[tauri::command]
pub async fn increment_saved_template_usage(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<(), String> {
    ops_commands::increment_saved_template_usage_impl(state, template_id).await
}

/// Find saved responses similar to current input
#[tauri::command]
pub async fn find_similar_saved_responses(
    state: State<'_, AppState>,
    input_text: String,
    limit: Option<usize>,
) -> Result<Vec<crate::db::SavedResponseTemplate>, String> {
    ops_commands::find_similar_saved_responses_impl(state, input_text, limit).await
}

// ============================================================================
// Phase 2 v0.4.0: Response Alternatives Commands
// ============================================================================

/// Save a response alternative
#[tauri::command]
pub async fn save_response_alternative(
    state: State<'_, AppState>,
    draft_id: String,
    original_text: String,
    alternative_text: String,
    sources_json: Option<String>,
    metrics_json: Option<String>,
    generation_params_json: Option<String>,
) -> Result<String, String> {
    ops_commands::save_response_alternative_impl(
        state,
        draft_id,
        original_text,
        alternative_text,
        sources_json,
        metrics_json,
        generation_params_json,
    )
    .await
}

/// Get alternatives for a draft
#[tauri::command]
pub async fn get_alternatives_for_draft(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<Vec<crate::db::ResponseAlternative>, String> {
    ops_commands::get_alternatives_for_draft_impl(state, draft_id).await
}

/// Choose an alternative response
#[tauri::command]
pub async fn choose_alternative(
    state: State<'_, AppState>,
    alternative_id: String,
    choice: String,
) -> Result<(), String> {
    ops_commands::choose_alternative_impl(state, alternative_id, choice).await
}

// ============================================================================
// Phase 2 v0.4.0: Jira Status Transition Commands
// ============================================================================

/// Get available Jira transitions for a ticket
#[tauri::command]
pub async fn get_jira_transitions(
    state: State<'_, AppState>,
    ticket_key: String,
) -> Result<Vec<crate::jira::JiraTransition>, String> {
    jira_commands::get_jira_transitions_impl(state, ticket_key).await
}

/// Transition a Jira ticket to a new status
#[tauri::command]
pub async fn transition_jira_ticket(
    state: State<'_, AppState>,
    ticket_key: String,
    transition_id: String,
    draft_id: Option<String>,
) -> Result<(), String> {
    jira_commands::transition_jira_ticket_impl(state, ticket_key, transition_id, draft_id).await
}

/// Post a comment to Jira and optionally transition the ticket
#[tauri::command]
pub async fn post_and_transition(
    state: State<'_, AppState>,
    ticket_key: String,
    comment: String,
    transition_id: Option<String>,
    draft_id: Option<String>,
) -> Result<String, String> {
    jira_commands::post_and_transition_impl(state, ticket_key, comment, transition_id, draft_id)
        .await
}

// ============================================================================
