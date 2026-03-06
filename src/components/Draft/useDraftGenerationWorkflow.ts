import { useCallback } from 'react';
import type { MutableRefObject } from 'react';
import type { JiraTicket } from '../../hooks/useJira';
import type {
  ConfidenceAssessment,
  ContextSource,
  GenerationMetrics,
  GroundedClaim,
  ResponseLength,
} from '../../types';
import type { TreeResult } from './DiagnosisPanel';
import type { DraftLifecycleEventType } from './workflowLifecycle';

interface GenerateResult {
  text: string;
  sources: ContextSource[];
  metrics?: GenerationMetrics | null;
  confidence?: ConfidenceAssessment | null;
  grounding?: GroundedClaim[] | null;
  tokens_generated: number;
  duration_ms: number;
}

interface UseDraftGenerationWorkflowParams {
  input: string;
  ocrText: string | null;
  responseLength: ResponseLength;
  generating: boolean;
  modelLoaded: boolean;
  treeResult: TreeResult | null;
  diagnosticNotes: string;
  currentTicket: JiraTicket | null;
  savedDraftId: string | null;
  currentTicketId: string | null;
  response: string;
  generatingAlternative: boolean;
  generateStreaming: (
    userInput: string,
    responseLength: ResponseLength,
    options: {
      treeDecisions?: {
        tree_name: string;
        path_summary: string;
      };
      diagnosticNotes?: string;
      jiraTicket?: JiraTicket;
    },
  ) => Promise<GenerateResult>;
  enrichDiagnosticNotes: (
    userInput: string,
    diagnosticNotes?: string,
  ) => Promise<{
    diagnosticNotes?: string;
    enrichmentApplied: boolean;
    status: string;
    message: string;
    fallbackReason: string | null;
    machineErrorCode: string | null;
  }>;
  clearStreamingText: () => void;
  showError: (message: string) => void;
  logEvent: (eventType: string, payload?: Record<string, unknown>) => void;
  countWords: (text: string) => number;
  emitLifecycle: (
    type: DraftLifecycleEventType,
    payload?: {
      draftId?: string | null;
      hasResponse?: boolean;
    },
  ) => void;
  firstDraftStartMsRef: MutableRefObject<number | null>;
  setGenerating: (value: boolean) => void;
  setResponse: (value: string) => void;
  setOriginalResponse: (value: string) => void;
  setIsResponseEdited: (value: boolean) => void;
  setSources: (value: ContextSource[]) => void;
  setMetrics: (value: GenerationMetrics | null) => void;
  setConfidence: (value: ConfidenceAssessment | null) => void;
  setGrounding: (value: GroundedClaim[]) => void;
  saveAlternative: (
    draftId: string,
    originalText: string,
    alternativeText: string,
    opts?: { sourcesJson?: string; metricsJson?: string },
  ) => Promise<unknown>;
  loadAlternatives: (draftId: string) => Promise<unknown>;
  setGeneratingAlternative: (value: boolean) => void;
}

export function useDraftGenerationWorkflow({
  input,
  ocrText,
  responseLength,
  generating,
  modelLoaded,
  treeResult,
  diagnosticNotes,
  currentTicket,
  savedDraftId,
  currentTicketId,
  response,
  generatingAlternative,
  generateStreaming,
  enrichDiagnosticNotes,
  clearStreamingText,
  showError,
  logEvent,
  countWords,
  emitLifecycle,
  firstDraftStartMsRef,
  setGenerating,
  setResponse,
  setOriginalResponse,
  setIsResponseEdited,
  setSources,
  setMetrics,
  setConfidence,
  setGrounding,
  saveAlternative,
  loadAlternatives,
  setGeneratingAlternative,
}: UseDraftGenerationWorkflowParams) {
  const handleGenerate = useCallback(async () => {
    if (!input.trim() || generating) return;

    if (!modelLoaded) {
      showError('No model loaded. Go to Settings to load a model.');
      return;
    }

    setGenerating(true);
    emitLifecycle('generation_started');
    if (firstDraftStartMsRef.current === null) {
      firstDraftStartMsRef.current = Date.now();
    }
    setResponse('');
    clearStreamingText();
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
      emitLifecycle('generation_succeeded', { hasResponse: result.text.trim().length > 0 });
      const responseWordCount = countWords(result.text);
      const timeToDraftMs = firstDraftStartMsRef.current ? Date.now() - firstDraftStartMsRef.current : null;
      logEvent('response_generated', {
        response_length: responseLength,
        tokens_generated: result.tokens_generated,
        duration_ms: result.duration_ms,
        sources_count: result.sources.length,
      });
      logEvent('response_quality_snapshot', {
        draft_id: savedDraftId,
        word_count: responseWordCount,
        edit_ratio: 0,
        time_to_draft_ms: timeToDraftMs,
        has_ticket: !!currentTicketId,
        has_tree_path: !!treeResult,
        has_notes: !!enrichment.diagnosticNotes?.trim(),
      });
    } catch (e) {
      console.error('Generation failed:', e);
      emitLifecycle('generation_failed');
      showError(`Generation failed: ${e}`);
    } finally {
      setGenerating(false);
    }
  }, [
    input,
    generating,
    modelLoaded,
    setGenerating,
    emitLifecycle,
    firstDraftStartMsRef,
    setResponse,
    clearStreamingText,
    setConfidence,
    setGrounding,
    ocrText,
    enrichDiagnosticNotes,
    diagnosticNotes,
    logEvent,
    treeResult,
    generateStreaming,
    responseLength,
    currentTicket,
    setOriginalResponse,
    setIsResponseEdited,
    setSources,
    setMetrics,
    countWords,
    savedDraftId,
    currentTicketId,
    showError,
  ]);

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
  }, [
    response,
    generating,
    generatingAlternative,
    modelLoaded,
    setGeneratingAlternative,
    ocrText,
    input,
    treeResult,
    generateStreaming,
    responseLength,
    diagnosticNotes,
    currentTicket,
    savedDraftId,
    saveAlternative,
    loadAlternatives,
    logEvent,
    showError,
  ]);

  return {
    handleGenerate,
    handleGenerateAlternative,
  };
}
