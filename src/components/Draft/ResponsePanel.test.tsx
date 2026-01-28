import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { ResponsePanel } from './ResponsePanel';
import { ToastProvider } from '../../contexts/ToastContext';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core');

const mockInvoke = vi.mocked(invoke);

// Wrapper with ToastProvider
function renderWithProviders(ui: React.ReactElement) {
  return render(<ToastProvider>{ui}</ToastProvider>);
}

describe('ResponsePanel', () => {
  const defaultProps = {
    response: '',
    streamingText: '',
    isStreaming: false,
    sources: [],
    generating: false,
    metrics: null,
    hasInput: true,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders placeholder when no response', () => {
    renderWithProviders(<ResponsePanel {...defaultProps} />);
    expect(screen.getByText('Generated response will appear here...')).toBeInTheDocument();
  });

  it('renders response text when provided', () => {
    renderWithProviders(
      <ResponsePanel {...defaultProps} response="This is the generated response" />
    );
    expect(screen.getByText('This is the generated response')).toBeInTheDocument();
  });

  it('shows word count when response is present', () => {
    renderWithProviders(
      <ResponsePanel {...defaultProps} response="One two three four five" />
    );
    expect(screen.getByText('5 words')).toBeInTheDocument();
  });

  it('shows streaming text with cursor', () => {
    renderWithProviders(
      <ResponsePanel {...defaultProps} streamingText="Streaming..." isStreaming={true} />
    );
    expect(screen.getByText('Streaming...')).toBeInTheDocument();
    expect(screen.getByText('|')).toBeInTheDocument(); // Cursor
  });

  it('shows generating indicator when generating without streaming', () => {
    renderWithProviders(<ResponsePanel {...defaultProps} generating={true} />);
    expect(screen.getByText('Generating response...')).toBeInTheDocument();
  });

  it('auto-shows sources panel when response completes with sources', () => {
    const sources = [
      {
        chunk_id: 'c1',
        document_id: 'd1',
        file_path: '/kb/doc.md',
        title: 'Test Doc',
        heading_path: 'Section',
        score: 0.95,
      },
    ];
    renderWithProviders(
      <ResponsePanel {...defaultProps} response="Response" sources={sources} />
    );
    // Sources auto-show when response completes with sources
    expect(screen.getByText('Hide Sources')).toBeInTheDocument();
    expect(screen.getByText('Knowledge Base Sources')).toBeInTheDocument();
  });

  it('toggles sources panel visibility', () => {
    const sources = [
      {
        chunk_id: 'c1',
        document_id: 'd1',
        file_path: '/kb/doc.md',
        title: 'Test Doc',
        heading_path: 'Section',
        score: 0.95,
      },
    ];
    renderWithProviders(
      <ResponsePanel {...defaultProps} response="Response" sources={sources} />
    );

    // Auto-shown initially
    expect(screen.getByText('Knowledge Base Sources')).toBeInTheDocument();

    // Click to hide
    fireEvent.click(screen.getByText('Hide Sources'));
    expect(screen.queryByText('Knowledge Base Sources')).not.toBeInTheDocument();

    // Click to show again
    fireEvent.click(screen.getByText('Sources (1)'));
    expect(screen.getByText('Knowledge Base Sources')).toBeInTheDocument();
  });

  it('copy button copies to clipboard', async () => {
    const mockClipboard = { writeText: vi.fn().mockResolvedValue(undefined) };
    Object.assign(navigator, { clipboard: mockClipboard });

    renderWithProviders(
      <ResponsePanel {...defaultProps} response="Copy me" />
    );

    fireEvent.click(screen.getByText('Copy'));

    await waitFor(() => {
      expect(mockClipboard.writeText).toHaveBeenCalledWith('Copy me');
    });
  });

  it('export dropdown shows options', () => {
    renderWithProviders(
      <ResponsePanel {...defaultProps} response="Export me" />
    );

    // Click export button
    fireEvent.click(screen.getByText('Export ▾'));

    // Check options appear
    expect(screen.getByText('Markdown (.md)')).toBeInTheDocument();
    expect(screen.getByText('Plain Text (.txt)')).toBeInTheDocument();
    expect(screen.getByText('HTML (.html)')).toBeInTheDocument();
  });

  it('export calls backend with correct format', async () => {
    mockInvoke.mockResolvedValue(true);

    renderWithProviders(
      <ResponsePanel {...defaultProps} response="Export me" />
    );

    // Open dropdown and click Markdown
    fireEvent.click(screen.getByText('Export ▾'));
    fireEvent.click(screen.getByText('Markdown (.md)'));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('export_draft', {
        responseText: 'Export me',
        format: 'Markdown',
      });
    });
  });

  it('disables copy and export when no response', () => {
    renderWithProviders(<ResponsePanel {...defaultProps} />);

    // Query by role to get the actual button elements
    const copyButton = screen.getByRole('button', { name: /copy/i });
    const exportButton = screen.getByRole('button', { name: /export/i });

    expect(copyButton).toBeDisabled();
    expect(exportButton).toBeDisabled();
  });

  it('calls onCancel when cancel button clicked during generation', () => {
    const onCancel = vi.fn();
    renderWithProviders(
      <ResponsePanel {...defaultProps} generating={true} onCancel={onCancel} />
    );

    fireEvent.click(screen.getByText('Cancel'));
    expect(onCancel).toHaveBeenCalled();
  });

  it('calls onSaveDraft when save button clicked', () => {
    const onSaveDraft = vi.fn();
    renderWithProviders(
      <ResponsePanel
        {...defaultProps}
        response="Save me"
        onSaveDraft={onSaveDraft}
        hasInput={true}
      />
    );

    fireEvent.click(screen.getByText('Save Draft'));
    expect(onSaveDraft).toHaveBeenCalled();
  });
});
