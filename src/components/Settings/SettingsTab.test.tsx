// @vitest-environment jsdom
import React from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { SettingsTab } from "./SettingsTab";

const invokeMock = vi.fn();
const showSuccess = vi.fn();
const showError = vi.fn();
const setTheme = vi.fn();
const setVectorConsent = vi.fn().mockResolvedValue(undefined);
const refreshSearchApiEmbeddingStatus = vi.fn().mockResolvedValue({
  installed: false,
  ready: false,
  model_name: "sentence-transformers/all-MiniLM-L6-v2",
  revision: "pinned-revision",
  local_path: null,
  error: null,
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock("../shared/Button", () => ({
  Button: ({
    children,
    onClick,
    disabled,
    type,
  }: {
    children: React.ReactNode;
    onClick?: () => void;
    disabled?: boolean;
    type?: "button" | "submit";
  }) => (
    <button type={type ?? "button"} onClick={onClick} disabled={disabled}>
      {children}
    </button>
  ),
}));

vi.mock("../../hooks/useLlm", () => ({
  useLlm: () => ({
    loadModel: vi.fn(),
    unloadModel: vi.fn(),
    getLoadedModel: vi.fn().mockResolvedValue(null),
    getModelInfo: vi.fn().mockResolvedValue(null),
    listModels: vi.fn().mockResolvedValue([]),
    getContextWindow: vi.fn().mockResolvedValue(null),
    setContextWindow: vi.fn().mockResolvedValue(undefined),
    loadCustomModel: vi.fn(),
    validateGgufFile: vi.fn(),
  }),
}));

vi.mock("../../hooks/useKb", () => ({
  useKb: () => ({
    setKbFolder: vi.fn(),
    getKbFolder: vi.fn().mockResolvedValue(null),
    rebuildIndex: vi.fn(),
    getIndexStats: vi.fn().mockResolvedValue(null),
    getVectorConsent: vi.fn().mockResolvedValue({ enabled: true }),
    setVectorConsent,
    generateEmbeddings: vi.fn(),
  }),
}));

vi.mock("../../hooks/useDownload", () => ({
  useDownload: () => ({
    downloadModel: vi.fn(),
    downloadProgress: null,
    isDownloading: false,
    cancelDownload: vi.fn(),
  }),
}));

vi.mock("../../hooks/useJira", () => ({
  useJira: () => ({
    checkConfiguration: vi.fn().mockResolvedValue(false),
    configure: vi.fn(),
    disconnect: vi.fn(),
    config: null,
    loading: false,
  }),
}));

vi.mock("../../hooks/useEmbedding", () => ({
  useEmbedding: () => ({
    initEngine: vi.fn(),
    loadModel: vi.fn(),
    unloadModel: vi.fn(),
    checkModelStatus: vi.fn().mockResolvedValue(false),
    isModelDownloaded: vi.fn().mockResolvedValue(false),
    getModelPath: vi.fn().mockResolvedValue(null),
    isLoaded: false,
    modelInfo: null,
    loading: false,
  }),
}));

vi.mock("../../hooks/useSearchApiEmbedding", () => ({
  useSearchApiEmbedding: () => ({
    status: {
      installed: false,
      ready: false,
      model_name: "sentence-transformers/all-MiniLM-L6-v2",
      revision: "pinned-revision",
      local_path: null,
      error: null,
    },
    loading: false,
    error: null,
    refreshStatus: refreshSearchApiEmbeddingStatus,
    installModel: vi.fn(),
  }),
}));

vi.mock("../../hooks/useCustomVariables", () => ({
  useCustomVariables: () => ({
    variables: [],
    loadVariables: vi.fn().mockResolvedValue(undefined),
    saveVariable: vi.fn(),
    deleteVariable: vi.fn(),
  }),
}));

vi.mock("../../hooks/useFeatureOps", () => ({
  useFeatureOps: () => ({
    getDeploymentHealthSummary: vi.fn().mockResolvedValue(null),
    runDeploymentPreflight: vi.fn(),
    listIntegrations: vi.fn().mockResolvedValue([]),
    configureIntegration: vi.fn(),
  }),
}));

vi.mock("../../contexts/ThemeContext", () => ({
  useTheme: () => ({
    theme: "system",
    setTheme,
  }),
}));

vi.mock("../../contexts/ToastContext", () => ({
  useToastContext: () => ({
    success: showSuccess,
    error: showError,
  }),
}));

vi.mock("../../features/analytics/qualityThresholds", () => ({
  getResponseQualityThresholds: () => ({
    editRatioWatch: 0.2,
    editRatioAction: 0.4,
    timeToDraftWatchMs: 1000,
    timeToDraftActionMs: 2000,
    copyPerSaveWatch: 3,
    copyPerSaveAction: 2,
    editedSaveRateWatch: 0.2,
    editedSaveRateAction: 0.4,
  }),
  resetResponseQualityThresholds: () => ({
    editRatioWatch: 0.2,
    editRatioAction: 0.4,
    timeToDraftWatchMs: 1000,
    timeToDraftActionMs: 2000,
    copyPerSaveWatch: 3,
    copyPerSaveAction: 2,
    editedSaveRateWatch: 0.2,
    editedSaveRateAction: 0.4,
  }),
  saveResponseQualityThresholds: (value: unknown) => value,
}));

vi.mock("./sections/SettingsOverviewSections", () => ({
  SettingsHero: () => <div>Settings Hero</div>,
  PolicyGatesSection: () => <div>Policy Gates</div>,
  MemoryKernelSection: () => <div>MemoryKernel</div>,
  AppearanceSection: () => <div>Appearance</div>,
  AboutSection: ({ versionLabel }: { versionLabel: string }) => (
    <div>{versionLabel}</div>
  ),
}));

vi.mock("./sections/SettingsOpsSections", () => ({
  DeploymentSection: () => <div>Deployment</div>,
  BackupSection: () => <div>Backup</div>,
  QualityThresholdSection: () => <div>Quality Thresholds</div>,
  AuditLogsSection: () => <div>Audit Logs</div>,
}));

beforeEach(() => {
  invokeMock.mockImplementation((command: string) => {
    if (command === "get_allow_unverified_local_models") {
      return Promise.resolve(false);
    }
    if (command === "get_audit_entries") {
      return Promise.resolve([]);
    }
    if (command === "get_memory_kernel_preflight_status") {
      return Promise.resolve(null);
    }
    return Promise.resolve(null);
  });
});

afterEach(() => {
  cleanup();
  invokeMock.mockReset();
  showSuccess.mockReset();
  showError.mockReset();
  setTheme.mockReset();
  setVectorConsent.mockClear();
  refreshSearchApiEmbeddingStatus.mockClear();
});

describe("SettingsTab", () => {
  it("renders separate semantic-search model cards and the advanced local-model toggle", async () => {
    render(<SettingsTab />);

    await waitFor(() => {
      expect(
        screen.getByRole("heading", { name: "Semantic Search Models" }),
      ).toBeTruthy();
    });

    expect(
      screen.getByRole("heading", { name: "Desktop Embedding Model" }),
    ).toBeTruthy();
    expect(
      screen.getByRole("heading", { name: "Search API Embedding Model" }),
    ).toBeTruthy();
    expect(
      screen.getByLabelText("Allow unverified local models (advanced)"),
    ).toBeTruthy();
    expect(
      screen.getByText(
        /install this managed model to keep search-api embeddings explicit/i,
      ),
    ).toBeTruthy();
  });

  it("persists the advanced unverified-model toggle when changed", async () => {
    render(<SettingsTab />);

    const toggle = await screen.findByLabelText(
      "Allow unverified local models (advanced)",
    );
    fireEvent.click(toggle);

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "set_allow_unverified_local_models",
        { enabled: true },
      );
    });
  });
});
