import { useCallback, useEffect } from 'react';
import type { JiraTicket } from '../../hooks/useJira';
import type {
  ChecklistState,
  ConfidenceAssessment,
  ContextSource,
  FirstResponseTone,
  GroundedClaim,
} from '../../types';
import type { TreeResult } from './DiagnosisPanel';
import type { DraftLifecycleEventType } from './workflowLifecycle';

interface UseDraftPersistenceWorkflowParams {
  input: string;
  diagnosticNotes: string;
  treeResult: TreeResult | null;
  checklistItems: Array<{ id: string; text: string }>;
  checklistCompleted: Record<string, boolean>;
  firstResponse: string;
  firstResponseTone: FirstResponseTone;
  approvalQuery: string;
  approvalSummary: string;
  approvalSources: ContextSource[];
  confidence: ConfidenceAssessment | null;
  grounding: GroundedClaim[];
  response: string;
  currentTicketId: string | null;
  sources: ContextSource[];
  loadedModelName: string | null;
  originalResponse: string;
  isResponseEdited: boolean;
  saveDraft: (params: {
    input_text: string;
    summary_text: string | null;
    diagnosis_json: string | null;
    response_text: string | null;
    ticket_id: string | null;
    kb_sources_json: string | null;
    is_autosave: boolean;
    model_name: string | null;
  }) => Promise<string | null>;
  triggerAutosave: (params: {
    input_text: string;
    summary_text: string | null;
    diagnosis_json: string | null;
    response_text: string | null;
    ticket_id: string | null;
    kb_sources_json: string | null;
    model_name: string | null;
  }) => void;
  cancelAutosave: () => void;
  showError: (message: string) => void;
  showSuccess: (message: string) => void;
  countWords: (text: string) => number;
  calculateEditRatio: (original: string, edited: string) => number;
  logEvent: (eventType: string, payload?: Record<string, unknown>) => void;
  emitLifecycle: (
    type: DraftLifecycleEventType,
    payload?: {
      draftId?: string | null;
      hasResponse?: boolean;
    },
  ) => void;
  setSavedDraftId: (id: string | null) => void;
  hydrateDiagnosisWorkflow: (payload: {
    checklist?: ChecklistState | null;
    firstResponse?: { text?: string; tone?: FirstResponseTone } | null;
    approval?: { query?: string; summary?: string; sources?: ContextSource[] } | null;
  } | null | undefined) => void;
  resetDiagnosisWorkflow: () => void;
  setInput: (value: string) => void;
  setResponse: (value: string) => void;
  setOriginalResponse: (value: string) => void;
  setIsResponseEdited: (value: boolean) => void;
  setDiagnosticNotes: (value: string) => void;
  setTreeResult: (value: TreeResult | null) => void;
  setConfidence: (value: ConfidenceAssessment | null) => void;
  setGrounding: (value: GroundedClaim[]) => void;
  setCurrentTicketId: (value: string | null) => void;
  setCurrentTicket: (value: JiraTicket | null) => void;
  setSources: (value: ContextSource[]) => void;
  setOcrText: (value: string | null) => void;
  invokeGetJiraTicket: (ticketKey: string) => Promise<JiraTicket>;
}

export function useDraftPersistenceWorkflow({
  input,
  diagnosticNotes,
  treeResult,
  checklistItems,
  checklistCompleted,
  firstResponse,
  firstResponseTone,
  approvalQuery,
  approvalSummary,
  approvalSources,
  confidence,
  grounding,
  response,
  currentTicketId,
  sources,
  loadedModelName,
  originalResponse,
  isResponseEdited,
  saveDraft,
  triggerAutosave,
  cancelAutosave,
  showError,
  showSuccess,
  countWords,
  calculateEditRatio,
  logEvent,
  emitLifecycle,
  setSavedDraftId,
  hydrateDiagnosisWorkflow,
  resetDiagnosisWorkflow,
  setInput,
  setResponse,
  setOriginalResponse,
  setIsResponseEdited,
  setDiagnosticNotes,
  setTreeResult,
  setConfidence,
  setGrounding,
  setCurrentTicketId,
  setCurrentTicket,
  setSources,
  setOcrText,
  invokeGetJiraTicket,
}: UseDraftPersistenceWorkflowParams) {
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
    const trustState = (confidence || grounding.length > 0)
      ? { confidence, grounding }
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
    if (trustState) {
      diagnosisData.trust = trustState;
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
    confidence,
    grounding,
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
      setSavedDraftId(draftId);
      emitLifecycle('draft_saved', { draftId });
      const responseWordCount = countWords(response);
      const editRatio = calculateEditRatio(originalResponse, response);
      logEvent('response_saved', {
        draft_id: draftId,
        word_count: responseWordCount,
        is_edited: isResponseEdited,
        edit_ratio: Number(editRatio.toFixed(3)),
      });
      showSuccess('Draft saved');
    }
  }, [
    input,
    showError,
    buildDiagnosisJson,
    saveDraft,
    response,
    currentTicketId,
    sources,
    loadedModelName,
    setSavedDraftId,
    emitLifecycle,
    countWords,
    calculateEditRatio,
    originalResponse,
    isResponseEdited,
    logEvent,
    showSuccess,
  ]);

  const handleLoadDraft = useCallback((draft: {
    id: string;
    input_text: string;
    response_text: string | null;
    diagnosis_json: string | null;
    ticket_id: string | null;
    kb_sources_json: string | null;
  }) => {
    setInput(draft.input_text);
    const loadedResponse = draft.response_text || '';
    setResponse(loadedResponse);
    setOriginalResponse(loadedResponse);
    setIsResponseEdited(false);
    setSavedDraftId(draft.id);
    if (draft.diagnosis_json) {
      try {
        const diagData = JSON.parse(draft.diagnosis_json);
        setDiagnosticNotes(diagData.notes || '');
        setTreeResult(diagData.treeResult || null);
        hydrateDiagnosisWorkflow(diagData);

        const trustState = diagData.trust;
        setConfidence(trustState?.confidence || null);
        setGrounding(trustState?.grounding || []);
      } catch {
        setDiagnosticNotes('');
        setTreeResult(null);
        resetDiagnosisWorkflow();
        setConfidence(null);
        setGrounding([]);
      }
    } else {
      setDiagnosticNotes('');
      setTreeResult(null);
      resetDiagnosisWorkflow();
      setConfidence(null);
      setGrounding([]);
    }
    const draftTicketId = draft.ticket_id?.trim() || null;
    setCurrentTicketId(draftTicketId);
    if (draftTicketId) {
      void invokeGetJiraTicket(draftTicketId)
        .then((ticket) => setCurrentTicket(ticket))
        .catch(() => setCurrentTicket(null));
    } else {
      setCurrentTicket(null);
    }
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
    emitLifecycle('draft_loaded', {
      draftId: draft.id,
      hasResponse: loadedResponse.trim().length > 0,
    });
  }, [
    setInput,
    setResponse,
    setOriginalResponse,
    setIsResponseEdited,
    setSavedDraftId,
    setDiagnosticNotes,
    setTreeResult,
    hydrateDiagnosisWorkflow,
    setConfidence,
    setGrounding,
    resetDiagnosisWorkflow,
    setCurrentTicketId,
    invokeGetJiraTicket,
    setCurrentTicket,
    setSources,
    setOcrText,
    emitLifecycle,
  ]);

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
  }, [
    input,
    buildDiagnosisJson,
    response,
    currentTicketId,
    sources,
    loadedModelName,
    triggerAutosave,
    cancelAutosave,
  ]);

  return {
    handleSaveDraft,
    handleLoadDraft,
  };
}
