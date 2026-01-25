import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { SettingsTab } from './SettingsTab';
import { invoke } from '@tauri-apps/api/core';
import { ThemeProvider } from '../../contexts/ThemeContext';
import { ToastProvider } from '../../contexts/ToastContext';

vi.mock('@tauri-apps/api/core');
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);

// Mock all hooks
vi.mock('../../hooks/useLlm', () => ({
  useLlm: () => ({
    loadModel: vi.fn(),
    unloadModel: vi.fn(),
    getLoadedModel: vi.fn().mockResolvedValue(null),
    listModels: vi.fn().mockResolvedValue([]),
    getContextWindow: vi.fn().mockResolvedValue(null),
    setContextWindow: vi.fn(),
  }),
}));

vi.mock('../../hooks/useKb', () => ({
  useKb: () => ({
    setKbFolder: vi.fn(),
    getKbFolder: vi.fn().mockResolvedValue(null),
    rebuildIndex: vi.fn(),
    getIndexStats: vi.fn().mockResolvedValue(null),
    getVectorConsent: vi.fn().mockResolvedValue({ enabled: false }),
    setVectorConsent: vi.fn(),
    generateEmbeddings: vi.fn(),
  }),
}));

vi.mock('../../hooks/useDownload', () => ({
  useDownload: () => ({
    downloadModel: vi.fn(),
    downloadProgress: null,
    isDownloading: false,
  }),
}));

vi.mock('../../hooks/useJira', () => ({
  useJira: () => ({
    checkConfiguration: vi.fn().mockResolvedValue(false),
    configure: vi.fn(),
    disconnect: vi.fn(),
    config: null,
    loading: false,
  }),
}));

vi.mock('../../hooks/useEmbedding', () => ({
  useEmbedding: () => ({
    initEngine: vi.fn(),
    loadModel: vi.fn(),
    unloadModel: vi.fn(),
    checkModelStatus: vi.fn(),
    isModelDownloaded: vi.fn().mockResolvedValue(false),
    getModelPath: vi.fn().mockResolvedValue(null),
    isLoaded: false,
    modelInfo: null,
    loading: false,
  }),
}));

vi.mock('../../hooks/useCustomVariables', () => ({
  useCustomVariables: () => ({
    variables: [],
    loadVariables: vi.fn(),
    saveVariable: vi.fn().mockResolvedValue(true),
    deleteVariable: vi.fn().mockResolvedValue(true),
  }),
}));

function renderWithProviders(ui: React.ReactElement) {
  return render(
    <ThemeProvider>
      <ToastProvider>{ui}</ToastProvider>
    </ThemeProvider>
  );
}

describe('SettingsTab', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Appearance Section', () => {
    it('renders theme selector', () => {
      renderWithProviders(<SettingsTab />);
      expect(screen.getByText('Appearance')).toBeInTheDocument();
      expect(screen.getByText('Light')).toBeInTheDocument();
      expect(screen.getByText('Dark')).toBeInTheDocument();
      expect(screen.getByText('System')).toBeInTheDocument();
    });

    it('allows theme selection', () => {
      renderWithProviders(<SettingsTab />);
      const darkRadio = screen.getByLabelText('Dark');
      fireEvent.click(darkRadio);
      expect(darkRadio).toBeChecked();
    });
  });

  describe('Language Model Section', () => {
    it('renders model list', () => {
      renderWithProviders(<SettingsTab />);
      expect(screen.getByText('Language Model')).toBeInTheDocument();
      expect(screen.getByText('Llama 3.2 1B Instruct')).toBeInTheDocument();
      expect(screen.getByText('Llama 3.2 3B Instruct')).toBeInTheDocument();
      expect(screen.getByText('Phi-3 Mini 4K')).toBeInTheDocument();
    });

    it('shows Download button for non-downloaded models', () => {
      renderWithProviders(<SettingsTab />);
      const downloadButtons = screen.getAllByText('Download');
      expect(downloadButtons.length).toBe(3); // All 3 models need download
    });
  });

  describe('Context Window Section', () => {
    it('renders context window selector', () => {
      renderWithProviders(<SettingsTab />);
      expect(screen.getByText('Context Window')).toBeInTheDocument();
      expect(screen.getByText('Load a model to configure context window.')).toBeInTheDocument();
    });

    it('context window selector is disabled without loaded model', () => {
      renderWithProviders(<SettingsTab />);
      const select = screen.getByRole('combobox');
      expect(select).toBeDisabled();
    });
  });

  describe('Knowledge Base Section', () => {
    it('renders KB folder selector', () => {
      renderWithProviders(<SettingsTab />);
      expect(screen.getByText('Knowledge Base')).toBeInTheDocument();
      expect(screen.getByText('Select Folder')).toBeInTheDocument();
    });

    it('shows placeholder when no folder selected', () => {
      renderWithProviders(<SettingsTab />);
      expect(screen.getByText('No folder selected')).toBeInTheDocument();
    });
  });

  describe('Advanced Search Section', () => {
    it('renders vector consent toggle', () => {
      renderWithProviders(<SettingsTab />);
      expect(screen.getByText('Advanced Search')).toBeInTheDocument();
      expect(screen.getByText('Enable vector embeddings')).toBeInTheDocument();
    });

    it('toggle is unchecked by default', () => {
      renderWithProviders(<SettingsTab />);
      const toggle = screen.getByRole('checkbox');
      expect(toggle).not.toBeChecked();
    });
  });

  describe('Template Variables Section', () => {
    it('renders variables section', () => {
      renderWithProviders(<SettingsTab />);
      expect(screen.getByText('Template Variables')).toBeInTheDocument();
      expect(screen.getByText('No custom variables defined yet.')).toBeInTheDocument();
    });

    it('shows Add Variable button', () => {
      renderWithProviders(<SettingsTab />);
      expect(screen.getByText('+ Add Variable')).toBeInTheDocument();
    });

    it('opens add variable form when button clicked', async () => {
      renderWithProviders(<SettingsTab />);
      fireEvent.click(screen.getByText('+ Add Variable'));

      await waitFor(() => {
        expect(screen.getByText('Add Variable')).toBeInTheDocument();
        expect(screen.getByPlaceholderText('my_variable')).toBeInTheDocument();
      });
    });

    it('validates variable name format', async () => {
      renderWithProviders(<SettingsTab />);
      fireEvent.click(screen.getByText('+ Add Variable'));

      await waitFor(() => {
        expect(screen.getByText('Add Variable')).toBeInTheDocument();
      });

      // Enter invalid name
      const nameInput = screen.getByPlaceholderText('my_variable');
      fireEvent.change(nameInput, { target: { value: '123invalid' } });

      const valueInput = screen.getByPlaceholderText('The value to substitute...');
      fireEvent.change(valueInput, { target: { value: 'test value' } });

      // Try to save (the Add button)
      const addButton = screen.getByRole('button', { name: 'Add' });
      fireEvent.click(addButton);

      await waitFor(() => {
        expect(screen.getByText(/must start with a letter or underscore/)).toBeInTheDocument();
      });
    });
  });

  describe('Jira Integration Section', () => {
    it('renders Jira form when not connected', () => {
      renderWithProviders(<SettingsTab />);
      expect(screen.getByText('Jira Integration')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('https://your-company.atlassian.net')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('your.email@company.com')).toBeInTheDocument();
      expect(screen.getByPlaceholderText('Your Jira API token')).toBeInTheDocument();
    });

    it('Connect button is disabled without all fields', () => {
      renderWithProviders(<SettingsTab />);
      const connectButton = screen.getByRole('button', { name: /connect/i });
      expect(connectButton).toBeDisabled();
    });
  });

  describe('Data Backup Section', () => {
    it('renders backup section', () => {
      renderWithProviders(<SettingsTab />);
      expect(screen.getByText('Data Backup')).toBeInTheDocument();
      expect(screen.getByText('Export Data')).toBeInTheDocument();
      expect(screen.getByText('Import Data')).toBeInTheDocument();
    });
  });

  describe('About Section', () => {
    it('renders about info', () => {
      renderWithProviders(<SettingsTab />);
      expect(screen.getByText('About')).toBeInTheDocument();
      expect(screen.getByText('Version 0.1.0')).toBeInTheDocument();
      expect(screen.getByText('All processing happens locally on your machine.')).toBeInTheDocument();
    });
  });
});
