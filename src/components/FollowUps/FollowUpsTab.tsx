import { useState, useEffect, useCallback, useRef } from 'react';
import { useDrafts } from '../../hooks/useDrafts';
import { useCustomVariables } from '../../hooks/useCustomVariables';
import { Button } from '../shared/Button';
import { Skeleton } from '../shared/Skeleton';
import type { SavedDraft, ResponseTemplate, TemplateContext } from '../../types';
import {
  applyTemplate,
  getBuiltinVariableNames,
  BUILTIN_VARIABLE_DESCRIPTIONS,
  formatVariable,
} from '../../utils/templates';
import './FollowUpsTab.css';

interface FollowUpsTabProps {
  onLoadDraft?: (draft: SavedDraft) => void;
  onUseTemplate?: (content: string) => void;
  templateContext?: TemplateContext;
}

type Section = 'history' | 'templates';

export function FollowUpsTab({ onLoadDraft, onUseTemplate, templateContext = {} }: FollowUpsTabProps) {
  const {
    drafts,
    templates,
    loading,
    loadDrafts,
    searchDrafts,
    loadTemplates,
    deleteDraft,
    updateDraft,
    saveTemplate,
    updateTemplate,
    deleteTemplate,
    getDraftVersions,
    computeInputHash,
  } = useDrafts();

  const { variables: customVariables, loadVariables: loadCustomVariables } = useCustomVariables();

  const [activeSection, setActiveSection] = useState<Section>('history');
  const [showTemplateForm, setShowTemplateForm] = useState(false);
  const [editingTemplate, setEditingTemplate] = useState<ResponseTemplate | null>(null);
  const [templateFormData, setTemplateFormData] = useState({
    name: '',
    category: '',
    content: '',
  });
  const [deleteConfirm, setDeleteConfirm] = useState<{ type: 'draft' | 'template'; id: string } | null>(null);
  const [showVariablePicker, setShowVariablePicker] = useState(false);
  const contentTextareaRef = useRef<HTMLTextAreaElement>(null);

  // Draft search state
  const [draftSearchQuery, setDraftSearchQuery] = useState('');
  const [ticketFilter, setTicketFilter] = useState('');
  const [filteredDrafts, setFilteredDrafts] = useState<SavedDraft[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const searchDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Version history state
  const [expandedVersions, setExpandedVersions] = useState<string | null>(null);
  const [versionData, setVersionData] = useState<Record<string, SavedDraft[]>>({});
  const [loadingVersions, setLoadingVersions] = useState<string | null>(null);

  useEffect(() => {
    loadDrafts();
    loadTemplates();
    loadCustomVariables();
  }, [loadDrafts, loadTemplates, loadCustomVariables]);

  // Update filtered drafts when drafts change or filters change
  useEffect(() => {
    let result = drafts;

    // Apply ticket filter if set
    if (ticketFilter.trim()) {
      result = result.filter(
        (d) => d.ticket_id?.toLowerCase().includes(ticketFilter.toLowerCase())
      );
    }

    // If no text search, use the ticket-filtered result
    if (!draftSearchQuery.trim()) {
      setFilteredDrafts(result);
    }
  }, [drafts, draftSearchQuery, ticketFilter]);

  // Debounced search
  const handleDraftSearch = useCallback((query: string) => {
    setDraftSearchQuery(query);

    if (searchDebounceRef.current) {
      clearTimeout(searchDebounceRef.current);
    }

    if (!query.trim()) {
      // Apply ticket filter even when no text search
      let result = drafts;
      if (ticketFilter.trim()) {
        result = result.filter(
          (d) => d.ticket_id?.toLowerCase().includes(ticketFilter.toLowerCase())
        );
      }
      setFilteredDrafts(result);
      setIsSearching(false);
      return;
    }

    setIsSearching(true);
    searchDebounceRef.current = setTimeout(async () => {
      let results = await searchDrafts(query);
      // Apply ticket filter on top of text search
      if (ticketFilter.trim()) {
        results = results.filter(
          (d) => d.ticket_id?.toLowerCase().includes(ticketFilter.toLowerCase())
        );
      }
      setFilteredDrafts(results);
      setIsSearching(false);
    }, 300);
  }, [drafts, searchDrafts, ticketFilter]);

  // Handle ticket filter changes
  const handleTicketFilter = useCallback((filter: string) => {
    const upperFilter = filter.toUpperCase();
    setTicketFilter(upperFilter);

    // Re-apply filters
    let result = drafts;

    // Apply ticket filter
    if (upperFilter.trim()) {
      result = result.filter(
        (d) => d.ticket_id?.toLowerCase().includes(upperFilter.toLowerCase())
      );
    }

    // If there's also a text search, re-run it with the ticket filter
    if (draftSearchQuery.trim()) {
      setIsSearching(true);
      searchDrafts(draftSearchQuery).then((searchResults) => {
        if (upperFilter.trim()) {
          searchResults = searchResults.filter(
            (d) => d.ticket_id?.toLowerCase().includes(upperFilter.toLowerCase())
          );
        }
        setFilteredDrafts(searchResults);
        setIsSearching(false);
      });
    } else {
      setFilteredDrafts(result);
    }
  }, [drafts, searchDrafts, draftSearchQuery]);

  // Handle clicking on a ticket badge to filter
  const handleTicketBadgeClick = useCallback((ticketId: string) => {
    handleTicketFilter(ticketId);
  }, [handleTicketFilter]);

  const handleLoadDraft = useCallback((draft: SavedDraft) => {
    onLoadDraft?.(draft);
  }, [onLoadDraft]);

  const handleDeleteDraft = useCallback(async (draftId: string) => {
    await deleteDraft(draftId);
    setDeleteConfirm(null);
  }, [deleteDraft]);

  // Toggle version history for a draft
  const handleToggleVersions = useCallback(async (draft: SavedDraft) => {
    if (expandedVersions === draft.id) {
      setExpandedVersions(null);
      return;
    }

    setExpandedVersions(draft.id);
    setLoadingVersions(draft.id);

    try {
      const inputHash = await computeInputHash(draft.input_text);
      const versions = await getDraftVersions(inputHash);
      // Filter out the current draft from versions
      const filteredVersions = versions.filter(v => v.id !== draft.id);
      setVersionData(prev => ({ ...prev, [draft.id]: filteredVersions }));
    } catch (err) {
      console.error('Failed to load versions:', err);
    } finally {
      setLoadingVersions(null);
    }
  }, [expandedVersions, computeInputHash, getDraftVersions]);

  // Restore a version by updating the current draft
  const handleRestoreVersion = useCallback(async (currentDraft: SavedDraft, version: SavedDraft) => {
    const updatedDraft: SavedDraft = {
      ...currentDraft,
      response_text: version.response_text,
      summary_text: version.summary_text,
      diagnosis_json: version.diagnosis_json,
      kb_sources_json: version.kb_sources_json,
      model_name: version.model_name,
    };
    await updateDraft(updatedDraft);
    setExpandedVersions(null);
  }, [updateDraft]);

  const handleUseTemplate = useCallback((template: ResponseTemplate) => {
    // Apply template variable replacement
    const processedContent = applyTemplate(template.content, templateContext, customVariables);
    navigator.clipboard.writeText(processedContent);
    onUseTemplate?.(processedContent);
  }, [onUseTemplate, templateContext, customVariables]);

  const handleInsertVariable = useCallback((varName: string) => {
    const textarea = contentTextareaRef.current;
    if (!textarea) return;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const text = templateFormData.content;
    const varText = formatVariable(varName);

    const newContent = text.substring(0, start) + varText + text.substring(end);
    setTemplateFormData({ ...templateFormData, content: newContent });
    setShowVariablePicker(false);

    // Restore focus and position cursor after the inserted variable
    setTimeout(() => {
      textarea.focus();
      const newPos = start + varText.length;
      textarea.setSelectionRange(newPos, newPos);
    }, 0);
  }, [templateFormData]);

  const handleEditTemplate = useCallback((template: ResponseTemplate) => {
    setEditingTemplate(template);
    setTemplateFormData({
      name: template.name,
      category: template.category || '',
      content: template.content,
    });
    setShowTemplateForm(true);
  }, []);

  const handleDeleteTemplate = useCallback(async (templateId: string) => {
    await deleteTemplate(templateId);
    setDeleteConfirm(null);
  }, [deleteTemplate]);

  const handleSaveTemplate = useCallback(async () => {
    if (!templateFormData.name.trim() || !templateFormData.content.trim()) return;

    if (editingTemplate) {
      await updateTemplate({
        ...editingTemplate,
        name: templateFormData.name.trim(),
        category: templateFormData.category.trim() || null,
        content: templateFormData.content.trim(),
      });
    } else {
      await saveTemplate({
        name: templateFormData.name.trim(),
        category: templateFormData.category.trim() || null,
        content: templateFormData.content.trim(),
      });
    }

    setShowTemplateForm(false);
    setEditingTemplate(null);
    setTemplateFormData({ name: '', category: '', content: '' });
  }, [templateFormData, editingTemplate, saveTemplate, updateTemplate]);

  const handleCancelTemplateForm = useCallback(() => {
    setShowTemplateForm(false);
    setEditingTemplate(null);
    setTemplateFormData({ name: '', category: '', content: '' });
  }, []);

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const truncateText = (text: string, maxLength: number) => {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + '...';
  };

  const renderHistorySection = () => {
    const displayDrafts = filteredDrafts.length > 0 || draftSearchQuery || ticketFilter ? filteredDrafts : drafts;

    return (
      <>
        {/* Search and filter inputs */}
        <div className="search-filters">
          <div className="search-bar">
            <input
              type="text"
              placeholder="Search drafts..."
              value={draftSearchQuery}
              onChange={(e) => handleDraftSearch(e.target.value)}
              className="search-input"
            />
            {draftSearchQuery && (
              <button
                className="search-clear"
                onClick={() => handleDraftSearch('')}
                aria-label="Clear search"
              >
                &times;
              </button>
            )}
          </div>
          <div className="ticket-filter-bar">
            <input
              type="text"
              placeholder="Filter by ticket..."
              value={ticketFilter}
              onChange={(e) => handleTicketFilter(e.target.value)}
              className="ticket-filter-input"
            />
            {ticketFilter && (
              <button
                className="search-clear"
                onClick={() => handleTicketFilter('')}
                aria-label="Clear ticket filter"
              >
                &times;
              </button>
            )}
          </div>
        </div>

        {loading && drafts.length === 0 ? (
          <div className="draft-list">
            {Array.from({ length: 3 }).map((_, i) => (
              <div key={i} className="draft-card skeleton-card">
                <Skeleton width="70%" height="1.2em" />
                <Skeleton width="100%" height="2em" />
                <Skeleton width="40%" height="1em" />
              </div>
            ))}
          </div>
        ) : isSearching ? (
          <div className="section-loading">Searching...</div>
        ) : displayDrafts.length === 0 ? (
          <div className="section-empty">
            {draftSearchQuery || ticketFilter ? (
              <>
                <p>
                  No drafts found
                  {draftSearchQuery && ` matching "${draftSearchQuery}"`}
                  {draftSearchQuery && ticketFilter && ' and'}
                  {ticketFilter && ` for ticket "${ticketFilter}"`}
                </p>
                <p className="empty-hint">Try different search terms or clear the filters.</p>
              </>
            ) : (
              <>
                <p>No saved drafts yet.</p>
                <p className="empty-hint">
                  Generate a response in the Draft tab and click "Save Draft" to save it here.
                </p>
              </>
            )}
          </div>
        ) : (
          <div className="draft-list">
            {displayDrafts.map((draft) => (
          <div key={draft.id} className="draft-card">
            <div className="draft-header">
              <span className="draft-date">{formatDate(draft.created_at)}</span>
              <div className="draft-badges">
                {draft.model_name && (
                  <span className="draft-model" title={`Generated by ${draft.model_name}`}>
                    {draft.model_name}
                  </span>
                )}
                {draft.ticket_id && (
                  <button
                    className="draft-ticket"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleTicketBadgeClick(draft.ticket_id!);
                    }}
                    title="Click to filter by this ticket"
                  >
                    {draft.ticket_id}
                  </button>
                )}
              </div>
            </div>
            <div className="draft-preview">
              <div className="draft-input-preview">
                <span className="preview-label">Input:</span>
                <span>{truncateText(draft.input_text, 100)}</span>
              </div>
              {draft.response_text && (
                <div className="draft-response-preview">
                  <span className="preview-label">Response:</span>
                  <span>{truncateText(draft.response_text, 100)}</span>
                </div>
              )}
            </div>
            <div className="draft-actions">
              <Button
                variant="primary"
                size="small"
                onClick={() => handleLoadDraft(draft)}
              >
                Load
              </Button>
              <Button
                variant="ghost"
                size="small"
                onClick={() => handleToggleVersions(draft)}
              >
                {loadingVersions === draft.id ? 'Loading...' : expandedVersions === draft.id ? 'Hide Versions' : 'Versions'}
              </Button>
              <Button
                variant="ghost"
                size="small"
                onClick={() => setDeleteConfirm({ type: 'draft', id: draft.id })}
              >
                Delete
              </Button>
            </div>

            {/* Version History */}
            {expandedVersions === draft.id && (
              <div className="version-history">
                <h4>Version History</h4>
                {loadingVersions === draft.id ? (
                  <div className="version-loading">Loading versions...</div>
                ) : versionData[draft.id]?.length === 0 ? (
                  <div className="version-empty">No previous versions found.</div>
                ) : (
                  <div className="version-list">
                    {versionData[draft.id]?.map((version) => (
                      <div key={version.id} className="version-item">
                        <div className="version-info">
                          <span className="version-date">{formatDate(version.created_at)}</span>
                          <span className="version-preview">
                            {truncateText(version.response_text || 'No response', 80)}
                          </span>
                        </div>
                        <Button
                          variant="ghost"
                          size="small"
                          onClick={() => handleRestoreVersion(draft, version)}
                        >
                          Restore
                        </Button>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
            ))}
          </div>
        )}
      </>
    );
  };

  const renderTemplatesSection = () => {
    if (loading && templates.length === 0) {
      return (
        <div className="template-list">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="template-card skeleton-card">
              <Skeleton width="50%" height="1.2em" />
              <Skeleton width="100%" height="3em" />
            </div>
          ))}
        </div>
      );
    }

    return (
      <>
        <div className="templates-header">
          <Button
            variant="primary"
            size="small"
            onClick={() => setShowTemplateForm(true)}
          >
            Create Template
          </Button>
        </div>

        {templates.length === 0 ? (
          <div className="section-empty">
            <p>No templates yet.</p>
            <p className="empty-hint">
              Create templates for common responses you send frequently.
            </p>
          </div>
        ) : (
          <div className="template-list">
            {templates.map((template) => (
              <div key={template.id} className="template-card">
                <div className="template-header">
                  <span className="template-name">{template.name}</span>
                  {template.category && (
                    <span className="template-category">{template.category}</span>
                  )}
                </div>
                <div className="template-preview">
                  {truncateText(template.content, 150)}
                </div>
                <div className="template-actions">
                  <Button
                    variant="primary"
                    size="small"
                    onClick={() => handleUseTemplate(template)}
                  >
                    Copy
                  </Button>
                  <Button
                    variant="ghost"
                    size="small"
                    onClick={() => handleEditTemplate(template)}
                  >
                    Edit
                  </Button>
                  <Button
                    variant="ghost"
                    size="small"
                    onClick={() => setDeleteConfirm({ type: 'template', id: template.id })}
                  >
                    Delete
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}
      </>
    );
  };

  return (
    <div className="followups-tab">
      <div className="section-tabs">
        <button
          className={`section-tab ${activeSection === 'history' ? 'active' : ''}`}
          onClick={() => setActiveSection('history')}
        >
          History ({drafts.length})
        </button>
        <button
          className={`section-tab ${activeSection === 'templates' ? 'active' : ''}`}
          onClick={() => setActiveSection('templates')}
        >
          Templates ({templates.length})
        </button>
      </div>

      <div className="section-content">
        {activeSection === 'history' ? renderHistorySection() : renderTemplatesSection()}
      </div>

      {/* Template Form Modal */}
      {showTemplateForm && (
        <div className="modal-overlay" onClick={handleCancelTemplateForm}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h3>{editingTemplate ? 'Edit Template' : 'Create Template'}</h3>
            <div className="template-form">
              <div className="form-field">
                <label htmlFor="template-name">Name</label>
                <input
                  id="template-name"
                  type="text"
                  placeholder="e.g., Password Reset Response"
                  value={templateFormData.name}
                  onChange={(e) => setTemplateFormData({ ...templateFormData, name: e.target.value })}
                  autoFocus
                />
              </div>
              <div className="form-field">
                <label htmlFor="template-category">Category (optional)</label>
                <input
                  id="template-category"
                  type="text"
                  placeholder="e.g., Password, VPN, General"
                  value={templateFormData.category}
                  onChange={(e) => setTemplateFormData({ ...templateFormData, category: e.target.value })}
                />
              </div>
              <div className="form-field">
                <div className="field-header">
                  <label htmlFor="template-content">Content</label>
                  <div className="variable-picker-wrapper">
                    <Button
                      variant="ghost"
                      size="small"
                      onClick={() => setShowVariablePicker(!showVariablePicker)}
                    >
                      Insert Variable
                    </Button>
                    {showVariablePicker && (
                      <div className="variable-picker-dropdown">
                        <div className="variable-section">
                          <div className="variable-section-title">Built-in Variables</div>
                          {getBuiltinVariableNames().map((name) => (
                            <button
                              key={name}
                              className="variable-option"
                              onClick={() => handleInsertVariable(name)}
                              title={BUILTIN_VARIABLE_DESCRIPTIONS[name]}
                            >
                              <span className="variable-name">{`{{${name}}}`}</span>
                              <span className="variable-desc">{BUILTIN_VARIABLE_DESCRIPTIONS[name]}</span>
                            </button>
                          ))}
                        </div>
                        {customVariables.length > 0 && (
                          <div className="variable-section">
                            <div className="variable-section-title">Custom Variables</div>
                            {customVariables.map((v) => (
                              <button
                                key={v.id}
                                className="variable-option"
                                onClick={() => handleInsertVariable(v.name)}
                                title={v.value}
                              >
                                <span className="variable-name">{`{{${v.name}}}`}</span>
                                <span className="variable-desc">{v.value}</span>
                              </button>
                            ))}
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                </div>
                <textarea
                  ref={contentTextareaRef}
                  id="template-content"
                  placeholder="Enter the template content... Use {{variable_name}} for dynamic values."
                  value={templateFormData.content}
                  onChange={(e) => setTemplateFormData({ ...templateFormData, content: e.target.value })}
                  rows={8}
                />
              </div>
              <div className="form-actions">
                <Button variant="ghost" onClick={handleCancelTemplateForm}>
                  Cancel
                </Button>
                <Button
                  variant="primary"
                  onClick={handleSaveTemplate}
                  disabled={!templateFormData.name.trim() || !templateFormData.content.trim()}
                >
                  {editingTemplate ? 'Save Changes' : 'Create'}
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Delete Confirmation Modal */}
      {deleteConfirm && (
        <div className="modal-overlay" onClick={() => setDeleteConfirm(null)}>
          <div className="modal-content modal-confirm" onClick={(e) => e.stopPropagation()}>
            <h3>Confirm Delete</h3>
            <p>
              Are you sure you want to delete this {deleteConfirm.type}? This action cannot be undone.
            </p>
            <div className="form-actions">
              <Button variant="ghost" onClick={() => setDeleteConfirm(null)}>
                Cancel
              </Button>
              <Button
                variant="primary"
                onClick={() => {
                  if (deleteConfirm.type === 'draft') {
                    handleDeleteDraft(deleteConfirm.id);
                  } else {
                    handleDeleteTemplate(deleteConfirm.id);
                  }
                }}
              >
                Delete
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
