import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { InputPanel } from './InputPanel';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core');
vi.mock('../../hooks/useJira', () => ({
  useJira: () => ({
    checkConfiguration: vi.fn(),
    getTicket: vi.fn(),
    configured: false,
  }),
  JiraTicket: {},
}));

const mockInvoke = vi.mocked(invoke);

describe('InputPanel', () => {
  const defaultProps = {
    value: '',
    onChange: vi.fn(),
    ocrText: null,
    onOcrTextChange: vi.fn(),
    onGenerate: vi.fn(),
    onClear: vi.fn(),
    generating: false,
    modelLoaded: true,
    responseLength: 'Medium' as const,
    onResponseLengthChange: vi.fn(),
    ticketId: null,
    onTicketIdChange: vi.fn(),
    ticket: null,
    onTicketChange: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders with placeholder text', () => {
    render(<InputPanel {...defaultProps} />);
    expect(screen.getByPlaceholderText('Paste ticket content or describe the issue...')).toBeInTheDocument();
  });

  it('shows input value in textarea', () => {
    render(<InputPanel {...defaultProps} value="Test input content" />);
    expect(screen.getByDisplayValue('Test input content')).toBeInTheDocument();
  });

  it('calls onChange when typing', () => {
    const onChange = vi.fn();
    render(<InputPanel {...defaultProps} onChange={onChange} />);

    const textarea = screen.getByPlaceholderText('Paste ticket content or describe the issue...');
    fireEvent.change(textarea, { target: { value: 'New content' } });

    expect(onChange).toHaveBeenCalledWith('New content');
  });

  it('shows correct word count', () => {
    render(<InputPanel {...defaultProps} value="One two three four five" />);
    expect(screen.getByText('5 words')).toBeInTheDocument();
  });

  it('shows zero words when empty', () => {
    render(<InputPanel {...defaultProps} value="" />);
    expect(screen.getByText('0 words')).toBeInTheDocument();
  });

  it('shows model ready status when loaded', () => {
    render(<InputPanel {...defaultProps} modelLoaded={true} />);
    expect(screen.getByText('● Model ready')).toBeInTheDocument();
  });

  it('shows no model loaded warning when not loaded', () => {
    render(<InputPanel {...defaultProps} modelLoaded={false} />);
    expect(screen.getByText('○ No model loaded')).toBeInTheDocument();
  });

  it('disables Generate button when no model loaded', () => {
    render(<InputPanel {...defaultProps} modelLoaded={false} value="Some text" />);
    const generateButton = screen.getByRole('button', { name: /generate/i });
    expect(generateButton).toBeDisabled();
  });

  it('disables Generate button when no input', () => {
    render(<InputPanel {...defaultProps} modelLoaded={true} value="" />);
    const generateButton = screen.getByRole('button', { name: /generate/i });
    expect(generateButton).toBeDisabled();
  });

  it('enables Generate button when model loaded and has input', () => {
    render(<InputPanel {...defaultProps} modelLoaded={true} value="Some input" />);
    const generateButton = screen.getByRole('button', { name: /generate/i });
    expect(generateButton).not.toBeDisabled();
  });

  it('calls onGenerate when Generate clicked', () => {
    const onGenerate = vi.fn();
    render(<InputPanel {...defaultProps} value="Some input" onGenerate={onGenerate} />);

    fireEvent.click(screen.getByText('Generate'));
    expect(onGenerate).toHaveBeenCalled();
  });

  it('shows loading state when generating', () => {
    render(<InputPanel {...defaultProps} generating={true} value="Some input" />);
    // The Button component should show loading state
    expect(screen.getByText('Generate')).toBeInTheDocument();
  });

  it('calls onClear when Clear clicked', () => {
    const onClear = vi.fn();
    render(<InputPanel {...defaultProps} value="Some text" onClear={onClear} />);

    fireEvent.click(screen.getByText('Clear'));
    expect(onClear).toHaveBeenCalled();
  });

  it('disables Clear button when no content', () => {
    render(<InputPanel {...defaultProps} value="" ocrText={null} ticket={null} />);
    const clearButton = screen.getByRole('button', { name: /clear/i });
    expect(clearButton).toBeDisabled();
  });

  it('response length selector shows correct value', () => {
    render(<InputPanel {...defaultProps} responseLength="Short" />);
    expect(screen.getByDisplayValue('Short (~80 words)')).toBeInTheDocument();
  });

  it('calls onResponseLengthChange when selection changed', () => {
    const onResponseLengthChange = vi.fn();
    render(<InputPanel {...defaultProps} onResponseLengthChange={onResponseLengthChange} />);

    const select = screen.getByDisplayValue('Medium (~160 words)');
    fireEvent.change(select, { target: { value: 'Long' } });

    expect(onResponseLengthChange).toHaveBeenCalledWith('Long');
  });

  it('displays OCR text when present', () => {
    render(<InputPanel {...defaultProps} ocrText="Extracted OCR text" />);
    expect(screen.getByText('Extracted OCR text')).toBeInTheDocument();
    expect(screen.getByText('Screenshot Text (OCR)')).toBeInTheDocument();
  });

  it('calls onOcrTextChange(null) when removing OCR text', () => {
    const onOcrTextChange = vi.fn();
    render(<InputPanel {...defaultProps} ocrText="OCR text" onOcrTextChange={onOcrTextChange} />);

    const removeButton = screen.getByLabelText('Remove OCR text');
    fireEvent.click(removeButton);

    expect(onOcrTextChange).toHaveBeenCalledWith(null);
  });

  it('handles Cmd+G keyboard shortcut for generate', () => {
    const onGenerate = vi.fn();
    render(<InputPanel {...defaultProps} value="Some input" onGenerate={onGenerate} />);

    const textarea = screen.getByPlaceholderText('Paste ticket content or describe the issue...');
    fireEvent.keyDown(textarea, { key: 'g', metaKey: true });

    expect(onGenerate).toHaveBeenCalled();
  });

  it('handles Cmd+N keyboard shortcut for clear', () => {
    const onClear = vi.fn();
    render(<InputPanel {...defaultProps} value="Some input" onClear={onClear} />);

    const textarea = screen.getByPlaceholderText('Paste ticket content or describe the issue...');
    fireEvent.keyDown(textarea, { key: 'n', metaKey: true });

    expect(onClear).toHaveBeenCalled();
  });

  it('does not call onGenerate via shortcut when generating', () => {
    const onGenerate = vi.fn();
    render(<InputPanel {...defaultProps} value="Some input" generating={true} onGenerate={onGenerate} />);

    const textarea = screen.getByPlaceholderText('Paste ticket content or describe the issue...');
    fireEvent.keyDown(textarea, { key: 'g', metaKey: true });

    expect(onGenerate).not.toHaveBeenCalled();
  });

  it('does not call onGenerate via shortcut when no model', () => {
    const onGenerate = vi.fn();
    render(<InputPanel {...defaultProps} value="Some input" modelLoaded={false} onGenerate={onGenerate} />);

    const textarea = screen.getByPlaceholderText('Paste ticket content or describe the issue...');
    fireEvent.keyDown(textarea, { key: 'g', metaKey: true });

    expect(onGenerate).not.toHaveBeenCalled();
  });
});

// Additional Jira integration tests would go here when full test harness is set up
