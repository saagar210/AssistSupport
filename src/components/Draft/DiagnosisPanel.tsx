import { useState, useEffect } from 'react';
import { useDecisionTrees } from '../../hooks/useDecisionTrees';
import { TreeRunner } from '../Trees/TreeRunner';
import { Button } from '../shared/Button';
import type { TreeStructure, ChecklistItem, SearchResult, ContextSource } from '../../types';
import './DiagnosisPanel.css';

export interface TreeResult {
  treeId: string;
  treeName: string;
  path: string[];
  pathSummary: string;
}

interface DiagnosisPanelProps {
  input: string;
  ocrText: string | null;
  notes: string;
  onNotesChange: (notes: string) => void;
  treeResult: TreeResult | null;
  onTreeComplete: (result: TreeResult) => void;
  onTreeClear: () => void;
  checklistItems: ChecklistItem[];
  checklistCompleted: Record<string, boolean>;
  checklistGenerating: boolean;
  checklistUpdating: boolean;
  checklistError: string | null;
  onChecklistToggle: (id: string) => void;
  onChecklistGenerate: () => void;
  onChecklistUpdate: () => void;
  onChecklistClear: () => void;
  approvalQuery: string;
  onApprovalQueryChange: (value: string) => void;
  approvalResults: SearchResult[];
  approvalSearching: boolean;
  approvalSummary: string;
  approvalSummarizing: boolean;
  approvalSources: ContextSource[];
  onApprovalSearch: () => void;
  onApprovalSummarize: () => void;
  approvalError: string | null;
  modelLoaded: boolean;
  hasTicket: boolean;
  collapsed: boolean;
  onToggleCollapse: () => void;
}

export function DiagnosisPanel({
  input,
  ocrText,
  notes,
  onNotesChange,
  treeResult,
  onTreeComplete,
  onTreeClear,
  checklistItems,
  checklistCompleted,
  checklistGenerating,
  checklistUpdating,
  checklistError,
  onChecklistToggle,
  onChecklistGenerate,
  onChecklistUpdate,
  onChecklistClear,
  approvalQuery,
  onApprovalQueryChange,
  approvalResults,
  approvalSearching,
  approvalSummary,
  approvalSummarizing,
  approvalSources,
  onApprovalSearch,
  onApprovalSummarize,
  approvalError,
  modelLoaded,
  hasTicket,
  collapsed,
  onToggleCollapse,
}: DiagnosisPanelProps) {
  const { trees, loading, loadTrees, getTree } = useDecisionTrees();
  const [selectedTreeId, setSelectedTreeId] = useState<string | null>(null);
  const [activeTree, setActiveTree] = useState<TreeStructure | null>(null);

  useEffect(() => {
    loadTrees();
  }, [loadTrees]);

  async function handleTreeSelect(treeId: string) {
    if (!treeId) {
      setSelectedTreeId(null);
      setActiveTree(null);
      return;
    }

    setSelectedTreeId(treeId);
    const tree = await getTree(treeId);
    setActiveTree(tree);
  }

  function handleTreeComplete(path: string[]) {
    if (activeTree && selectedTreeId) {
      const pathSummary = path
        .map(nodeId => activeTree.nodes[nodeId]?.title || nodeId)
        .join(' → ');
      const tree = trees.find(t => t.id === selectedTreeId);
      const treeName = tree?.name || 'Decision Tree';

      // Return structured result
      onTreeComplete({
        treeId: selectedTreeId,
        treeName,
        path,
        pathSummary,
      });
    }
    // Reset local state
    setActiveTree(null);
    setSelectedTreeId(null);
  }

  function handleTreeReset() {
    setActiveTree(null);
    setSelectedTreeId(null);
  }

  function handleClearTreeResult() {
    onTreeClear();
  }

  const completedCount = checklistItems.reduce((count, item) => {
    return checklistCompleted[item.id] ? count + 1 : count;
  }, 0);
  const hasChecklist = checklistItems.length > 0;
  const hasChecklistInput = Boolean(input.trim() || ocrText?.trim() || hasTicket);

  if (collapsed) {
    return (
      <div className="diagnosis-collapsed-strip" onClick={onToggleCollapse}>
        <span className="collapse-label">Diagnosis</span>
        <button className="collapse-btn" aria-label="Expand">
          &#9654;
        </button>
      </div>
    );
  }

  return (
    <>
      <div className="panel-header">
        <h3>Diagnosis</h3>
        <button
          className="collapse-btn"
          onClick={onToggleCollapse}
          aria-label="Collapse"
        >
          &#9664;
        </button>
      </div>

      <div className="panel-content diagnosis-content">
        {/* Decision Tree Section */}
        <div className="diagnosis-section">
          <h4>Decision Trees</h4>

          {/* Show completed tree result */}
          {treeResult && (
            <div className="tree-result">
              <div className="tree-result-header">
                <span className="tree-result-name">{treeResult.treeName}</span>
                <button
                  className="tree-result-clear"
                  onClick={handleClearTreeResult}
                  title="Clear tree result"
                >
                  &times;
                </button>
              </div>
              <div className="tree-result-path">{treeResult.pathSummary}</div>
            </div>
          )}

          {/* Tree selector / runner */}
          {!activeTree ? (
            <div className="tree-selector">
              <select
                value={selectedTreeId || ''}
                onChange={e => handleTreeSelect(e.target.value)}
                disabled={loading}
                className="tree-dropdown"
              >
                <option value="">{treeResult ? 'Run another tree...' : 'Select a troubleshooting tree...'}</option>
                {trees.map(tree => (
                  <option key={tree.id} value={tree.id}>
                    {tree.name}
                    {tree.category && ` (${tree.category})`}
                  </option>
                ))}
              </select>
              {loading && <span className="tree-loading">Loading...</span>}
            </div>
          ) : (
            <TreeRunner
              tree={activeTree}
              onComplete={handleTreeComplete}
              onReset={handleTreeReset}
            />
          )}
        </div>

        {/* Troubleshooting Checklist */}
        <div className="diagnosis-section">
          <div className="checklist-header">
            <h4>Troubleshooting Checklist</h4>
            <div className="checklist-actions">
              <Button
                variant="ghost"
                size="small"
                onClick={onChecklistGenerate}
                loading={checklistGenerating}
                disabled={!modelLoaded || !hasChecklistInput}
              >
                Generate
              </Button>
              {hasChecklist && (
                <Button
                  variant="ghost"
                  size="small"
                  onClick={onChecklistUpdate}
                  loading={checklistUpdating}
                  disabled={!modelLoaded}
                >
                  Update
                </Button>
              )}
              {hasChecklist && (
                <Button
                  variant="ghost"
                  size="small"
                  onClick={onChecklistClear}
                >
                  Clear
                </Button>
              )}
            </div>
          </div>

          {checklistError && (
            <div className="checklist-error">{checklistError}</div>
          )}

          {hasChecklist ? (
            <>
              <fieldset className="check-fieldset">
                <legend className="check-legend">Checklist</legend>
                <ul className="check-list">
                  {checklistItems.map(item => {
                    const checkboxId = `check-${item.id}`;
                    const isChecked = !!checklistCompleted[item.id];
                    return (
                      <li key={item.id} className={isChecked ? 'check-item done' : 'check-item'}>
                        <input
                          type="checkbox"
                          id={checkboxId}
                          checked={isChecked}
                          onChange={() => onChecklistToggle(item.id)}
                        />
                        <label htmlFor={checkboxId}>
                          <span className="check-text">{item.text}</span>
                          {(item.category || item.priority) && (
                            <span className="check-meta">
                              {item.category && <span className="check-category">{item.category}</span>}
                              {item.priority && <span className={`check-priority priority-${item.priority}`}>{item.priority}</span>}
                            </span>
                          )}
                        </label>
                      </li>
                    );
                  })}
                </ul>
              </fieldset>
              <div className="checklist-progress">
                {completedCount}/{checklistItems.length} completed
              </div>
            </>
          ) : (
            <p className="diagnosis-placeholder">
              Generate a checklist to guide troubleshooting steps.
            </p>
          )}
        </div>

        {/* Approval Lookup */}
        <div className="diagnosis-section">
          <div className="approval-header">
            <h4>Approval Lookup</h4>
            <div className="approval-actions">
              <Button
                variant="ghost"
                size="small"
                onClick={onApprovalSearch}
                disabled={!approvalQuery.trim() || approvalSearching}
              >
                Search
              </Button>
              <Button
                variant="ghost"
                size="small"
                onClick={onApprovalSummarize}
                loading={approvalSummarizing}
                disabled={!approvalQuery.trim() || !modelLoaded}
              >
                Summarize
              </Button>
            </div>
          </div>

          <input
            className="approval-input"
            placeholder="Search for approval steps, owners, or app name..."
            value={approvalQuery}
            onChange={e => onApprovalQueryChange(e.target.value)}
          />

          {approvalSearching && <div className="approval-loading">Searching...</div>}
          {approvalError && <div className="approval-error">{approvalError}</div>}

          {approvalResults.length > 0 && (
            <ul className="approval-results">
              {approvalResults.map(result => {
                const snippet = result.snippet || result.content;
                const snippetText = snippet.length > 240 ? `${snippet.slice(0, 240)}...` : snippet;
                return (
                  <li key={result.chunk_id} className="approval-result">
                    <div className="approval-result-title">
                      {result.title || result.file_path}
                    </div>
                    {result.heading_path && (
                      <div className="approval-result-heading">{result.heading_path}</div>
                    )}
                    <div className="approval-result-snippet">
                      {snippetText}
                    </div>
                  </li>
                );
              })}
            </ul>
          )}

          {approvalSummary && (
            <div className="approval-summary">
              <div className="approval-summary-header">Summary</div>
              <div className="approval-summary-text">{approvalSummary}</div>
              {approvalSources.length > 0 && (
                <ul className="approval-sources">
                  {approvalSources.map((source, index) => (
                    <li key={source.chunk_id} className="approval-source">
                      <span className="approval-source-number">[{index + 1}]</span>
                      <span className="approval-source-title">
                        {source.title || source.file_path}
                      </span>
                      {source.heading_path && (
                        <span className="approval-source-heading">
                          &rsaquo; {source.heading_path}
                        </span>
                      )}
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </div>

        {/* Notes Section */}
        <div className="diagnosis-section">
          <h4>Notes</h4>
          <textarea
            className="notes-textarea"
            placeholder="Add your diagnostic notes here..."
            value={notes}
            onChange={e => onNotesChange(e.target.value)}
          />
        </div>
      </div>
    </>
  );
}
