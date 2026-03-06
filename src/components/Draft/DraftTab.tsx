import { useState, useCallback, useEffect, useReducer, forwardRef, useImperativeHandle, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { TreeResult } from './DiagnosisPanel';
import { SaveAsTemplateModal } from './SaveAsTemplateModal';
import { ConversationThread, ConversationEntry } from './ConversationThread';
import { ConversationInput } from './ConversationInput';
import { DraftWorkflowStrip } from './DraftWorkflowStrip';
import { DraftPanelsLayout } from './DraftPanelsLayout';
import { useDraftDiagnosisWorkflow } from './useDraftDiagnosisWorkflow';
import { useDraftGenerationWorkflow } from './useDraftGenerationWorkflow';
import { useDraftPersistenceWorkflow } from './useDraftPersistenceWorkflow';
import { useLlm } from '../../hooks/useLlm';
import { useDrafts } from '../../hooks/useDrafts';
import { useKb } from '../../hooks/useKb';
import { useAnalytics } from '../../hooks/useAnalytics';
import { useAlternatives } from '../../hooks/useAlternatives';
import { useSavedResponses } from '../../hooks/useSavedResponses';
import { useMemoryKernelEnrichment } from '../../hooks/useMemoryKernelEnrichment';
import { useToastContext } from '../../contexts/ToastContext';
import { useAppStatus } from '../../contexts/AppStatusContext';
import { AiReadinessBanner } from './AiReadinessBanner';
import { calculateEditRatio, countWords } from '../../features/analytics/qualityMetrics';
import type { JiraTicket } from '../../hooks/useJira';
import type {
  ContextSource,
  ConfidenceAssessment,
  GenerationMetrics,
  GroundedClaim,
  ResponseLength,
  SavedDraft,
} from '../../types';
import {
  createInitialDraftLifecycleState,
  reduceDraftLifecycle,
  type DraftLifecycleEventType,
} from './workflowLifecycle';
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
  revampModeEnabled?: boolean;
}

type DraftPanelDensityMode = 'balanced' | 'focus-intake' | 'focus-response';

const DRAFT_PANEL_DENSITY_STORAGE_KEY = 'draft-panel-density-mode';

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  const tag = target.tagName.toLowerCase();
  return (
    tag === 'input' ||
    tag === 'textarea' ||
    tag === 'select' ||
    target.isContentEditable
  );
}

export const DraftTab = forwardRef<DraftTabHandle, DraftTabProps>(function DraftTab(
  { initialDraft, onNavigateToSource, revampModeEnabled = false },
  ref
) {
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
  const [panelDensityMode, setPanelDensityMode] = useState<DraftPanelDensityMode>(() => {
    const stored = localStorage.getItem(DRAFT_PANEL_DENSITY_STORAGE_KEY);
    if (stored === 'balanced' || stored === 'focus-intake' || stored === 'focus-response') {
      return stored;
    }
    return 'balanced';
  });
  const [conversationEntries, setConversationEntries] = useState<ConversationEntry[]>([]);
  const [lifecycle, dispatchLifecycle] = useReducer(
    reduceDraftLifecycle,
    undefined,
    createInitialDraftLifecycleState,
  );

  const emitLifecycle = useCallback(
    (
      type: DraftLifecycleEventType,
      payload?: {
        draftId?: string | null;
        hasResponse?: boolean;
      },
    ) => {
      dispatchLifecycle({
        type,
        at: new Date().toISOString(),
        payload,
      });
    },
    [],
  );

  // Alternatives & saved responses
  const { alternatives, loadAlternatives, saveAlternative, chooseAlternative } = useAlternatives();
  const { suggestions, findSimilar, saveAsTemplate, incrementUsage } = useSavedResponses();
  const [generatingAlternative, setGeneratingAlternative] = useState(false);
  const [showTemplateModal, setShowTemplateModal] = useState(false);
  const [templateModalRating, setTemplateModalRating] = useState<number | undefined>(undefined);
  const [suggestionsDismissed, setSuggestionsDismissed] = useState(false);
  const firstDraftStartMsRef = useRef<number | null>(null);

  const treeDecisions = treeResult ? {
    tree_name: treeResult.treeName,
    path_summary: treeResult.pathSummary,
  } : undefined;

  const {
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
  } = useDraftDiagnosisWorkflow({
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
  });

  const { handleGenerate, handleGenerateAlternative } = useDraftGenerationWorkflow({
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
  });

  const handleApplyTemplate = useCallback((content: string) => {
    setResponse(content);
  }, []);

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
    resetDiagnosisWorkflow();
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
    emitLifecycle('draft_cleared');
    setGeneratingAlternative(false);
    setShowTemplateModal(false);
    setTemplateModalRating(undefined);
    setSuggestionsDismissed(false);
    firstDraftStartMsRef.current = null;
  }, [emitLifecycle, resetDiagnosisWorkflow]);

  const handleResponseChange = useCallback((text: string) => {
    setResponse(text);
    setIsResponseEdited(text !== originalResponse);
    emitLifecycle('response_updated', { hasResponse: text.trim().length > 0 });
  }, [originalResponse, emitLifecycle]);

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

  const handlePanelDensityModeChange = useCallback((mode: DraftPanelDensityMode) => {
    setPanelDensityMode(mode);
    localStorage.setItem(DRAFT_PANEL_DENSITY_STORAGE_KEY, mode);
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
    emitLifecycle('generation_started');
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
      emitLifecycle('generation_succeeded', { hasResponse: result.text.trim().length > 0 });

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
      emitLifecycle('generation_failed');
    } finally {
      setGenerating(false);
    }
  }, [modelLoaded, responseLength, generateStreaming, clearStreamingText, emitLifecycle]);

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

  useEffect(() => {
    if (viewMode !== 'panels') {
      return;
    }
    const handleKeydown = (event: KeyboardEvent) => {
      if (!event.metaKey || event.altKey || event.ctrlKey) {
        return;
      }
      if (isEditableTarget(event.target)) {
        return;
      }

      if (event.key === '1') {
        event.preventDefault();
        handlePanelDensityModeChange('balanced');
      } else if (event.key === '2') {
        event.preventDefault();
        handlePanelDensityModeChange('focus-intake');
      } else if (event.key === '3') {
        event.preventDefault();
        handlePanelDensityModeChange('focus-response');
      }
    };

    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  }, [viewMode, handlePanelDensityModeChange]);

  const { handleSaveDraft, handleLoadDraft } = useDraftPersistenceWorkflow({
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
    invokeGetJiraTicket: (ticketKey: string) => invoke<JiraTicket>('get_jira_ticket', { ticketKey }),
  });

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

  const handleCopyResponse = useCallback(async () => {
    if (!response) return;
    try {
      const mode = confidence?.mode ?? 'answer';
      const hasCitations = sources.length > 0;
      const copyAllowed = mode === 'answer' && hasCitations;

      if (!copyAllowed) {
        const reason = window.prompt(
          'Copy override required. This response is missing citations or is not in answer mode.\n\nEnter a reason to proceed (will be logged locally):',
        );
        if (!reason || !reason.trim()) {
          showError('Copy cancelled (reason required).');
          return;
        }
        await invoke('audit_response_copy_override', {
          reason: reason.trim(),
          confidenceMode: confidence?.mode ?? null,
          sourcesCount: sources.length,
        });
      }
      await navigator.clipboard.writeText(response);
      emitLifecycle('handoff_completed');
      logEvent('response_copied', {
        draft_id: savedDraftId,
        word_count: countWords(response),
        is_edited: isResponseEdited,
        edit_ratio: Number(calculateEditRatio(originalResponse, response).toFixed(3)),
      });
      showSuccess('Response copied to clipboard');
    } catch (e) {
      showError('Failed to copy response');
    }
  }, [
    response,
    confidence?.mode,
    sources.length,
    showSuccess,
    showError,
    logEvent,
    savedDraftId,
    isResponseEdited,
    originalResponse,
    emitLifecycle,
  ]);

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
        emitLifecycle('handoff_completed');
        showSuccess('Response exported successfully');
      }
    } catch (e) {
      showError(`Export failed: ${e}`);
    }
  }, [response, showSuccess, showError, emitLifecycle]);

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
  const checklistCompletedCount = checklistItems.reduce((count, item) => {
    return checklistCompleted[item.id] ? count + 1 : count;
  }, 0);
  const responseWordCount = countWords(response);
  const responseEditRatio = calculateEditRatio(originalResponse, response);

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
      <div className={['draft-tab', 'conversation-mode', revampModeEnabled ? 'draft-tab--revamp' : ''].filter(Boolean).join(' ')}>
        {viewToggle}
        <AiReadinessBanner
          modelLoaded={modelLoaded}
          modelName={loadedModelName}
          kbIndexed={appStatus.kbIndexed}
          kbDocumentCount={appStatus.kbDocumentCount}
          kbChunkCount={appStatus.kbChunkCount}
          memoryKernelEnabled={appStatus.memoryKernelFeatureEnabled}
          memoryKernelReady={appStatus.memoryKernelReady}
          memoryKernelStatus={appStatus.memoryKernelStatus}
          memoryKernelDetail={appStatus.memoryKernelDetail}
          onRefreshStatus={() => {
            void appStatus.refresh();
          }}
        />
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
    <div
      className={[
        'draft-tab',
        `panel-density-${panelDensityMode}`,
        diagnosisCollapsed ? 'diagnosis-collapsed' : '',
        revampModeEnabled ? 'draft-tab--revamp' : '',
      ]
        .filter(Boolean)
        .join(' ')}
    >
      {viewToggle}
      <AiReadinessBanner
        modelLoaded={modelLoaded}
        modelName={loadedModelName}
        kbIndexed={appStatus.kbIndexed}
        kbDocumentCount={appStatus.kbDocumentCount}
        kbChunkCount={appStatus.kbChunkCount}
        memoryKernelEnabled={appStatus.memoryKernelFeatureEnabled}
        memoryKernelReady={appStatus.memoryKernelReady}
        memoryKernelStatus={appStatus.memoryKernelStatus}
        memoryKernelDetail={appStatus.memoryKernelDetail}
        onRefreshStatus={() => {
          void appStatus.refresh();
        }}
      />
      <DraftWorkflowStrip
        inputWordCount={countWords(input)}
        hasTicket={Boolean(currentTicketId)}
        treeCompleted={Boolean(treeResult)}
        checklistCompletedCount={checklistCompletedCount}
        checklistTotal={checklistItems.length}
        responseWordCount={responseWordCount}
        isResponseEdited={isResponseEdited}
        responseEditRatioPercent={Math.round(responseEditRatio * 100)}
        hasResponse={Boolean(response?.trim())}
        handoffTouched={lifecycle.handoffTouched}
        panelDensityMode={panelDensityMode}
        onPanelDensityModeChange={handlePanelDensityModeChange}
        onGenerateFirstResponse={() => {
          void handleGenerateFirstResponse();
        }}
        onChecklistGenerate={() => {
          void handleChecklistGenerate();
        }}
        onGenerateFullResponse={() => {
          void handleGenerate();
        }}
        onSaveDraft={() => {
          void handleSaveDraft();
        }}
        modelLoaded={modelLoaded}
        firstResponseGenerating={firstResponseGenerating}
        checklistGenerating={checklistGenerating}
        generating={generating}
        canGenerateChecklist={Boolean(input.trim() || ocrText?.trim() || currentTicket)}
        canGenerateResponse={Boolean(input.trim())}
        canSave={Boolean(input.trim())}
      />
      <DraftPanelsLayout
        diagnosisCollapsed={diagnosisCollapsed}
        onToggleDiagnosisCollapse={() => setDiagnosisCollapsed(!diagnosisCollapsed)}
        inputPanelProps={{
          value: input,
          onChange: setInput,
          ocrText,
          onOcrTextChange: setOcrText,
          onGenerate: handleGenerate,
          onClear: handleClear,
          generating,
          modelLoaded,
          responseLength,
          onResponseLengthChange: setResponseLength,
          ticketId: currentTicketId,
          onTicketIdChange: setCurrentTicketId,
          ticket: currentTicket,
          onTicketChange: setCurrentTicket,
          firstResponse,
          onFirstResponseChange: setFirstResponse,
          firstResponseTone,
          onFirstResponseToneChange: setFirstResponseTone,
          onGenerateFirstResponse: handleGenerateFirstResponse,
          onCopyFirstResponse: handleCopyFirstResponse,
          onClearFirstResponse: handleClearFirstResponse,
          firstResponseGenerating,
          templates,
          onApplyTemplate: handleApplyTemplate,
          onNavigateToSource,
        }}
        diagnosisPanelProps={{
          input,
          ocrText,
          notes: diagnosticNotes,
          onNotesChange: setDiagnosticNotes,
          treeResult,
          onTreeComplete: handleTreeComplete,
          onTreeClear: handleTreeClear,
          checklistItems,
          checklistCompleted,
          checklistGenerating,
          checklistUpdating,
          checklistError,
          onChecklistToggle: handleChecklistToggle,
          onChecklistGenerate: handleChecklistGenerate,
          onChecklistUpdate: handleChecklistUpdate,
          onChecklistClear: handleChecklistClear,
          approvalQuery,
          onApprovalQueryChange: setApprovalQuery,
          approvalResults,
          approvalSearching,
          approvalSummary,
          approvalSummarizing,
          approvalSources,
          onApprovalSearch: handleApprovalSearch,
          onApprovalSummarize: handleApprovalSummarize,
          approvalError,
          modelLoaded,
          hasTicket: !!currentTicket,
        }}
        responsePanelProps={{
          response,
          streamingText,
          isStreaming,
          sources,
          generating,
          metrics,
          confidence,
          grounding,
          draftId: savedDraftId,
          onSaveDraft: handleSaveDraft,
          onCancel: handleCancel,
          hasInput: !!input.trim(),
          onResponseChange: handleResponseChange,
          isEdited: isResponseEdited,
          modelName: loadedModelName,
          onGenerateAlternative: handleGenerateAlternative,
          generatingAlternative,
          ticketKey: currentTicketId,
          onSaveAsTemplate: handleSaveAsTemplate,
        }}
        showSuggestions={!suggestionsDismissed && suggestions.length > 0 && !response}
        suggestionProps={{
          suggestions,
          onApply: handleSuggestionApply,
          onDismiss: handleSuggestionDismiss,
        }}
        showAlternatives={alternatives.length > 0 && !!response && !generating && !isStreaming}
        alternativesProps={{
          alternatives,
          onChoose: handleChooseAlternative,
          onUseAlternative: handleUseAlternative,
        }}
      />

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
