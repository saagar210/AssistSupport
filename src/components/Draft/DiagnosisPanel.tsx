import { useState, useEffect } from 'react';
import { useDecisionTrees } from '../../hooks/useDecisionTrees';
import { TreeRunner } from '../Trees/TreeRunner';
import type { TreeStructure } from '../../types';
import './DiagnosisPanel.css';

interface DiagnosisPanelProps {
  input: string;
  ocrText: string | null;
  notes: string;
  onNotesChange: (notes: string) => void;
  collapsed: boolean;
  onToggleCollapse: () => void;
}

export function DiagnosisPanel({
  input,
  ocrText: _ocrText,
  notes,
  onNotesChange,
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
    // Add path summary to notes
    if (activeTree) {
      const pathSummary = path
        .map(nodeId => activeTree.nodes[nodeId]?.title || nodeId)
        .join(' → ');
      const tree = trees.find(t => t.id === selectedTreeId);
      const treeName = tree?.name || 'Decision Tree';
      const newNote = `[${treeName}]\n${pathSummary}\n`;
      onNotesChange(notes ? `${notes}\n${newNote}` : newNote);
    }
  }

  function handleTreeReset() {
    setActiveTree(null);
    setSelectedTreeId(null);
  }

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
          {!activeTree ? (
            <div className="tree-selector">
              <select
                value={selectedTreeId || ''}
                onChange={e => handleTreeSelect(e.target.value)}
                disabled={loading}
                className="tree-dropdown"
              >
                <option value="">Select a troubleshooting tree...</option>
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

        {/* Quick Checklist */}
        <div className="diagnosis-section">
          <h4>Quick Checklist</h4>
          {input.trim() ? (
            <ul className="check-list">
              <li>
                <input type="checkbox" id="check-1" />
                <label htmlFor="check-1">Verify issue reproduction steps</label>
              </li>
              <li>
                <input type="checkbox" id="check-2" />
                <label htmlFor="check-2">Check relevant KB documentation</label>
              </li>
              <li>
                <input type="checkbox" id="check-3" />
                <label htmlFor="check-3">Review any error messages</label>
              </li>
              <li>
                <input type="checkbox" id="check-4" />
                <label htmlFor="check-4">Identify affected systems/users</label>
              </li>
            </ul>
          ) : (
            <p className="diagnosis-placeholder">
              Enter ticket content to see diagnostic suggestions...
            </p>
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
