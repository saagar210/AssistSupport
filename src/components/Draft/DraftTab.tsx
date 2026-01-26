import { useState, useCallback, useEffect, forwardRef, useImperativeHandle } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { InputPanel } from './InputPanel';
import { DiagnosisPanel, TreeResult } from './DiagnosisPanel';
import { ResponsePanel } from './ResponsePanel';
import { useLlm } from '../../hooks/useLlm';
import { useDrafts } from '../../hooks/useDrafts';
import { useKb } from '../../hooks/useKb';
import { useToastContext } from '../../contexts/ToastContext';
import { useAppStatus } from '../../contexts/AppStatusContext';
import type { JiraTicket } from '../../hooks/useJira';
import type {
  ContextSource,
  ResponseLength,
  SavedDraft,
  ChecklistItem,
  ChecklistState,
  FirstResponseTone,
  SearchResult,
} from '../../types';
import './DraftTab.css';

export interface DraftTabHandle {
  generate: () => void;
  loadDraft: (draft: SavedDraft) => void;
  saveDraft: () => void;
  copyResponse: () => void;
  cancelGeneration: () => void;
  exportResponse: () => void;
  clearDraft: () => void;
}

interface DraftTabProps {
  initialDraft?: SavedDraft | null;
}

export const DraftTab = forwardRef<DraftTabHandle, DraftTabProps>(function DraftTab({ initialDraft }, ref) {
  const { error: showError, success: showSuccess } = useToastContext();
  const {
    generateStreaming,
    streamingText,
    isStreaming,
    clearStreamingText,
    cancelGeneration,
    generateFirstResponse,
    generateChecklist,
    updateChecklist,
    generateWithContextParams,
  } = useLlm();
  const { saveDraft, triggerAutosave, cancelAutosave } = useDrafts();
  const { search: searchKb } = useKb();
  const appStatus = useAppStatus();

  // Use centralized model status from AppStatusContext
  const modelLoaded = appStatus.llmLoaded;
  const loadedModelName = appStatus.llmModelName;

  const [input, setInput] = useState('');
  const [ocrText, setOcrText] = useState<string | null>(null);
  const [diagnosticNotes, setDiagnosticNotes] = useState('');
  const [treeResult, setTreeResult] = useState<TreeResult | null>(null);
  const [checklistItems, setChecklistItems] = useState<ChecklistItem[]>([]);
  const [checklistCompleted, setChecklistCompleted] = useState<Record<string, boolean>>({});
  const [checklistGenerating, setChecklistGenerating] = useState(false);
  const [checklistUpdating, setChecklistUpdating] = useState(false);
  const [checklistError, setChecklistError] = useState<string | null>(null);
  const [firstResponse, setFirstResponse] = useState('');
  const [firstResponseTone, setFirstResponseTone] = useState<FirstResponseTone>('slack');
  const [firstResponseGenerating, setFirstResponseGenerating] = useState(false);
  const [approvalQuery, setApprovalQuery] = useState('');
  const [approvalResults, setApprovalResults] = useState<SearchResult[]>([]);
  const [approvalSearching, setApprovalSearching] = useState(false);
  const [approvalSummary, setApprovalSummary] = useState('');
  const [approvalSummarizing, setApprovalSummarizing] = useState(false);
  const [approvalSources, setApprovalSources] = useState<ContextSource[]>([]);
  const [approvalError, setApprovalError] = useState<string | null>(null);
  const [response, setResponse] = useState('');
  const [sources, setSources] = useState<ContextSource[]>([]);
  const [responseLength, setResponseLength] = useState<ResponseLength>('Medium');
  const [diagnosisCollapsed, setDiagnosisCollapsed] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [currentTicketId, setCurrentTicketId] = useState<string | null>(null);
  const [currentTicket, setCurrentTicket] = useState<JiraTicket | null>(null);
  const [originalResponse, setOriginalResponse] = useState<string>('');
  const [isResponseEdited, setIsResponseEdited] = useState(false);

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

      // Build tree decisions if available
      const treeDecisions = treeResult ? {
        tree_name: treeResult.treeName,
        path_summary: treeResult.pathSummary,
      } : undefined;

      const result = await generateStreaming(combinedInput, responseLength, {
        treeDecisions,
        diagnosticNotes: diagnosticNotes || undefined,
        jiraTicket: currentTicket || undefined,
      });
      setResponse(result.text);
      setOriginalResponse(result.text);
      setIsResponseEdited(false);
      setSources(result.sources);
    } catch (e) {
      console.error('Generation failed:', e);
      showError(`Generation failed: ${e}`);
    } finally {
      setGenerating(false);
    }
  }, [input, ocrText, responseLength, generating, modelLoaded, treeResult, diagnosticNotes, currentTicket, generateStreaming, clearStreamingText, showError]);

  const handleGenerateFirstResponse = useCallback(async () => {
    if (firstResponseGenerating) return;

    if (!modelLoaded) {
      showError('No model loaded. Go to Settings to load a model.');
      return;
    }

    const ticketFallback = currentTicket
      ? `${currentTicket.summary}${currentTicket.description ? `\n\n${currentTicket.description}` : ''}`
      : '';
    const promptInput = input.trim() || ticketFallback.trim() || ocrText?.trim() || '';
    if (!promptInput) {
      showError('Add ticket details or notes before generating a first response.');
      return;
    }

    setFirstResponseGenerating(true);
    try {
      const result = await generateFirstResponse({
        user_input: promptInput,
        tone: firstResponseTone,
        ocr_text: ocrText ?? undefined,
        jira_ticket: currentTicket ?? undefined,
      });
      setFirstResponse(result.text);
    } catch (e) {
      console.error('First response generation failed:', e);
      showError(`First response failed: ${e}`);
    } finally {
      setFirstResponseGenerating(false);
    }
  }, [input, firstResponseGenerating, modelLoaded, generateFirstResponse, firstResponseTone, ocrText, currentTicket, showError]);

  const handleCopyFirstResponse = useCallback(async () => {
    if (!firstResponse.trim()) return;
    try {
      await navigator.clipboard.writeText(firstResponse);
      showSuccess('First response copied to clipboard');
    } catch (e) {
      showError('Failed to copy first response');
    }
  }, [firstResponse, showSuccess, showError]);

  const handleClearFirstResponse = useCallback(() => {
    setFirstResponse('');
  }, []);

  const handleChecklistGenerate = useCallback(async () => {
    if (checklistGenerating) return;

    if (!modelLoaded) {
      showError('No model loaded. Go to Settings to load a model.');
      return;
    }

    const ticketFallback = currentTicket
      ? `${currentTicket.summary}${currentTicket.description ? `\n\n${currentTicket.description}` : ''}`
      : '';
    const promptInput = input.trim() || ticketFallback.trim() || ocrText?.trim() || '';
    if (!promptInput) {
      setChecklistError('Add ticket details or notes before generating a checklist.');
      return;
    }

    setChecklistGenerating(true);
    setChecklistError(null);
    try {
      const treeDecisions = treeResult ? {
        tree_name: treeResult.treeName,
        path_summary: treeResult.pathSummary,
      } : undefined;

      const result = await generateChecklist({
        user_input: promptInput,
        ocr_text: ocrText ?? undefined,
        diagnostic_notes: diagnosticNotes || undefined,
        tree_decisions: treeDecisions,
        jira_ticket: currentTicket ?? undefined,
      });

      setChecklistItems(result.items);
      setChecklistCompleted({});
    } catch (e) {
      console.error('Checklist generation failed:', e);
      setChecklistError(`Checklist failed: ${e}`);
    } finally {
      setChecklistGenerating(false);
    }
  }, [input, checklistGenerating, modelLoaded, treeResult, ocrText, diagnosticNotes, currentTicket, generateChecklist, showError]);

  const handleChecklistUpdate = useCallback(async () => {
    if (!checklistItems.length || checklistUpdating) return;

    if (!modelLoaded) {
      showError('No model loaded. Go to Settings to load a model.');
      return;
    }

    const ticketFallback = currentTicket
      ? `${currentTicket.summary}${currentTicket.description ? `\n\n${currentTicket.description}` : ''}`
      : '';
    const promptInput = input.trim() || ticketFallback.trim() || ocrText?.trim() || '';
    if (!promptInput) {
      setChecklistError('Add ticket details or notes before updating the checklist.');
      return;
    }

    setChecklistUpdating(true);
    setChecklistError(null);
    try {
      const treeDecisions = treeResult ? {
        tree_name: treeResult.treeName,
        path_summary: treeResult.pathSummary,
      } : undefined;

      const completedIds = Object.keys(checklistCompleted).filter(id => checklistCompleted[id]);
      const checklist: ChecklistState = {
        items: checklistItems,
        completed_ids: completedIds,
      };

      const result = await updateChecklist({
        user_input: promptInput,
        ocr_text: ocrText ?? undefined,
        diagnostic_notes: diagnosticNotes || undefined,
        tree_decisions: treeDecisions,
        jira_ticket: currentTicket ?? undefined,
        checklist,
      });

      const updatedCompleted: Record<string, boolean> = {};
      for (const item of result.items) {
        if (checklistCompleted[item.id]) {
          updatedCompleted[item.id] = true;
        }
      }

      setChecklistItems(result.items);
      setChecklistCompleted(updatedCompleted);
    } catch (e) {
      console.error('Checklist update failed:', e);
      setChecklistError(`Checklist update failed: ${e}`);
    } finally {
      setChecklistUpdating(false);
    }
  }, [checklistItems, checklistUpdating, modelLoaded, input, ocrText, diagnosticNotes, treeResult, currentTicket, checklistCompleted, updateChecklist, showError]);

  const handleChecklistToggle = useCallback((id: string) => {
    setChecklistCompleted(prev => ({
      ...prev,
      [id]: !prev[id],
    }));
  }, []);

  const handleChecklistClear = useCallback(() => {
    setChecklistItems([]);
    setChecklistCompleted({});
    setChecklistError(null);
  }, []);

  const handleApprovalSearch = useCallback(async () => {
    if (!approvalQuery.trim()) {
      setApprovalError('Enter a search term to look up approvals.');
      return;
    }

    setApprovalSearching(true);
    setApprovalError(null);
    try {
      const results = await searchKb(approvalQuery.trim(), 5);
      setApprovalResults(results);
    } catch (e) {
      console.error('Approval search failed:', e);
      setApprovalError('Approval search failed.');
    } finally {
      setApprovalSearching(false);
    }
  }, [approvalQuery, searchKb]);

  const handleApprovalSummarize = useCallback(async () => {
    if (!approvalQuery.trim()) {
      setApprovalError('Enter a search term to summarize approvals.');
      return;
    }

    if (!modelLoaded) {
      showError('No model loaded. Go to Settings to load a model.');
      return;
    }

    setApprovalSummarizing(true);
    setApprovalError(null);
    try {
      const prompt = `Summarize the approval steps and owner(s) for: ${approvalQuery.trim()}. Keep it concise. If sources do not mention it, say so.`;
      const result = await generateWithContextParams({
        user_input: prompt,
        kb_limit: 5,
        response_length: 'Short',
      });

      setApprovalSummary(result.text);
      setApprovalSources(result.sources);
    } catch (e) {
      console.error('Approval summary failed:', e);
      setApprovalError('Approval summary failed.');
    } finally {
      setApprovalSummarizing(false);
    }
  }, [approvalQuery, modelLoaded, generateWithContextParams, showError]);

  const handleClear = useCallback(() => {
    setInput('');
    setOcrText(null);
    setDiagnosticNotes('');
    setTreeResult(null);
    setChecklistItems([]);
    setChecklistCompleted({});
    setChecklistError(null);
    setChecklistGenerating(false);
    setChecklistUpdating(false);
    setFirstResponse('');
    setFirstResponseTone('slack');
    setFirstResponseGenerating(false);
    setApprovalQuery('');
    setApprovalResults([]);
    setApprovalSummary('');
    setApprovalSources([]);
    setApprovalError(null);
    setApprovalSearching(false);
    setApprovalSummarizing(false);
    setResponse('');
    setOriginalResponse('');
    setIsResponseEdited(false);
    setSources([]);
    setCurrentTicketId(null);
    setCurrentTicket(null);
  }, []);

  const handleResponseChange = useCallback((text: string) => {
    setResponse(text);
    setIsResponseEdited(text !== originalResponse);
  }, [originalResponse]);

  const handleTreeComplete = useCallback((result: TreeResult) => {
    setTreeResult(result);
  }, []);

  const handleTreeClear = useCallback(() => {
    setTreeResult(null);
  }, []);

  const handleCancel = useCallback(async () => {
    await cancelGeneration();
    setGenerating(false);
    // Keep the streaming text that was generated so far
    if (streamingText) {
      setResponse(streamingText);
      setOriginalResponse(streamingText);
      setIsResponseEdited(false);
    }
  }, [cancelGeneration, streamingText]);

  const handleLoadDraft = useCallback((draft: SavedDraft) => {
    setInput(draft.input_text);
    const loadedResponse = draft.response_text || '';
    setResponse(loadedResponse);
    setOriginalResponse(loadedResponse);
    setIsResponseEdited(false);
    if (draft.diagnosis_json) {
      try {
        const diagData = JSON.parse(draft.diagnosis_json);
        setDiagnosticNotes(diagData.notes || '');
        setTreeResult(diagData.treeResult || null);
        const checklistState = diagData.checklist;
        if (checklistState?.items) {
          setChecklistItems(checklistState.items);
          const completed: Record<string, boolean> = {};
          for (const id of checklistState.completed_ids || []) {
            completed[id] = true;
          }
          setChecklistCompleted(completed);
        } else {
          setChecklistItems([]);
          setChecklistCompleted({});
        }
        setChecklistError(null);

        const firstResponseState = diagData.firstResponse;
        if (firstResponseState?.text) {
          setFirstResponse(firstResponseState.text);
          setFirstResponseTone(firstResponseState.tone || 'slack');
        } else {
          setFirstResponse('');
          setFirstResponseTone('slack');
        }

        const approvalState = diagData.approval;
        if (approvalState) {
          setApprovalQuery(approvalState.query || '');
          setApprovalSummary(approvalState.summary || '');
          setApprovalSources(approvalState.sources || []);
        } else {
          setApprovalQuery('');
          setApprovalSummary('');
          setApprovalSources([]);
        }
        setApprovalResults([]);
        setApprovalError(null);
      } catch {
        setDiagnosticNotes('');
        setTreeResult(null);
        setChecklistItems([]);
        setChecklistCompleted({});
        setChecklistError(null);
        setFirstResponse('');
        setFirstResponseTone('slack');
        setApprovalQuery('');
        setApprovalSummary('');
        setApprovalSources([]);
        setApprovalResults([]);
        setApprovalError(null);
      }
    } else {
      setDiagnosticNotes('');
      setTreeResult(null);
      setChecklistItems([]);
      setChecklistCompleted({});
      setChecklistError(null);
      setFirstResponse('');
      setFirstResponseTone('slack');
      setApprovalQuery('');
      setApprovalSummary('');
      setApprovalSources([]);
      setApprovalResults([]);
      setApprovalError(null);
    }
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

  const buildDiagnosisJson = useCallback(() => {
    const completedIds = Object.keys(checklistCompleted).filter(id => checklistCompleted[id]);
    const checklistState = checklistItems.length > 0
      ? { items: checklistItems, completed_ids: completedIds }
      : null;
    const firstResponseState = firstResponse.trim()
      ? { text: firstResponse, tone: firstResponseTone }
      : null;
    const approvalState = (approvalQuery.trim() || approvalSummary.trim() || approvalSources.length > 0)
      ? { query: approvalQuery, summary: approvalSummary, sources: approvalSources }
      : null;

    const diagnosisData: Record<string, unknown> = {};
    if (diagnosticNotes.trim()) {
      diagnosisData.notes = diagnosticNotes;
    }
    if (treeResult) {
      diagnosisData.treeResult = treeResult;
    }
    if (checklistState) {
      diagnosisData.checklist = checklistState;
    }
    if (firstResponseState) {
      diagnosisData.firstResponse = firstResponseState;
    }
    if (approvalState) {
      diagnosisData.approval = approvalState;
    }

    return Object.keys(diagnosisData).length > 0
      ? JSON.stringify(diagnosisData)
      : null;
  }, [
    checklistCompleted,
    checklistItems,
    firstResponse,
    firstResponseTone,
    approvalQuery,
    approvalSummary,
    approvalSources,
    diagnosticNotes,
    treeResult,
  ]);

  const handleSaveDraft = useCallback(async () => {
    if (!input.trim()) {
      showError('Cannot save empty draft');
      return;
    }

    const diagnosisData = buildDiagnosisJson();

    const draftId = await saveDraft({
      input_text: input,
      summary_text: null,
      diagnosis_json: diagnosisData,
      response_text: response || null,
      ticket_id: currentTicketId,
      kb_sources_json: sources.length > 0 ? JSON.stringify(sources) : null,
      is_autosave: false,
      model_name: loadedModelName,
    });

    if (draftId) {
      showSuccess('Draft saved');
    }
  }, [input, buildDiagnosisJson, response, currentTicketId, sources, saveDraft, showError, showSuccess]);

  // Load initial draft if provided
  useEffect(() => {
    if (initialDraft) {
      handleLoadDraft(initialDraft);
    }
  }, [initialDraft, handleLoadDraft]);

  useEffect(() => {
    if (!approvalQuery.trim()) {
      setApprovalResults([]);
      setApprovalSummary('');
      setApprovalSources([]);
      setApprovalError(null);
    }
  }, [approvalQuery]);

  // Trigger autosave on content changes
  useEffect(() => {
    if (input.trim()) {
      const diagnosisData = buildDiagnosisJson();

      triggerAutosave({
        input_text: input,
        summary_text: null,
        diagnosis_json: diagnosisData,
        response_text: response || null,
        ticket_id: currentTicketId,
        kb_sources_json: sources.length > 0 ? JSON.stringify(sources) : null,
        model_name: loadedModelName,
      });
    }
    return () => {
      cancelAutosave();
    };
  }, [input, buildDiagnosisJson, response, currentTicketId, sources, loadedModelName, triggerAutosave, cancelAutosave]);

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
    clearDraft: handleClear,
  }), [handleGenerate, handleLoadDraft, handleSaveDraft, handleCopyResponse, handleCancel, handleExportResponse, handleClear]);

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
          ticketId={currentTicketId}
          onTicketIdChange={setCurrentTicketId}
          ticket={currentTicket}
          onTicketChange={setCurrentTicket}
          firstResponse={firstResponse}
          onFirstResponseChange={setFirstResponse}
          firstResponseTone={firstResponseTone}
          onFirstResponseToneChange={setFirstResponseTone}
          onGenerateFirstResponse={handleGenerateFirstResponse}
          onCopyFirstResponse={handleCopyFirstResponse}
          onClearFirstResponse={handleClearFirstResponse}
          firstResponseGenerating={firstResponseGenerating}
        />
      </div>

      <div className={`draft-panel diagnosis-panel ${diagnosisCollapsed ? 'collapsed' : ''}`}>
        <DiagnosisPanel
          input={input}
          ocrText={ocrText}
          notes={diagnosticNotes}
          onNotesChange={setDiagnosticNotes}
          treeResult={treeResult}
          onTreeComplete={handleTreeComplete}
          onTreeClear={handleTreeClear}
          checklistItems={checklistItems}
          checklistCompleted={checklistCompleted}
          checklistGenerating={checklistGenerating}
          checklistUpdating={checklistUpdating}
          checklistError={checklistError}
          onChecklistToggle={handleChecklistToggle}
          onChecklistGenerate={handleChecklistGenerate}
          onChecklistUpdate={handleChecklistUpdate}
          onChecklistClear={handleChecklistClear}
          approvalQuery={approvalQuery}
          onApprovalQueryChange={setApprovalQuery}
          approvalResults={approvalResults}
          approvalSearching={approvalSearching}
          approvalSummary={approvalSummary}
          approvalSummarizing={approvalSummarizing}
          approvalSources={approvalSources}
          onApprovalSearch={handleApprovalSearch}
          onApprovalSummarize={handleApprovalSummarize}
          approvalError={approvalError}
          modelLoaded={modelLoaded}
          hasTicket={!!currentTicket}
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
          onResponseChange={handleResponseChange}
          isEdited={isResponseEdited}
          modelName={loadedModelName}
        />
      </div>
    </div>
  );
});
