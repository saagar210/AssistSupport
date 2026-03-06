import { Button } from '../shared/Button';

interface DraftWorkflowStripProps {
  inputWordCount: number;
  hasTicket: boolean;
  treeCompleted: boolean;
  checklistCompletedCount: number;
  checklistTotal: number;
  responseWordCount: number;
  isResponseEdited: boolean;
  responseEditRatioPercent: number;
  hasResponse: boolean;
  handoffTouched: boolean;
  panelDensityMode: 'balanced' | 'focus-intake' | 'focus-response';
  onPanelDensityModeChange: (mode: 'balanced' | 'focus-intake' | 'focus-response') => void;
  onGenerateFirstResponse: () => void;
  onChecklistGenerate: () => void;
  onGenerateFullResponse: () => void;
  onSaveDraft: () => void;
  modelLoaded: boolean;
  firstResponseGenerating: boolean;
  checklistGenerating: boolean;
  generating: boolean;
  canGenerateChecklist: boolean;
  canGenerateResponse: boolean;
  canSave: boolean;
}

export function DraftWorkflowStrip({
  inputWordCount,
  hasTicket,
  treeCompleted,
  checklistCompletedCount,
  checklistTotal,
  responseWordCount,
  isResponseEdited,
  responseEditRatioPercent,
  hasResponse,
  handoffTouched,
  panelDensityMode,
  onPanelDensityModeChange,
  onGenerateFirstResponse,
  onChecklistGenerate,
  onGenerateFullResponse,
  onSaveDraft,
  modelLoaded,
  firstResponseGenerating,
  checklistGenerating,
  generating,
  canGenerateChecklist,
  canGenerateResponse,
  canSave,
}: DraftWorkflowStripProps) {
  return (
    <section className="draft-workflow-strip" aria-label="Draft workflow overview">
      <div className="draft-workflow-step">
        <h4>1. Intake</h4>
        <p>{inputWordCount} words captured {hasTicket ? '· ticket linked' : '· no ticket linked'}</p>
      </div>
      <div className="draft-workflow-step">
        <h4>2. Diagnose</h4>
        <p>
          {treeCompleted ? 'Tree completed' : 'Tree not run'}
          {' · '}
          checklist {checklistCompletedCount}/{checklistTotal}
        </p>
      </div>
      <div className="draft-workflow-step">
        <h4>3. Draft</h4>
        <p>
          {responseWordCount} words
          {isResponseEdited ? ` · edited (${responseEditRatioPercent}%)` : ' · unedited'}
        </p>
      </div>
      <div className="draft-workflow-step">
        <h4>4. Handoff</h4>
        <p>
          {hasResponse
            ? handoffTouched
              ? 'Copied/exported'
              : 'Ready to copy/export'
            : 'No response yet'}
        </p>
      </div>
      <div className="draft-workflow-actions">
        <div className="draft-layout-mode-toggle" role="group" aria-label="Draft panel layout">
          <button
            type="button"
            className={`draft-layout-mode-btn ${panelDensityMode === 'balanced' ? 'active' : ''}`}
            onClick={() => onPanelDensityModeChange('balanced')}
          >
            Balanced
          </button>
          <button
            type="button"
            className={`draft-layout-mode-btn ${panelDensityMode === 'focus-intake' ? 'active' : ''}`}
            onClick={() => onPanelDensityModeChange('focus-intake')}
          >
            Intake Focus
          </button>
          <button
            type="button"
            className={`draft-layout-mode-btn ${panelDensityMode === 'focus-response' ? 'active' : ''}`}
            onClick={() => onPanelDensityModeChange('focus-response')}
          >
            Response Focus
          </button>
        </div>
        <Button
          size="small"
          variant="secondary"
          onClick={onGenerateFirstResponse}
          disabled={!modelLoaded || firstResponseGenerating || !canGenerateResponse}
        >
          Draft First Reply
        </Button>
        <Button
          size="small"
          variant="ghost"
          onClick={onChecklistGenerate}
          disabled={!modelLoaded || checklistGenerating || !canGenerateChecklist}
        >
          Build Checklist
        </Button>
        <Button
          size="small"
          variant="primary"
          onClick={onGenerateFullResponse}
          disabled={!modelLoaded || generating || !canGenerateResponse}
          title="Generate response (Cmd+G in input)"
          aria-keyshortcuts="Meta+G"
        >
          Generate Full Response
        </Button>
        <Button
          size="small"
          variant="ghost"
          onClick={onSaveDraft}
          disabled={!canSave}
        >
          Save
        </Button>
        <div className="draft-workflow-shortcuts" aria-label="Keyboard shortcuts">
          <span><kbd>Cmd</kbd>+<kbd>G</kbd> Generate</span>
          <span><kbd>Cmd</kbd>+<kbd>N</kbd> Clear</span>
          <span><kbd>Cmd</kbd>+<kbd>1</kbd>/<kbd>2</kbd>/<kbd>3</kbd> Layout</span>
        </div>
      </div>
    </section>
  );
}
