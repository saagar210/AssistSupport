import { useState, useCallback, useEffect, forwardRef, useImperativeHandle } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { InputPanel } from './InputPanel';
import { DiagnosisPanel, TreeResult } from './DiagnosisPanel';
import { ResponsePanel } from './ResponsePanel';
import { AlternativePanel } from './AlternativePanel';
import { SaveAsTemplateModal } from './SaveAsTemplateModal';
import { SavedResponsesSuggestion } from './SavedResponsesSuggestion';
import { ConversationThread, ConversationEntry } from './ConversationThread';
import { ConversationInput } from './ConversationInput';
import { useLlm } from '../../hooks/useLlm';
import { useDrafts } from '../../hooks/useDrafts';
import { useKb } from '../../hooks/useKb';
import { useAnalytics } from '../../hooks/useAnalytics';
import { useAlternatives } from '../../hooks/useAlternatives';
import { useSavedResponses } from '../../hooks/useSavedResponses';
import { useMemoryKernelEnrichment } from '../../hooks/useMemoryKernelEnrichment';
import { useToastContext } from '../../contexts/ToastContext';
import { useAppStatus } from '../../contexts/AppStatusContext';
import type { JiraTicket } from '../../hooks/useJira';
import type {
  ContextSource,
  ConfidenceAssessment,
  GenerationMetrics,
  GroundedClaim,
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
  onNavigateToSource?: (searchQuery: string) => void;
}

export const DraftTab = forwardRef<DraftTabHandle, DraftTabProps>(function DraftTab({ initialDraft, onNavigateToSource }, ref) {
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
  const { saveDraft, triggerAutosave, cancelAutosave, templates, loadTemplates } = useDrafts();
  const { search: searchKb } = useKb();
  const { enrichDiagnosticNotes } = useMemoryKernelEnrichment();
  const { logEvent } = useAnalytics();
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
  const [metrics, setMetrics] = useState<GenerationMetrics | null>(null);
  const [confidence, setConfidence] = useState<ConfidenceAssessment | null>(null);
  const [grounding, setGrounding] = useState<GroundedClaim[]>([]);
  const [responseLength, setResponseLength] = useState<ResponseLength>('Medium');
  const [diagnosisCollapsed, setDiagnosisCollapsed] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [currentTicketId, setCurrentTicketId] = useState<string | null>(null);
  const [currentTicket, setCurrentTicket] = useState<JiraTicket | null>(null);
  const [originalResponse, setOriginalResponse] = useState<string>('');
  const [isResponseEdited, setIsResponseEdited] = useState(false);
  const [savedDraftId, setSavedDraftId] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'panels' | 'conversation'>(() => {
    return (localStorage.getItem('draft-view-mode') as 'panels' | 'conversation') || 'panels';
  });
  const [conversationEntries, setConversationEntries] = useState<ConversationEntry[]>([]);

  // Alternatives & saved responses
  const { alternatives, loadAlternatives, saveAlternative, chooseAlternative } = useAlternatives();
  const { suggestions, findSimilar, saveAsTemplate, incrementUsage } = useSavedResponses();
  const [generatingAlternative, setGeneratingAlternative] = useState(false);
  const [showTemplateModal, setShowTemplateModal] = useState(false);
  const [templateModalRating, setTemplateModalRating] = useState<number | undefined>(undefined);
  const [suggestionsDismissed, setSuggestionsDismissed] = useState(false);

  const handleGenerate = useCallback(async () => {
    if (!input.trim() || generating) return;

    if (!modelLoaded) {
      showError('No model loaded. Go to Settings to load a model.');
      return;
    }

    setGenerating(true);
    setResponse(''); // Clear previous response
    clearStreamingText(); // Clear streaming buffer
    setConfidence(null);
    setGrounding([]);
    try {
      const combinedInput = ocrText ? `${input}\n\n[Screenshot OCR Text]:\n${ocrText}` : input;
      const enrichment = await enrichDiagnosticNotes(combinedInput, diagnosticNotes || undefined);
      logEvent('memorykernel_enrichment_attempted', {
        applied: enrichment.enrichmentApplied,
        status: enrichment.status,
        fallback_reason: enrichment.fallbackReason,
        machine_error_code: enrichment.machineErrorCode,
      });
      if (!enrichment.enrichmentApplied) {
        console.info('MemoryKernel enrichment skipped:', enrichment.message);
      }

      // Build tree decisions if available
      const treeDecisions = treeResult ? {
        tree_name: treeResult.treeName,
        path_summary: treeResult.pathSummary,
      } : undefined;

      const result = await generateStreaming(combinedInput, responseLength, {
        treeDecisions,
        diagnosticNotes: enrichment.diagnosticNotes,
        jiraTicket: currentTicket || undefined,
      });
      setResponse(result.text);
      setOriginalResponse(result.text);
      setIsResponseEdited(false);
      setSources(result.sources);
      setMetrics(result.metrics ?? null);
      setConfidence(result.confidence ?? null);
      setGrounding(result.grounding ?? []);
      logEvent('response_generated', {
        response_length: responseLength,
        tokens_generated: result.tokens_generated,
        duration_ms: result.duration_ms,
        sources_count: result.sources.length,
      });
    } catch (e) {
      console.error('Generation failed:', e);
      showError(`Generation failed: ${e}`);
    } finally {
      setGenerating(false);
    }
  }, [input, ocrText, responseLength, generating, modelLoaded, treeResult, diagnosticNotes, currentTicket, generateStreaming, clearStreamingText, showError, logEvent, enrichDiagnosticNotes]);

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

  const handleApplyTemplate = useCallback((content: string) => {
    setResponse(content);
  }, []);

  const handleGenerateAlternative = useCallback(async () => {
    if (!response || generating || generatingAlternative || !modelLoaded) return;

    setGeneratingAlternative(true);
    try {
      const combinedInput = ocrText ? `${input}\n\n[Screenshot OCR Text]:\n${ocrText}` : input;
      const treeDecisions = treeResult ? {
        tree_name: treeResult.treeName,
        path_summary: treeResult.pathSummary,
      } : undefined;

      const result = await generateStreaming(combinedInput, responseLength, {
        treeDecisions,
        diagnosticNotes: diagnosticNotes || undefined,
        jiraTicket: currentTicket || undefined,
      });

      // Save the alternative
      if (savedDraftId) {
        await saveAlternative(savedDraftId, response, result.text, {
          sourcesJson: result.sources.length > 0 ? JSON.stringify(result.sources) : undefined,
          metricsJson: result.metrics ? JSON.stringify(result.metrics) : undefined,
        });
        await loadAlternatives(savedDraftId);
      }

      logEvent('alternative_generated', {
        draft_id: savedDraftId,
        tokens_generated: result.tokens_generated,
      });
    } catch (e) {
      console.error('Alternative generation failed:', e);
      showError(`Alternative generation failed: ${e}`);
    } finally {
      setGeneratingAlternative(false);
    }
  }, [response, generating, generatingAlternative, modelLoaded, input, ocrText, responseLength, treeResult, diagnosticNotes, currentTicket, generateStreaming, savedDraftId, saveAlternative, loadAlternatives, logEvent, showError]);

  const handleChooseAlternative = useCallback(async (alternativeId: string, choice: 'original' | 'alternative') => {
    await chooseAlternative(alternativeId, choice);
    if (savedDraftId) {
      await loadAlternatives(savedDraftId);
    }
  }, [chooseAlternative, loadAlternatives, savedDraftId]);

  const handleUseAlternative = useCallback((text: string) => {
    setResponse(text);
    setOriginalResponse(text);
    setIsResponseEdited(false);
  }, []);

  const handleSaveAsTemplate = useCallback((rating: number) => {
    setTemplateModalRating(rating);
    setShowTemplateModal(true);
  }, []);

  const handleTemplateModalSave = useCallback(async (
    name: string,
    category: string | null,
    content: string,
    variablesJson: string | null,
  ): Promise<boolean> => {
    const id = await saveAsTemplate(name, content, {
      sourceDraftId: savedDraftId ?? undefined,
      sourceRating: templateModalRating,
      category: category ?? undefined,
      variablesJson: variablesJson ?? undefined,
    });
    if (id) {
      showSuccess('Response saved as template');
      return true;
    }
    showError('Failed to save template');
    return false;
  }, [saveAsTemplate, savedDraftId, templateModalRating, showSuccess, showError]);

  const handleSuggestionApply = useCallback((content: string, templateId: string) => {
    setResponse(content);
    setOriginalResponse(content);
    setIsResponseEdited(false);
    incrementUsage(templateId);
    setSuggestionsDismissed(true);
  }, [incrementUsage]);

  const handleSuggestionDismiss = useCallback(() => {
    setSuggestionsDismissed(true);
  }, []);

  // Find similar saved responses when input changes
  useEffect(() => {
    if (input.trim().length >= 10) {
      setSuggestionsDismissed(false);
      findSimilar(input);
    }
  }, [input, findSimilar]);

  // Load alternatives when draft is loaded/saved
  useEffect(() => {
    if (savedDraftId) {
      loadAlternatives(savedDraftId);
    }
  }, [savedDraftId, loadAlternatives]);

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
    setMetrics(null);
    setConfidence(null);
    setGrounding([]);
    setCurrentTicketId(null);
    setCurrentTicket(null);
    setSavedDraftId(null);
    setConversationEntries([]);
    setGeneratingAlternative(false);
    setShowTemplateModal(false);
    setTemplateModalRating(undefined);
    setSuggestionsDismissed(false);
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

  const handleViewModeChange = useCallback((mode: 'panels' | 'conversation') => {
    setViewMode(mode);
    localStorage.setItem('draft-view-mode', mode);
  }, []);

  const handleConversationSubmit = useCallback(async (text: string) => {
    if (!modelLoaded) return;

    // Add input entry
    const inputEntry: ConversationEntry = {
      id: crypto.randomUUID(),
      type: 'input',
      timestamp: new Date().toISOString(),
      content: text,
    };
    setConversationEntries(prev => [...prev, inputEntry]);
    setInput(text);

    // Generate
    setGenerating(true);
    setResponse('');
    clearStreamingText();
    setConfidence(null);
    setGrounding([]);
    try {
      const result = await generateStreaming(text, responseLength, {});
      setResponse(result.text);
      setOriginalResponse(result.text);
      setIsResponseEdited(false);
      setSources(result.sources);
      setConfidence(result.confidence ?? null);
      setGrounding(result.grounding ?? []);

      // Add response entry
      const responseEntry: ConversationEntry = {
        id: crypto.randomUUID(),
        type: 'response',
        timestamp: new Date().toISOString(),
        content: result.text,
        sources: result.sources,
        metrics: result.metrics ? {
          tokens_per_second: result.metrics.tokens_per_second,
          sources_used: result.metrics.sources_used,
          word_count: result.metrics.word_count,
        } : undefined,
      };
      setConversationEntries(prev => [...prev, responseEntry]);
    } catch (e) {
      console.error('Generation failed:', e);
    } finally {
      setGenerating(false);
    }
  }, [modelLoaded, responseLength, generateStreaming, clearStreamingText]);

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
    setSavedDraftId(draft.id);
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

        const trustState = diagData.trust;
        setConfidence(trustState?.confidence || null);
        setGrounding(trustState?.grounding || []);
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
        setConfidence(null);
        setGrounding([]);
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
      setConfidence(null);
      setGrounding([]);
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
    diagnosticNotes,
    treeResult,
    confidence,
    grounding,
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
      showSuccess('Draft saved');
    }
  }, [input, buildDiagnosisJson, response, currentTicketId, sources, saveDraft, showError, showSuccess, loadedModelName]);

  // Load initial draft if provided
  useEffect(() => {
    if (initialDraft) {
      handleLoadDraft(initialDraft);
    }
  }, [initialDraft, handleLoadDraft]);

  // Load templates on mount
  useEffect(() => {
    loadTemplates();
  }, [loadTemplates]);

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

  const isConversation = viewMode === 'conversation';

  const viewToggle = (
    <div className="draft-view-header">
      <div className="view-toggle">
        <button className={`view-btn ${!isConversation ? 'active' : ''}`} onClick={() => handleViewModeChange('panels')}>Panels</button>
        <button className={`view-btn ${isConversation ? 'active' : ''}`} onClick={() => handleViewModeChange('conversation')}>Conversation</button>
      </div>
    </div>
  );

  if (isConversation) {
    return (
      <div className="draft-tab conversation-mode">
        {viewToggle}
        <ConversationThread
          entries={conversationEntries}
          streamingText={streamingText}
          isStreaming={isStreaming}
        />
        <ConversationInput
          onSubmit={handleConversationSubmit}
          generating={generating}
          modelLoaded={modelLoaded}
          responseLength={responseLength}
          onResponseLengthChange={setResponseLength}
          onCancel={handleCancel}
        />
      </div>
    );
  }

  return (
    <div className={`draft-tab ${diagnosisCollapsed ? 'diagnosis-collapsed' : ''}`}>
      {viewToggle}
      <div className="draft-panels-container">
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
            templates={templates}
            onApplyTemplate={handleApplyTemplate}
            onNavigateToSource={onNavigateToSource}
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
          {!suggestionsDismissed && suggestions.length > 0 && !response && (
            <SavedResponsesSuggestion
              suggestions={suggestions}
              onApply={handleSuggestionApply}
              onDismiss={handleSuggestionDismiss}
            />
          )}
          <ResponsePanel
            response={response}
            streamingText={streamingText}
            isStreaming={isStreaming}
            sources={sources}
            generating={generating}
            metrics={metrics}
            confidence={confidence}
            grounding={grounding}
            draftId={savedDraftId}
            onSaveDraft={handleSaveDraft}
            onCancel={handleCancel}
            hasInput={!!input.trim()}
            onResponseChange={handleResponseChange}
            isEdited={isResponseEdited}
            modelName={loadedModelName}
            onGenerateAlternative={handleGenerateAlternative}
            generatingAlternative={generatingAlternative}
            ticketKey={currentTicketId}
            onSaveAsTemplate={handleSaveAsTemplate}
          />
          {alternatives.length > 0 && response && !generating && !isStreaming && (
            <AlternativePanel
              alternatives={alternatives}
              onChoose={handleChooseAlternative}
              onUseAlternative={handleUseAlternative}
            />
          )}
        </div>
      </div>

      {showTemplateModal && response && (
        <SaveAsTemplateModal
          content={response}
          sourceDraftId={savedDraftId ?? undefined}
          sourceRating={templateModalRating}
          onSave={handleTemplateModalSave}
          onClose={() => setShowTemplateModal(false)}
        />
      )}
    </div>
  );
});
