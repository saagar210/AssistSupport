export type DraftLifecycleStage =
  | 'input'
  | 'generating'
  | 'review'
  | 'saved'
  | 'handoff';

export type DraftLifecycleEventType =
  | 'draft_loaded'
  | 'generation_started'
  | 'generation_succeeded'
  | 'generation_failed'
  | 'response_updated'
  | 'draft_saved'
  | 'handoff_completed'
  | 'draft_cleared';

export interface DraftLifecycleEvent {
  type: DraftLifecycleEventType;
  at: string;
  payload?: {
    draftId?: string | null;
    hasResponse?: boolean;
  };
}

export interface DraftLifecycleState {
  stage: DraftLifecycleStage;
  hasResponse: boolean;
  handoffTouched: boolean;
  currentDraftId: string | null;
  lastEventType: DraftLifecycleEventType | null;
  lastUpdatedAt: string | null;
}

export function createInitialDraftLifecycleState(): DraftLifecycleState {
  return {
    stage: 'input',
    hasResponse: false,
    handoffTouched: false,
    currentDraftId: null,
    lastEventType: null,
    lastUpdatedAt: null,
  };
}

function nextStageFromResponse(hasResponse: boolean): DraftLifecycleStage {
  return hasResponse ? 'review' : 'input';
}

export function reduceDraftLifecycle(
  state: DraftLifecycleState,
  event: DraftLifecycleEvent,
): DraftLifecycleState {
  switch (event.type) {
    case 'draft_loaded': {
      const hasResponse = Boolean(event.payload?.hasResponse);
      const draftId = event.payload?.draftId ?? null;
      return {
        ...state,
        stage: hasResponse && draftId ? 'saved' : nextStageFromResponse(hasResponse),
        hasResponse,
        handoffTouched: false,
        currentDraftId: draftId,
        lastEventType: event.type,
        lastUpdatedAt: event.at,
      };
    }
    case 'generation_started':
      return {
        ...state,
        stage: 'generating',
        handoffTouched: false,
        lastEventType: event.type,
        lastUpdatedAt: event.at,
      };
    case 'generation_succeeded': {
      const hasResponse = event.payload?.hasResponse ?? true;
      return {
        ...state,
        stage: nextStageFromResponse(hasResponse),
        hasResponse,
        handoffTouched: false,
        lastEventType: event.type,
        lastUpdatedAt: event.at,
      };
    }
    case 'generation_failed':
      return {
        ...state,
        stage: nextStageFromResponse(state.hasResponse),
        lastEventType: event.type,
        lastUpdatedAt: event.at,
      };
    case 'response_updated': {
      const hasResponse = Boolean(event.payload?.hasResponse);
      return {
        ...state,
        stage: nextStageFromResponse(hasResponse),
        hasResponse,
        handoffTouched: hasResponse ? state.handoffTouched : false,
        lastEventType: event.type,
        lastUpdatedAt: event.at,
      };
    }
    case 'draft_saved':
      return {
        ...state,
        stage: state.hasResponse ? 'saved' : 'input',
        currentDraftId: event.payload?.draftId ?? state.currentDraftId,
        lastEventType: event.type,
        lastUpdatedAt: event.at,
      };
    case 'handoff_completed':
      return {
        ...state,
        stage: state.hasResponse ? 'handoff' : state.stage,
        handoffTouched: state.hasResponse,
        lastEventType: event.type,
        lastUpdatedAt: event.at,
      };
    case 'draft_cleared':
      return {
        ...createInitialDraftLifecycleState(),
        lastEventType: event.type,
        lastUpdatedAt: event.at,
      };
    default:
      return state;
  }
}

export function replayDraftLifecycle(
  events: DraftLifecycleEvent[],
  initialState: DraftLifecycleState = createInitialDraftLifecycleState(),
): DraftLifecycleState {
  return events.reduce((state, event) => reduceDraftLifecycle(state, event), initialState);
}
