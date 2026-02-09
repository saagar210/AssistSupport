import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '../../test/test-utils';
import { AiReadinessBanner } from './AiReadinessBanner';

describe('AiReadinessBanner', () => {
  it('renders ready state when model + KB are ready and MemoryKernel is disabled', () => {
    render(
      <AiReadinessBanner
        modelLoaded={true}
        modelName="Llama 3.1 8B Instruct Q4_K_M"
        kbIndexed={true}
        kbDocumentCount={12}
        kbChunkCount={345}
        memoryKernelEnabled={false}
        memoryKernelReady={false}
        memoryKernelStatus="unknown"
        memoryKernelDetail=""
        onRefreshStatus={vi.fn()}
      />
    );

    expect(screen.getByText(/Local AI Ready/i)).toBeInTheDocument();
    expect(screen.getByText(/Loaded:/i)).toBeInTheDocument();
    expect(screen.getByText(/Indexed:/i)).toBeInTheDocument();
    expect(screen.getByText(/Disabled \(optional\)/i)).toBeInTheDocument();
  });

  it('renders guidance when no model is loaded', () => {
    render(
      <AiReadinessBanner
        modelLoaded={false}
        modelName={null}
        kbIndexed={true}
        kbDocumentCount={1}
        kbChunkCount={10}
        memoryKernelEnabled={false}
        memoryKernelReady={false}
        memoryKernelStatus="unknown"
        memoryKernelDetail=""
        onRefreshStatus={vi.fn()}
      />
    );

    expect(screen.getByText(/Local AI Needs Attention/i)).toBeInTheDocument();
    expect(screen.getByText(/No model loaded/i)).toBeInTheDocument();
    const modelRow = screen.getByText('Model').closest('[role="listitem"]');
    expect(modelRow?.textContent).toContain('Go to Settings');
  });

  it('renders KB + MemoryKernel warnings when not ready', () => {
    render(
      <AiReadinessBanner
        modelLoaded={true}
        modelName="Some Model"
        kbIndexed={false}
        kbDocumentCount={0}
        kbChunkCount={0}
        memoryKernelEnabled={true}
        memoryKernelReady={false}
        memoryKernelStatus="schema_unavailable"
        memoryKernelDetail="Service unavailable"
        onRefreshStatus={vi.fn()}
      />
    );

    expect(screen.getByText(/Not indexed/i)).toBeInTheDocument();
    expect(screen.getByText(/Degraded:/i)).toBeInTheDocument();
    expect(screen.getByText(/Deterministic fallback remains active/i)).toBeInTheDocument();
  });
});
