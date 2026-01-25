import { useState, useCallback, useEffect, forwardRef, useImperativeHandle } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { InputPanel } from './InputPanel';
import { DiagnosisPanel } from './DiagnosisPanel';
import { ResponsePanel } from './ResponsePanel';
import { useLlm } from '../../hooks/useLlm';
import { useDrafts } from '../../hooks/useDrafts';
import { useToastContext } from '../../contexts/ToastContext';
import type { ContextSource, ResponseLength, SavedDraft } from '../../types';
import './DraftTab.css';

export interface DraftTabHandle {
  generate: () => void;
  loadDraft: (draft: SavedDraft) => void;
  saveDraft: () => void;
  copyResponse: () => void;
  cancelGeneration: () => void;
  exportResponse: () => void;
}

interface DraftTabProps {
  initialDraft?: SavedDraft | null;
}

export const DraftTab = forwardRef<DraftTabHandle, DraftTabProps>(function DraftTab({ initialDraft }, ref) {
  const { error: showError, success: showSuccess } = useToastContext();
  const { generateStreaming, getLoadedModel, streamingText, isStreaming, clearStreamingText, cancelGeneration } = useLlm();
  const { saveDraft, triggerAutosave, cancelAutosave } = useDrafts();

  const [input, setInput] = useState('');
  const [ocrText, setOcrText] = useState<string | null>(null);
  const [diagnosticNotes, setDiagnosticNotes] = useState('');
  const [response, setResponse] = useState('');
  const [sources, setSources] = useState<ContextSource[]>([]);
  const [responseLength, setResponseLength] = useState<ResponseLength>('Medium');
  const [diagnosisCollapsed, setDiagnosisCollapsed] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [modelLoaded, setModelLoaded] = useState(false);
  const [currentTicketId, setCurrentTicketId] = useState<string | null>(null);

  // Check model status on mount and when window gains focus
  useEffect(() => {
    checkModelStatus();

    // Re-check when user returns to the window (e.g., after changing tabs)
    const handleFocus = () => {
      checkModelStatus();
    };

    window.addEventListener('focus', handleFocus);

    // Also check periodically in case model is loaded in Settings tab
    const interval = setInterval(checkModelStatus, 3000);

    return () => {
      window.removeEventListener('focus', handleFocus);
      clearInterval(interval);
    };
  }, []);

  async function checkModelStatus() {
    try {
      const loaded = await getLoadedModel();
      setModelLoaded(!!loaded);
    } catch {
      setModelLoaded(false);
    }
  }

  const handleGenerate = useCallback(async () => {
    if (!input.trim() || generating) return;

    if (!modelLoaded) {
      showError('No model loaded. Go to Settings to load a model.');
      return;
    }

    setGenerating(true);
    setResponse(''); // Clear previous response
    clearStreamingText(); // Clear streaming buffer
    try {
      const combinedInput = ocrText ? `${input}\n\n[Screenshot OCR Text]:\n${ocrText}` : input;
      const result = await generateStreaming(combinedInput, responseLength);
      setResponse(result.text);
      setSources(result.sources);
    } catch (e) {
      console.error('Generation failed:', e);
      showError(`Generation failed: ${e}`);
    } finally {
      setGenerating(false);
    }
  }, [input, ocrText, responseLength, generating, modelLoaded, generateStreaming, clearStreamingText, showError]);

  const handleClear = useCallback(() => {
    setInput('');
    setOcrText(null);
    setDiagnosticNotes('');
    setResponse('');
    setSources([]);
    setCurrentTicketId(null);
  }, []);

  const handleCancel = useCallback(async () => {
    await cancelGeneration();
    setGenerating(false);
    // Keep the streaming text that was generated so far
    if (streamingText) {
      setResponse(streamingText);
    }
  }, [cancelGeneration, streamingText]);

  const handleLoadDraft = useCallback((draft: SavedDraft) => {
    setInput(draft.input_text);
    setResponse(draft.response_text || '');
    setDiagnosticNotes(draft.diagnosis_json ? JSON.parse(draft.diagnosis_json).notes || '' : '');
    setCurrentTicketId(draft.ticket_id);
    if (draft.kb_sources_json) {
      try {
        setSources(JSON.parse(draft.kb_sources_json));
      } catch {
        setSources([]);
      }
    } else {
      setSources([]);
    }
    setOcrText(null);
  }, []);

  const handleSaveDraft = useCallback(async () => {
    if (!input.trim()) {
      showError('Cannot save empty draft');
      return;
    }

    const draftId = await saveDraft({
      input_text: input,
      summary_text: null,
      diagnosis_json: diagnosticNotes ? JSON.stringify({ notes: diagnosticNotes }) : null,
      response_text: response || null,
      ticket_id: currentTicketId,
      kb_sources_json: sources.length > 0 ? JSON.stringify(sources) : null,
      is_autosave: false,
    });

    if (draftId) {
      showSuccess('Draft saved');
    }
  }, [input, diagnosticNotes, response, currentTicketId, sources, saveDraft, showError, showSuccess]);

  // Load initial draft if provided
  useEffect(() => {
    if (initialDraft) {
      handleLoadDraft(initialDraft);
    }
  }, [initialDraft, handleLoadDraft]);

  // Trigger autosave on content changes
  useEffect(() => {
    if (input.trim()) {
      triggerAutosave({
        input_text: input,
        summary_text: null,
        diagnosis_json: diagnosticNotes ? JSON.stringify({ notes: diagnosticNotes }) : null,
        response_text: response || null,
        ticket_id: currentTicketId,
        kb_sources_json: sources.length > 0 ? JSON.stringify(sources) : null,
      });
    }
    return () => {
      cancelAutosave();
    };
  }, [input, diagnosticNotes, response, currentTicketId, sources, triggerAutosave, cancelAutosave]);

  const handleCopyResponse = useCallback(async () => {
    if (!response) return;
    try {
      await navigator.clipboard.writeText(response);
      showSuccess('Response copied to clipboard');
    } catch (e) {
      showError('Failed to copy response');
    }
  }, [response, showSuccess, showError]);

  const handleExportResponse = useCallback(async () => {
    if (!response) {
      showError('No response to export');
      return;
    }
    try {
      const saved = await invoke<boolean>('export_draft', {
        responseText: response,
        format: 'Markdown',
      });
      if (saved) {
        showSuccess('Response exported successfully');
      }
    } catch (e) {
      showError(`Export failed: ${e}`);
    }
  }, [response, showSuccess, showError]);

  // Expose functions to parent via ref
  useImperativeHandle(ref, () => ({
    generate: handleGenerate,
    loadDraft: handleLoadDraft,
    saveDraft: handleSaveDraft,
    copyResponse: handleCopyResponse,
    cancelGeneration: handleCancel,
    exportResponse: handleExportResponse,
  }), [handleGenerate, handleLoadDraft, handleSaveDraft, handleCopyResponse, handleCancel, handleExportResponse]);

  return (
    <div className={`draft-tab ${diagnosisCollapsed ? 'diagnosis-collapsed' : ''}`}>
      <div className="draft-panel input-panel">
        <InputPanel
          value={input}
          onChange={setInput}
          ocrText={ocrText}
          onOcrTextChange={setOcrText}
          onGenerate={handleGenerate}
          onClear={handleClear}
          generating={generating}
          modelLoaded={modelLoaded}
          responseLength={responseLength}
          onResponseLengthChange={setResponseLength}
        />
      </div>

      <div className={`draft-panel diagnosis-panel ${diagnosisCollapsed ? 'collapsed' : ''}`}>
        <DiagnosisPanel
          input={input}
          ocrText={ocrText}
          notes={diagnosticNotes}
          onNotesChange={setDiagnosticNotes}
          collapsed={diagnosisCollapsed}
          onToggleCollapse={() => setDiagnosisCollapsed(!diagnosisCollapsed)}
        />
      </div>

      <div className="draft-panel response-panel">
        <ResponsePanel
          response={response}
          streamingText={streamingText}
          isStreaming={isStreaming}
          sources={sources}
          generating={generating}
          onSaveDraft={handleSaveDraft}
          onCancel={handleCancel}
          hasInput={!!input.trim()}
        />
      </div>
    </div>
  );
});
