import { Button } from '../shared/Button';

export interface AiReadinessBannerProps {
  modelLoaded: boolean;
  modelName: string | null;

  kbIndexed: boolean;
  kbDocumentCount: number;
  kbChunkCount: number;

  memoryKernelEnabled: boolean;
  memoryKernelReady: boolean;
  memoryKernelStatus: string;
  memoryKernelDetail: string;

  onRefreshStatus: () => void;
}

type CheckState = 'ok' | 'warn' | 'info';

function getCheckState(opts: { ok: boolean; warn: boolean }): CheckState {
  if (opts.ok) return 'ok';
  if (opts.warn) return 'warn';
  return 'info';
}

function formatCount(n: number): string {
  return n.toLocaleString();
}

export function AiReadinessBanner(props: AiReadinessBannerProps) {
  const {
    modelLoaded,
    modelName,
    kbIndexed,
    kbDocumentCount,
    kbChunkCount,
    memoryKernelEnabled,
    memoryKernelReady,
    memoryKernelStatus,
    memoryKernelDetail,
    onRefreshStatus,
  } = props;

  const modelState: CheckState = getCheckState({ ok: modelLoaded, warn: !modelLoaded });
  const kbState: CheckState = getCheckState({ ok: kbIndexed, warn: !kbIndexed });
  const memoryKernelState: CheckState = memoryKernelEnabled
    ? getCheckState({ ok: memoryKernelReady, warn: !memoryKernelReady })
    : 'info';

  const allReady =
    modelLoaded &&
    kbIndexed &&
    (!memoryKernelEnabled || memoryKernelReady);

  return (
    <section
      className={`ai-readiness-banner ai-readiness-${allReady ? 'ready' : 'needs-attention'}`}
      aria-label="AI readiness"
    >
      <div className="ai-readiness-main">
        <div className="ai-readiness-header">
          <div>
            <h3 className="ai-readiness-title">
              Local AI {allReady ? 'Ready' : 'Needs Attention'}
            </h3>
            <p className="ai-readiness-subtitle">
              Offline-first: ticket/context data stays on this Mac. Contract: no citation, no claim.
            </p>
          </div>
          <div className="ai-readiness-actions">
            <Button size="small" variant="secondary" onClick={onRefreshStatus}>
              Refresh Status
            </Button>
          </div>
        </div>

        <div className="ai-readiness-checklist" role="list" aria-label="AI readiness checks">
          <div className={`ai-readiness-check ai-check-${modelState}`} role="listitem">
            <div className="ai-readiness-check-label">Model</div>
            <div className="ai-readiness-check-value">
              {modelLoaded ? (
                <>Loaded: <span className="ai-readiness-mono">{modelName ?? 'Unknown'}</span></>
              ) : (
                <>No model loaded. Go to <span className="ai-readiness-mono">Settings</span> to download and load a model.</>
              )}
            </div>
          </div>

          <div className={`ai-readiness-check ai-check-${kbState}`} role="listitem">
            <div className="ai-readiness-check-label">Knowledge Base</div>
            <div className="ai-readiness-check-value">
              {kbIndexed ? (
                <>
                  Indexed: <span className="ai-readiness-mono">{formatCount(kbDocumentCount)}</span> docs,
                  {' '}
                  <span className="ai-readiness-mono">{formatCount(kbChunkCount)}</span> chunks.
                </>
              ) : (
                <>
                  Not indexed. Point AssistSupport at your Confluence-export folder in <span className="ai-readiness-mono">Knowledge</span> and run indexing.
                </>
              )}
            </div>
          </div>

          <div className={`ai-readiness-check ai-check-${memoryKernelState}`} role="listitem">
            <div className="ai-readiness-check-label">MemoryKernel</div>
            <div className="ai-readiness-check-value">
              {!memoryKernelEnabled ? (
                <>Disabled (optional). Draft generation remains available without enrichment.</>
              ) : memoryKernelReady ? (
                <>Ready. Enrichment is enabled.</>
              ) : (
                <>
                  Degraded: <span className="ai-readiness-mono">{memoryKernelStatus}</span>
                  {memoryKernelDetail ? <> ({memoryKernelDetail})</> : null}. Deterministic fallback remains active.
                </>
              )}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

