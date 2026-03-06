import { useCallback, useEffect, useState } from 'react';
import type { JiraTicket } from '../../hooks/useJira';
import type {
  ChecklistItem,
  ChecklistState,
  ContextSource,
  FirstResponseTone,
  SearchResult,
  TreeDecisions,
} from '../../types';

interface UseDraftDiagnosisWorkflowParams {
  modelLoaded: boolean;
  input: string;
  ocrText: string | null;
  diagnosticNotes: string;
  treeDecisions: TreeDecisions | undefined;
  currentTicket: JiraTicket | null;
  searchKb: (query: string, topK?: number) => Promise<SearchResult[]>;
  generateFirstResponse: (params: {
    user_input: string;
    tone: FirstResponseTone;
    ocr_text?: string;
    jira_ticket?: JiraTicket;
  }) => Promise<{ text: string }>;
  generateChecklist: (params: {
    user_input: string;
    ocr_text?: string;
    diagnostic_notes?: string;
    tree_decisions?: TreeDecisions;
    jira_ticket?: JiraTicket;
  }) => Promise<{ items: ChecklistItem[] }>;
  updateChecklist: (params: {
    user_input: string;
    ocr_text?: string;
    diagnostic_notes?: string;
    tree_decisions?: TreeDecisions;
    jira_ticket?: JiraTicket;
    checklist: ChecklistState;
  }) => Promise<{ items: ChecklistItem[] }>;
  generateWithContextParams: (params: {
    user_input: string;
    kb_limit: number;
    response_length: 'Short' | 'Medium' | 'Long';
  }) => Promise<{ text: string; sources: ContextSource[] }>;
  showError: (message: string) => void;
  showSuccess: (message: string) => void;
}

interface DiagnosisHydratePayload {
  checklist?: ChecklistState | null;
  firstResponse?: { text?: string; tone?: FirstResponseTone } | null;
  approval?: { query?: string; summary?: string; sources?: ContextSource[] } | null;
}

export function useDraftDiagnosisWorkflow({
  modelLoaded,
  input,
  ocrText,
  diagnosticNotes,
  treeDecisions,
  currentTicket,
  searchKb,
  generateFirstResponse,
  generateChecklist,
  updateChecklist,
  generateWithContextParams,
  showError,
  showSuccess,
}: UseDraftDiagnosisWorkflowParams) {
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

  const buildPromptInput = useCallback(() => {
    const ticketFallback = currentTicket
      ? `${currentTicket.summary}${currentTicket.description ? `\n\n${currentTicket.description}` : ''}`
      : '';
    return input.trim() || ticketFallback.trim() || ocrText?.trim() || '';
  }, [currentTicket, input, ocrText]);

  const handleGenerateFirstResponse = useCallback(async () => {
    if (firstResponseGenerating) return;

    if (!modelLoaded) {
      showError('No model loaded. Go to Settings to load a model.');
      return;
    }

    const promptInput = buildPromptInput();
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
  }, [
    firstResponseGenerating,
    modelLoaded,
    buildPromptInput,
    generateFirstResponse,
    firstResponseTone,
    ocrText,
    currentTicket,
    showError,
  ]);

  const handleCopyFirstResponse = useCallback(async () => {
    if (!firstResponse.trim()) return;
    try {
      await navigator.clipboard.writeText(firstResponse);
      showSuccess('First response copied to clipboard');
    } catch {
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

    const promptInput = buildPromptInput();
    if (!promptInput) {
      setChecklistError('Add ticket details or notes before generating a checklist.');
      return;
    }

    setChecklistGenerating(true);
    setChecklistError(null);
    try {
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
  }, [
    checklistGenerating,
    modelLoaded,
    showError,
    buildPromptInput,
    generateChecklist,
    ocrText,
    diagnosticNotes,
    treeDecisions,
    currentTicket,
  ]);

  const handleChecklistUpdate = useCallback(async () => {
    if (!checklistItems.length || checklistUpdating) return;

    if (!modelLoaded) {
      showError('No model loaded. Go to Settings to load a model.');
      return;
    }

    const promptInput = buildPromptInput();
    if (!promptInput) {
      setChecklistError('Add ticket details or notes before updating the checklist.');
      return;
    }

    setChecklistUpdating(true);
    setChecklistError(null);
    try {
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
  }, [
    checklistItems,
    checklistUpdating,
    modelLoaded,
    showError,
    buildPromptInput,
    checklistCompleted,
    updateChecklist,
    ocrText,
    diagnosticNotes,
    treeDecisions,
    currentTicket,
  ]);

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

  const resetDiagnosisWorkflow = useCallback(() => {
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
  }, []);

  const hydrateDiagnosisWorkflow = useCallback((payload: DiagnosisHydratePayload | null | undefined) => {
    if (!payload) {
      resetDiagnosisWorkflow();
      return;
    }

    const checklistState = payload.checklist;
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

    const firstResponseState = payload.firstResponse;
    if (firstResponseState?.text) {
      setFirstResponse(firstResponseState.text);
      setFirstResponseTone(firstResponseState.tone || 'slack');
    } else {
      setFirstResponse('');
      setFirstResponseTone('slack');
    }

    const approvalState = payload.approval;
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
  }, [resetDiagnosisWorkflow]);

  useEffect(() => {
    if (!approvalQuery.trim()) {
      setApprovalResults([]);
      setApprovalSummary('');
      setApprovalSources([]);
      setApprovalError(null);
    }
  }, [approvalQuery]);

  return {
    checklistItems,
    checklistCompleted,
    checklistGenerating,
    checklistUpdating,
    checklistError,
    firstResponse,
    firstResponseTone,
    firstResponseGenerating,
    approvalQuery,
    approvalResults,
    approvalSearching,
    approvalSummary,
    approvalSummarizing,
    approvalSources,
    approvalError,
    setFirstResponse,
    setFirstResponseTone,
    setApprovalQuery,
    setApprovalSummary,
    setApprovalSources,
    handleGenerateFirstResponse,
    handleCopyFirstResponse,
    handleClearFirstResponse,
    handleChecklistGenerate,
    handleChecklistUpdate,
    handleChecklistToggle,
    handleChecklistClear,
    handleApprovalSearch,
    handleApprovalSummarize,
    resetDiagnosisWorkflow,
    hydrateDiagnosisWorkflow,
  };
}
