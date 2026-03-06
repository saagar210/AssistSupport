use super::*;
use crate::db::{ResponseTemplate, SavedDraft};
use crate::exports::{
    format_draft, format_for_clipboard, ExportFormat as DraftExportFormat, ExportedSource,
    SafeExportOptions,
};

fn parse_exported_sources(sources_json: Option<String>) -> Vec<ExportedSource> {
    sources_json
        .and_then(|json| serde_json::from_str::<Vec<serde_json::Value>>(&json).ok())
        .map(|sources| {
            sources
                .iter()
                .map(|s| ExportedSource {
                    title: s["title"].as_str().unwrap_or("Unknown").to_string(),
                    path: s["file_path"].as_str().map(|p| p.to_string()),
                    url: s["url"].as_str().map(|u| u.to_string()),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn list_drafts_impl(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<SavedDraft>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_drafts(limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

pub(crate) fn search_drafts_impl(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SavedDraft>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.search_drafts(&query, limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

pub(crate) fn get_draft_impl(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<SavedDraft, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_draft(&draft_id).map_err(|e| e.to_string())
}

pub(crate) fn save_draft_impl(
    state: State<'_, AppState>,
    draft: SavedDraft,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.save_draft(&draft).map_err(|e| e.to_string())
}

pub(crate) fn delete_draft_impl(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_draft(&draft_id).map_err(|e| e.to_string())
}

pub(crate) fn list_autosaves_impl(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<SavedDraft>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_autosaves(limit.unwrap_or(10))
        .map_err(|e| e.to_string())
}

pub(crate) fn cleanup_autosaves_impl(
    state: State<'_, AppState>,
    keep_count: Option<usize>,
) -> Result<usize, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.cleanup_autosaves(keep_count.unwrap_or(10))
        .map_err(|e| e.to_string())
}

pub(crate) fn list_decision_trees_impl(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::DecisionTree>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_decision_trees().map_err(|e| e.to_string())
}

pub(crate) fn get_decision_tree_impl(
    state: State<'_, AppState>,
    tree_id: String,
) -> Result<crate::db::DecisionTree, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_decision_tree(&tree_id).map_err(|e| e.to_string())
}

pub(crate) fn export_draft_formatted_impl(
    state: State<'_, AppState>,
    draft_id: String,
    format: String,
    safe_export: Option<SafeExportOptions>,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    let draft = db.get_draft(&draft_id).map_err(|e| e.to_string())?;

    let response_text = draft.response_text.as_deref().unwrap_or("");
    let sources = parse_exported_sources(draft.kb_sources_json);

    let export_format = match format.as_str() {
        "html" => DraftExportFormat::Html,
        "ticket_html" => DraftExportFormat::TicketHtml,
        "json" => DraftExportFormat::Json,
        _ => DraftExportFormat::Plaintext,
    };

    Ok(format_draft(
        response_text,
        draft.summary_text.as_deref(),
        &sources,
        export_format,
        safe_export.as_ref(),
    ))
}

pub(crate) fn format_draft_for_clipboard_impl(
    state: State<'_, AppState>,
    draft_id: String,
    include_sources: bool,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    let draft = db.get_draft(&draft_id).map_err(|e| e.to_string())?;

    let response_text = draft.response_text.as_deref().unwrap_or("");
    let sources = parse_exported_sources(draft.kb_sources_json);

    Ok(format_for_clipboard(
        response_text,
        &sources,
        include_sources,
    ))
}

pub(crate) fn get_draft_versions_impl(
    state: State<'_, AppState>,
    input_hash: String,
) -> Result<Vec<SavedDraft>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_draft_versions(&input_hash)
        .map_err(|e| e.to_string())
}

pub(crate) fn create_draft_version_impl(
    state: State<'_, AppState>,
    draft_id: String,
    change_reason: Option<String>,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.create_draft_version(&draft_id, change_reason.as_deref())
        .map_err(|e| e.to_string())
}

pub(crate) fn list_draft_versions_impl(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<Vec<crate::db::DraftVersion>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_draft_versions(&draft_id).map_err(|e| e.to_string())
}

pub(crate) fn finalize_draft_impl(
    state: State<'_, AppState>,
    draft_id: String,
    finalized_by: Option<String>,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.finalize_draft(&draft_id, finalized_by.as_deref())
        .map_err(|e| e.to_string())
}

pub(crate) fn archive_draft_impl(
    state: State<'_, AppState>,
    draft_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.archive_draft(&draft_id).map_err(|e| e.to_string())
}

pub(crate) fn update_draft_handoff_impl(
    state: State<'_, AppState>,
    draft_id: String,
    handoff_summary: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.update_draft_handoff(&draft_id, &handoff_summary)
        .map_err(|e| e.to_string())
}

pub(crate) fn list_playbooks_impl(
    state: State<'_, AppState>,
    category: Option<String>,
) -> Result<Vec<crate::db::Playbook>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_playbooks(category.as_deref())
        .map_err(|e| e.to_string())
}

pub(crate) fn get_playbook_impl(
    state: State<'_, AppState>,
    playbook_id: String,
) -> Result<crate::db::Playbook, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_playbook(&playbook_id).map_err(|e| e.to_string())
}

pub(crate) fn save_playbook_impl(
    state: State<'_, AppState>,
    playbook: crate::db::Playbook,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.save_playbook(&playbook).map_err(|e| e.to_string())
}

pub(crate) fn use_playbook_impl(
    state: State<'_, AppState>,
    playbook_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.increment_playbook_usage(&playbook_id)
        .map_err(|e| e.to_string())
}

pub(crate) fn delete_playbook_impl(
    state: State<'_, AppState>,
    playbook_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_playbook(&playbook_id).map_err(|e| e.to_string())
}

pub(crate) fn list_action_shortcuts_impl(
    state: State<'_, AppState>,
    category: Option<String>,
) -> Result<Vec<crate::db::ActionShortcut>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_action_shortcuts(category.as_deref())
        .map_err(|e| e.to_string())
}

pub(crate) fn get_action_shortcut_impl(
    state: State<'_, AppState>,
    shortcut_id: String,
) -> Result<crate::db::ActionShortcut, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_action_shortcut(&shortcut_id)
        .map_err(|e| e.to_string())
}

pub(crate) fn save_action_shortcut_impl(
    state: State<'_, AppState>,
    shortcut: crate::db::ActionShortcut,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.save_action_shortcut(&shortcut)
        .map_err(|e| e.to_string())
}

pub(crate) fn delete_action_shortcut_impl(
    state: State<'_, AppState>,
    shortcut_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_action_shortcut(&shortcut_id)
        .map_err(|e| e.to_string())
}

pub(crate) fn list_templates_impl(
    state: State<'_, AppState>,
) -> Result<Vec<ResponseTemplate>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_templates().map_err(|e| e.to_string())
}

pub(crate) fn get_template_impl(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<ResponseTemplate, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_template(&template_id).map_err(|e| e.to_string())
}

pub(crate) fn save_template_impl(
    state: State<'_, AppState>,
    template: ResponseTemplate,
) -> Result<String, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    let template_id = template.id.clone();
    db.save_template(&template).map_err(|e| e.to_string())?;
    Ok(template_id)
}

pub(crate) fn delete_template_impl(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_template(&template_id).map_err(|e| e.to_string())
}

pub(crate) fn list_custom_variables_impl(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::CustomVariable>, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.list_custom_variables().map_err(|e| e.to_string())
}

pub(crate) fn get_custom_variable_impl(
    state: State<'_, AppState>,
    variable_id: String,
) -> Result<crate::db::CustomVariable, String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.get_custom_variable(&variable_id)
        .map_err(|e| e.to_string())
}

pub(crate) fn save_custom_variable_impl(
    state: State<'_, AppState>,
    variable: crate::db::CustomVariable,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.save_custom_variable(&variable)
        .map_err(|e| e.to_string())
}

pub(crate) fn delete_custom_variable_impl(
    state: State<'_, AppState>,
    variable_id: String,
) -> Result<(), String> {
    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.delete_custom_variable(&variable_id)
        .map_err(|e| e.to_string())
}
