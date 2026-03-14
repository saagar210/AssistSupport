import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "../shared/Button";
import { useLlm } from "../../hooks/useLlm";
import { useKb } from "../../hooks/useKb";
import { useDownload } from "../../hooks/useDownload";
import { useJira } from "../../hooks/useJira";
import { useEmbedding } from "../../hooks/useEmbedding";
import { useSearchApiEmbedding } from "../../hooks/useSearchApiEmbedding";
import { useCustomVariables } from "../../hooks/useCustomVariables";
import { useFeatureOps } from "../../hooks/useFeatureOps";
import {
  getResponseQualityThresholds,
  resetResponseQualityThresholds,
  ResponseQualityThresholds,
  saveResponseQualityThresholds,
} from "../../features/analytics/qualityThresholds";
import { useTheme } from "../../contexts/ThemeContext";
import { useToastContext } from "../../contexts/ToastContext";
import { resolveRevampFlags } from "../../features/revamp/flags";
import {
  AboutSection,
  AppearanceSection,
  MemoryKernelSection,
  PolicyGatesSection,
  SettingsHero,
} from "./sections/SettingsOverviewSections";
import {
  AuditLogsSection,
  BackupSection,
  DeploymentSection,
  QualityThresholdSection,
} from "./sections/SettingsOpsSections";
import appPackage from "../../../package.json";
import { formatAppVersion } from "./versionLabel";
import type {
  AuditEntry,
  CustomVariable,
  DeploymentHealthSummary,
  IntegrationConfigRecord,
  MemoryKernelPreflightStatus,
  ModelInfo,
  SearchApiEmbeddingModelStatus,
} from "../../types";
import "./SettingsTab.css";

const RECOMMENDED_MODELS: ModelInfo[] = [
  {
    id: "llama-3.1-8b-instruct",
    name: "Llama 3.1 8B Instruct",
    size: "4.9 GB",
    description: "Recommended: higher quality and more reliable grounding",
  },
];

// Still supported, but intentionally hidden behind progressive disclosure to keep
// operators focused on a single default model path.
const OTHER_SUPPORTED_MODELS: ModelInfo[] = [
  {
    id: "llama-3.2-1b-instruct",
    name: "Llama 3.2 1B Instruct",
    size: "1.3 GB",
    description: "Fast, lightweight model for quick responses",
  },
  {
    id: "llama-3.2-3b-instruct",
    name: "Llama 3.2 3B Instruct",
    size: "2.0 GB",
    description: "Balanced performance and quality",
  },
  {
    id: "phi-3-mini-4k-instruct",
    name: "Phi-3 Mini 4K",
    size: "2.4 GB",
    description: "Microsoft model, good for reasoning",
  },
];

const APP_VERSION = appPackage.version;

// Audit event types can be either a plain string (unit variants like "key_generated")
// or an object (data variants like { custom: "value" }). Normalize for display.
function formatAuditEvent(event: string | Record<string, string>): string {
  if (typeof event === "string") return event;
  if (typeof event === "object" && event !== null) {
    const key = Object.keys(event)[0];
    return key ? `${key}: ${event[key]}` : JSON.stringify(event);
  }
  return String(event);
}

const CONTEXT_WINDOW_OPTIONS = [
  { value: null, label: "Model Default" },
  { value: 2048, label: "2K (2,048 tokens)" },
  { value: 4096, label: "4K (4,096 tokens)" },
  { value: 8192, label: "8K (8,192 tokens)" },
  { value: 16384, label: "16K (16,384 tokens)" },
  { value: 32768, label: "32K (32,768 tokens)" },
];

const AUDIT_PAGE_SIZE = 50;

// Helper to format bytes for display
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

// Helper to format download speed
function formatSpeed(bps: number): string {
  if (bps === 0) return "";
  return `${formatBytes(bps)}/s`;
}

function formatVerificationStatus(status: string | null | undefined): string {
  if (status === "verified") return "Verified";
  if (status === "unverified") return "Unverified";
  return "Unknown";
}

function getSearchApiEmbeddingBadge(
  status: SearchApiEmbeddingModelStatus | null,
  installError: string | null,
): { label: string; className: string; detail: string } {
  if (installError) {
    return {
      label: "Unavailable",
      className: "error",
      detail: installError,
    };
  }

  if (!status) {
    return {
      label: "Checking",
      className: "downloaded",
      detail:
        "Checking whether the managed search API embedding model is installed.",
    };
  }

  if (!status.installed) {
    return {
      label: "Not Installed",
      className: "not-downloaded",
      detail:
        "Install this managed model to keep search-api embeddings explicit, pinned, and offline at runtime.",
    };
  }

  if (!status.ready) {
    return {
      label: "Needs Repair",
      className: "error",
      detail:
        status.error ??
        "The managed search API embedding model is installed but not ready.",
    };
  }

  return {
    label: "Ready",
    className: "loaded",
    detail: `Pinned revision ${status.revision}. Loaded from local disk only at runtime.`,
  };
}

function validateQualityThresholds(
  thresholds: ResponseQualityThresholds,
): string | null {
  if (thresholds.editRatioWatch >= thresholds.editRatioAction) {
    return "Edit ratio watch threshold must be lower than action threshold.";
  }
  if (thresholds.timeToDraftWatchMs >= thresholds.timeToDraftActionMs) {
    return "Time-to-draft watch threshold must be lower than action threshold.";
  }
  if (thresholds.copyPerSaveWatch <= thresholds.copyPerSaveAction) {
    return "Copy-per-save watch threshold must be higher than action threshold.";
  }
  if (thresholds.editedSaveRateWatch >= thresholds.editedSaveRateAction) {
    return "Edited save rate watch threshold must be lower than action threshold.";
  }
  return null;
}

export function SettingsTab() {
  const {
    loadModel,
    unloadModel,
    getLoadedModel,
    getModelInfo,
    listModels,
    getContextWindow,
    setContextWindow,
    loadCustomModel,
    validateGgufFile,
  } = useLlm();
  const {
    setKbFolder,
    getKbFolder,
    rebuildIndex,
    getIndexStats,
    getVectorConsent,
    setVectorConsent,
    generateEmbeddings,
  } = useKb();
  const { downloadModel, downloadProgress, isDownloading, cancelDownload } =
    useDownload();
  const {
    checkConfiguration: checkJiraConfig,
    configure: configureJira,
    disconnect: disconnectJira,
    config: jiraConfig,
    loading: jiraLoading,
  } = useJira();
  const {
    initEngine: initEmbeddingEngine,
    loadModel: loadEmbeddingModel,
    unloadModel: unloadEmbeddingModel,
    checkModelStatus: checkEmbeddingStatus,
    isModelDownloaded: isEmbeddingDownloaded,
    getModelPath: getEmbeddingModelPath,
    isLoaded: isEmbeddingLoaded,
    modelInfo: embeddingModelInfo,
    loading: embeddingLoading,
  } = useEmbedding();
  const {
    status: searchApiEmbeddingStatus,
    loading: searchApiEmbeddingLoading,
    error: searchApiEmbeddingError,
    refreshStatus: refreshSearchApiEmbeddingStatus,
    installModel: installSearchApiEmbeddingModel,
  } = useSearchApiEmbedding();
  const {
    getDeploymentHealthSummary,
    runDeploymentPreflight,
    listIntegrations,
    configureIntegration,
  } = useFeatureOps();
  const { theme, setTheme } = useTheme();
  const { success: showSuccess, error: showError } = useToastContext();
  const {
    variables: customVariables,
    loadVariables,
    saveVariable,
    deleteVariable,
  } = useCustomVariables();

  const [loadedModel, setLoadedModel] = useState<string | null>(null);
  const [loadedModelInfo, setLoadedModelInfo] = useState<ModelInfo | null>(
    null,
  );
  const [downloadedModels, setDownloadedModels] = useState<string[]>([]);
  const [showOtherModels, setShowOtherModels] = useState(false);
  const [kbFolder, setKbFolderState] = useState<string | null>(null);
  const [indexStats, setIndexStats] = useState<{
    total_chunks: number;
    total_files: number;
  } | null>(null);
  const [vectorEnabled, setVectorEnabled] = useState(false);
  const [jiraConfigured, setJiraConfigured] = useState(false);
  const [jiraForm, setJiraForm] = useState({
    baseUrl: "",
    email: "",
    apiToken: "",
  });
  const [contextWindowSize, setContextWindowSize] = useState<number | null>(
    null,
  );
  const [embeddingDownloaded, setEmbeddingDownloaded] = useState(false);
  const [allowUnverifiedLocalModels, setAllowUnverifiedLocalModels] =
    useState(false);
  const [generatingEmbeddings, setGeneratingEmbeddings] = useState(false);
  const [loading, setLoading] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [backupLoading, setBackupLoading] = useState<
    "export" | "import" | null
  >(null);
  const [auditEntries, setAuditEntries] = useState<AuditEntry[]>([]);
  const [auditLoading, setAuditLoading] = useState(false);
  const [auditExporting, setAuditExporting] = useState(false);
  const [auditSeverityFilter, setAuditSeverityFilter] = useState<
    "all" | "info" | "warning" | "error" | "critical"
  >("all");
  const [auditSearchQuery, setAuditSearchQuery] = useState("");
  const [auditPage, setAuditPage] = useState(1);

  // Deployment and integration state
  const [deploymentHealth, setDeploymentHealth] =
    useState<DeploymentHealthSummary | null>(null);
  const [deployPreflightChecks, setDeployPreflightChecks] = useState<string[]>(
    [],
  );
  const [deployPreflightRunning, setDeployPreflightRunning] = useState(false);
  const [integrations, setIntegrations] = useState<IntegrationConfigRecord[]>(
    [],
  );
  const [qualityThresholds, setQualityThresholds] =
    useState<ResponseQualityThresholds>(() => getResponseQualityThresholds());
  const [qualityThresholdError, setQualityThresholdError] = useState<
    string | null
  >(null);
  const [memoryKernelPreflight, setMemoryKernelPreflight] =
    useState<MemoryKernelPreflightStatus | null>(null);
  const [memoryKernelLoading, setMemoryKernelLoading] = useState(false);
  const revampFlags = useMemo(() => resolveRevampFlags(), []);

  // Custom variables state
  const [editingVariable, setEditingVariable] = useState<CustomVariable | null>(
    null,
  );
  const [variableForm, setVariableForm] = useState({ name: "", value: "" });
  const [showVariableForm, setShowVariableForm] = useState(false);
  const [variableFormError, setVariableFormError] = useState<string | null>(
    null,
  );

  const loadAuditEntries = useCallback(async () => {
    setAuditLoading(true);
    try {
      const entries = await invoke<AuditEntry[]>("get_audit_entries", {
        limit: 200,
      });
      setAuditEntries(entries ?? []);
      setAuditPage(1);
    } catch (err) {
      showError(`Failed to load audit logs: ${err}`);
    } finally {
      setAuditLoading(false);
    }
  }, [showError]);

  const refreshMemoryKernelStatus = useCallback(async () => {
    setMemoryKernelLoading(true);
    try {
      const status = await invoke<MemoryKernelPreflightStatus>(
        "get_memory_kernel_preflight_status",
      );
      setMemoryKernelPreflight(status);
    } catch (err) {
      // Non-blocking: show as unavailable rather than failing settings load.
      setMemoryKernelPreflight(null);
    } finally {
      setMemoryKernelLoading(false);
    }
  }, []);

  const filteredAuditEntries = useMemo(() => {
    const normalized = auditEntries.slice().reverse();
    const query = auditSearchQuery.trim().toLowerCase();
    return normalized.filter((entry) => {
      if (
        auditSeverityFilter !== "all" &&
        entry.severity !== auditSeverityFilter
      ) {
        return false;
      }
      if (!query) {
        return true;
      }
      const eventText = formatAuditEvent(entry.event).toLowerCase();
      return (
        eventText.includes(query) || entry.message.toLowerCase().includes(query)
      );
    });
  }, [auditEntries, auditSearchQuery, auditSeverityFilter]);

  const auditTotalPages = useMemo(
    () => Math.max(1, Math.ceil(filteredAuditEntries.length / AUDIT_PAGE_SIZE)),
    [filteredAuditEntries.length],
  );

  useEffect(() => {
    setAuditPage((prev) => Math.min(prev, auditTotalPages));
  }, [auditTotalPages]);

  useEffect(() => {
    refreshMemoryKernelStatus();
  }, [refreshMemoryKernelStatus]);

  const pagedAuditEntries = useMemo(() => {
    const start = (auditPage - 1) * AUDIT_PAGE_SIZE;
    return filteredAuditEntries.slice(start, start + AUDIT_PAGE_SIZE);
  }, [filteredAuditEntries, auditPage]);

  useEffect(() => {
    Promise.resolve(loadInitialState()).catch((err) =>
      console.error("Settings init failed:", err),
    );
    Promise.resolve(loadVariables()).catch((err) =>
      console.error("Variables load failed:", err),
    );
    Promise.resolve(loadAuditEntries()).catch((err) =>
      console.error("Audit load failed:", err),
    );
  }, [loadVariables, loadAuditEntries]);

  async function loadInitialState() {
    try {
      const [
        loaded,
        modelInfo,
        downloaded,
        folder,
        stats,
        consent,
        jiraConfigResult,
        ctxWindow,
        embDownloaded,
        allowUnverifiedModels,
        deployHealth,
        integrationsList,
      ] = await Promise.all([
        getLoadedModel(),
        getModelInfo().catch(() => null),
        listModels(),
        getKbFolder(),
        getIndexStats().catch(() => null),
        getVectorConsent().catch(() => null),
        checkJiraConfig().catch(() => false),
        getContextWindow().catch(() => null),
        isEmbeddingDownloaded().catch(() => false),
        invoke<boolean>("get_allow_unverified_local_models").catch(() => false),
        getDeploymentHealthSummary().catch(() => null),
        listIntegrations().catch(() => []),
      ]);
      setLoadedModel(loaded);
      setLoadedModelInfo(modelInfo);
      setDownloadedModels(downloaded);
      setKbFolderState(folder);
      setIndexStats(stats);
      if (consent) {
        setVectorEnabled(consent.enabled);
      }
      setJiraConfigured(jiraConfigResult);
      setContextWindowSize(ctxWindow);
      setEmbeddingDownloaded(embDownloaded);
      setAllowUnverifiedLocalModels(allowUnverifiedModels);
      setDeploymentHealth(deployHealth);
      setIntegrations(integrationsList ?? []);
      setQualityThresholds(getResponseQualityThresholds());

      // Check embedding model status
      await Promise.all([
        checkEmbeddingStatus(),
        refreshSearchApiEmbeddingStatus(),
      ]);
    } catch (err) {
      console.error("Failed to load settings state:", err);
    }
  }

  async function handleVectorToggle() {
    const newValue = !vectorEnabled;
    try {
      await setVectorConsent(newValue);
      setVectorEnabled(newValue);
    } catch (err) {
      setError(`Failed to update vector consent: ${err}`);
    }
  }

  async function handleJiraConnect(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      await configureJira(jiraForm.baseUrl, jiraForm.email, jiraForm.apiToken);
      setJiraConfigured(true);
      setJiraForm({ baseUrl: "", email: "", apiToken: "" });
    } catch (err) {
      setError(`Failed to connect to Jira: ${err}`);
    }
  }

  async function handleJiraDisconnect() {
    setError(null);
    try {
      await disconnectJira();
      setJiraConfigured(false);
    } catch (err) {
      setError(`Failed to disconnect Jira: ${err}`);
    }
  }

  async function handleLoadModel(modelId: string) {
    setLoading(modelId);
    setError(null);
    try {
      const info = await loadModel(modelId);
      setLoadedModel(modelId);
      setLoadedModelInfo(info);
    } catch (err) {
      setError(`Failed to load model: ${err}`);
    } finally {
      setLoading(null);
    }
  }

  async function handleUnloadModel() {
    setLoading("unload");
    setError(null);
    try {
      await unloadModel();
      setLoadedModel(null);
      setLoadedModelInfo(null);
    } catch (err) {
      setError(`Failed to unload model: ${err}`);
    } finally {
      setLoading(null);
    }
  }

  async function handleDownloadModel(modelId: string) {
    setError(null);
    try {
      await downloadModel(modelId);
      setDownloadedModels((prev) => [...prev, modelId]);
    } catch (err) {
      setError(`Failed to download model: ${err}`);
    }
  }

  async function handleLoadCustomModel() {
    setError(null);
    setLoading("custom");
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "GGUF Model",
            extensions: ["gguf"],
          },
        ],
        title: "Select GGUF Model File",
      });

      if (selected && typeof selected === "string") {
        // Validate the file first
        const validation = await validateGgufFile(selected);
        if (!validation.is_valid) {
          setError(
            `Invalid GGUF file: ${validation.file_name}. Please select a valid GGUF model file.`,
          );
          return;
        }

        if (
          validation.integrity_status === "unverified" &&
          !allowUnverifiedLocalModels
        ) {
          setError(
            "This GGUF file is not on the verified allowlist. Enable the advanced override below if you need to load an unverified local model.",
          );
          return;
        }

        if (
          validation.integrity_status === "unverified" &&
          allowUnverifiedLocalModels &&
          !window.confirm(
            "This GGUF file is not verified. Load it anyway? Only continue if you trust the file source.",
          )
        ) {
          return;
        }

        // Load the model
        const info = await loadCustomModel(selected);
        setLoadedModel(validation.file_name);
        setLoadedModelInfo(info);
        showSuccess(
          validation.integrity_status === "verified"
            ? `Loaded verified custom model: ${validation.file_name}`
            : `Loaded unverified custom model: ${validation.file_name}`,
        );
      }
    } catch (err) {
      setError(`Failed to load custom model: ${err}`);
    } finally {
      setLoading(null);
    }
  }

  async function handleSelectKbFolder() {
    setError(null);
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Knowledge Base Folder",
      });
      if (selected && typeof selected === "string") {
        await setKbFolder(selected);
        setKbFolderState(selected);
        const stats = await getIndexStats().catch(() => null);
        setIndexStats(stats);
      }
    } catch (err) {
      setError(`Failed to set KB folder: ${err}`);
    }
  }

  async function handleRebuildIndex() {
    if (!kbFolder) return;
    setLoading("rebuild");
    setError(null);
    try {
      await rebuildIndex();
      const stats = await getIndexStats();
      setIndexStats(stats);
    } catch (err) {
      setError(`Failed to rebuild index: ${err}`);
    } finally {
      setLoading(null);
    }
  }

  async function handleContextWindowChange(value: string) {
    const newSize = value === "" ? null : parseInt(value, 10);
    setError(null);
    try {
      await setContextWindow(newSize);
      setContextWindowSize(newSize);
      showSuccess("Context window updated");
    } catch (err) {
      setError(`Failed to update context window: ${err}`);
    }
  }

  async function handleDownloadEmbeddingModel() {
    setError(null);
    try {
      await downloadModel("nomic-embed-text");
      setEmbeddingDownloaded(true);
      showSuccess("Embedding model downloaded");
    } catch (err) {
      setError(`Failed to download embedding model: ${err}`);
    }
  }

  async function handleLoadEmbeddingModel() {
    setError(null);
    try {
      // Engine is initialized at startup; this is idempotent
      await initEmbeddingEngine();
      // Get model path
      const path = await getEmbeddingModelPath("nomic-embed-text");
      if (!path) {
        showError("Embedding model file not found. Try re-downloading.");
        return;
      }
      await loadEmbeddingModel(path);
      showSuccess("Embedding model loaded");
    } catch (err) {
      const msg = `Failed to load embedding model: ${err}`;
      showError(msg);
      setError(msg);
    }
  }

  async function handleUnloadEmbeddingModel() {
    setError(null);
    try {
      await unloadEmbeddingModel();
      showSuccess("Embedding model unloaded");
    } catch (err) {
      setError(`Failed to unload embedding model: ${err}`);
    }
  }

  async function handleSetAllowUnverifiedLocalModels(enabled: boolean) {
    setError(null);
    try {
      await invoke("set_allow_unverified_local_models", { enabled });
      setAllowUnverifiedLocalModels(enabled);
      showSuccess(
        enabled
          ? "Advanced override enabled. Unverified local GGUF models now require confirmation before loading."
          : "Advanced override disabled. Only verified local GGUF models can be loaded.",
      );
    } catch (err) {
      setError(`Failed to update advanced model setting: ${err}`);
    }
  }

  async function handleInstallSearchApiEmbeddingModel() {
    setError(null);
    try {
      const status = await installSearchApiEmbeddingModel();
      if (status.ready) {
        showSuccess("Search API embedding model installed");
      } else {
        showError(
          status.error ??
            "Search API embedding model install completed, but it is not ready yet.",
        );
      }
    } catch (err) {
      setError(`Failed to install search API embedding model: ${err}`);
    }
  }

  async function handleGenerateEmbeddings() {
    if (!vectorEnabled || !isEmbeddingLoaded) {
      setError("Vector search and embedding model must be enabled");
      return;
    }
    setGeneratingEmbeddings(true);
    setError(null);
    try {
      const result = await generateEmbeddings();
      showSuccess(`Generated embeddings for ${result.chunks_processed} chunks`);
    } catch (err) {
      showError(`Failed to generate embeddings: ${err}`);
    } finally {
      setGeneratingEmbeddings(false);
    }
  }

  // Custom variable handlers
  const handleEditVariable = useCallback((variable: CustomVariable) => {
    setEditingVariable(variable);
    setVariableForm({ name: variable.name, value: variable.value });
    setShowVariableForm(true);
    setVariableFormError(null);
  }, []);

  const handleAddVariable = useCallback(() => {
    setEditingVariable(null);
    setVariableForm({ name: "", value: "" });
    setShowVariableForm(true);
    setVariableFormError(null);
  }, []);

  const handleCancelVariableForm = useCallback(() => {
    setShowVariableForm(false);
    setEditingVariable(null);
    setVariableForm({ name: "", value: "" });
    setVariableFormError(null);
  }, []);

  const handleSaveVariable = useCallback(async () => {
    const name = variableForm.name.trim();
    const value = variableForm.value.trim();

    // Validate name format (alphanumeric and underscores only)
    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(name)) {
      setVariableFormError(
        "Name must start with a letter or underscore and contain only letters, numbers, and underscores",
      );
      return;
    }

    if (!value) {
      setVariableFormError("Value is required");
      return;
    }

    // Check for duplicate name (except when editing the same variable)
    const isDuplicate = customVariables.some(
      (v) =>
        v.name.toLowerCase() === name.toLowerCase() &&
        v.id !== editingVariable?.id,
    );
    if (isDuplicate) {
      setVariableFormError("A variable with this name already exists");
      return;
    }

    const success = await saveVariable(name, value, editingVariable?.id);
    if (success) {
      showSuccess(editingVariable ? "Variable updated" : "Variable created");
      handleCancelVariableForm();
    } else {
      setVariableFormError("Failed to save variable");
    }
  }, [
    variableForm,
    editingVariable,
    customVariables,
    saveVariable,
    showSuccess,
    handleCancelVariableForm,
  ]);

  const handleDeleteVariable = useCallback(
    async (variableId: string) => {
      const success = await deleteVariable(variableId);
      if (success) {
        showSuccess("Variable deleted");
      } else {
        showError("Failed to delete variable");
      }
    },
    [deleteVariable, showSuccess, showError],
  );

  // Backup handlers
  const handleExportBackup = useCallback(async () => {
    setBackupLoading("export");
    setError(null);
    try {
      const result = await invoke<{
        drafts_count: number;
        templates_count: number;
        variables_count: number;
        trees_count: number;
        path: string;
      }>("export_backup");
      showSuccess(
        `Exported ${result.drafts_count} drafts, ${result.templates_count} templates, ${result.variables_count} variables, ${result.trees_count} trees`,
      );
    } catch (err) {
      if (String(err) !== "Export cancelled") {
        showError(`Export failed: ${err}`);
      }
    } finally {
      setBackupLoading(null);
    }
  }, [showSuccess, showError]);

  const handleImportBackup = useCallback(async () => {
    setBackupLoading("import");
    setError(null);
    try {
      const result = await invoke<{
        drafts_imported: number;
        templates_imported: number;
        variables_imported: number;
        trees_imported: number;
      }>("import_backup");
      showSuccess(
        `Imported ${result.drafts_imported} drafts, ${result.templates_imported} templates, ${result.variables_imported} variables, ${result.trees_imported} trees`,
      );
      // Reload data
      loadInitialState();
      loadVariables();
    } catch (err) {
      if (String(err) !== "Import cancelled") {
        showError(`Import failed: ${err}`);
      }
    } finally {
      setBackupLoading(null);
    }
  }, [showSuccess, showError, loadVariables]);

  const handleExportAuditLog = useCallback(async () => {
    setAuditExporting(true);
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const path = await save({
        title: "Export Audit Log",
        defaultPath: "assist-support-audit.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!path) {
        setAuditExporting(false);
        return;
      }
      const output = await invoke<string>("export_audit_log", {
        exportPath: path,
      });
      showSuccess(`Audit log exported to ${output}`);
    } catch (err) {
      if (String(err) !== "Export cancelled") {
        showError(`Audit export failed: ${err}`);
      }
    } finally {
      setAuditExporting(false);
    }
  }, [showSuccess, showError]);

  const handleRunDeploymentPreflight = useCallback(async () => {
    setDeployPreflightRunning(true);
    try {
      const result = await runDeploymentPreflight("stable");
      setDeployPreflightChecks(result.checks);
      const latest = await getDeploymentHealthSummary().catch(() => null);
      setDeploymentHealth(latest);
      if (result.ok) {
        showSuccess("Deployment preflight passed");
      } else {
        showError("Deployment preflight reported failures");
      }
    } catch (err) {
      showError(`Deployment preflight failed: ${err}`);
    } finally {
      setDeployPreflightRunning(false);
    }
  }, [
    runDeploymentPreflight,
    getDeploymentHealthSummary,
    showSuccess,
    showError,
  ]);

  const handleToggleIntegration = useCallback(
    async (integrationType: string, enabled: boolean) => {
      try {
        await configureIntegration(integrationType, enabled);
        const updated = await listIntegrations();
        setIntegrations(updated ?? []);
      } catch (err) {
        showError(`Failed to update ${integrationType}: ${err}`);
      }
    },
    [configureIntegration, listIntegrations, showError],
  );

  const updateQualityThreshold = useCallback(
    <K extends keyof ResponseQualityThresholds>(key: K, value: number) => {
      setQualityThresholds((prev) => ({ ...prev, [key]: value }));
      setQualityThresholdError(null);
    },
    [],
  );

  const handleSaveQualityThresholds = useCallback(() => {
    const validationError = validateQualityThresholds(qualityThresholds);
    if (validationError) {
      setQualityThresholdError(validationError);
      return;
    }
    const saved = saveResponseQualityThresholds(qualityThresholds);
    setQualityThresholds(saved);
    setQualityThresholdError(null);
    showSuccess("Response quality coaching thresholds updated");
  }, [qualityThresholds, showSuccess]);

  const handleResetQualityThresholds = useCallback(() => {
    const defaults = resetResponseQualityThresholds();
    setQualityThresholds(defaults);
    setQualityThresholdError(null);
    showSuccess("Response quality coaching thresholds reset to defaults");
  }, [showSuccess]);

  const searchApiEmbeddingBadge = getSearchApiEmbeddingBadge(
    searchApiEmbeddingStatus,
    searchApiEmbeddingError,
  );

  return (
    <div className="settings-tab">
      {error && <div className="settings-error">{error}</div>}

      <SettingsHero
        loadedModel={loadedModel}
        kbFolder={kbFolder}
        isEmbeddingLoaded={isEmbeddingLoaded}
        embeddingDownloaded={embeddingDownloaded}
        memoryKernelPreflight={memoryKernelPreflight}
      />

      <PolicyGatesSection
        adminTabsEnabled={revampFlags.ASSISTSUPPORT_ENABLE_ADMIN_TABS}
        networkIngestEnabled={revampFlags.ASSISTSUPPORT_ENABLE_NETWORK_INGEST}
      />

      <MemoryKernelSection
        memoryKernelPreflight={memoryKernelPreflight}
        memoryKernelLoading={memoryKernelLoading}
        onRefresh={() => {
          void refreshMemoryKernelStatus();
        }}
      />

      <AppearanceSection theme={theme} onThemeChange={setTheme} />

      <section className="settings-section">
        <h2>Language Model</h2>
        <p className="settings-description">
          Select and load a language model for generating responses.
        </p>

        {loadedModel && (
          <div className="loaded-model-banner">
            <span>
              Currently loaded: <strong>{loadedModel}</strong>
              {loadedModelInfo?.verification_status && (
                <strong
                  className={`verification-badge ${loadedModelInfo.verification_status}`}
                >
                  {formatVerificationStatus(
                    loadedModelInfo.verification_status,
                  )}
                </strong>
              )}
            </span>
            <Button
              variant="secondary"
              size="small"
              onClick={handleUnloadModel}
              disabled={loading === "unload"}
            >
              {loading === "unload" ? "Unloading..." : "Unload"}
            </Button>
          </div>
        )}

        <div className="settings-subsection">
          <h3>Recommended</h3>
          <p className="setting-note">
            For consistent results across operators, AssistSupport recommends a
            single default model.
          </p>
        </div>
        <div className="model-list">
          {RECOMMENDED_MODELS.map((model) => {
            const isDownloaded = downloadedModels.includes(model.id);
            const isLoaded = loadedModel === model.id;
            const isLoadingThis = loading === model.id;
            const isDownloadingThis =
              isDownloading && downloadProgress?.model_id === model.id;

            return (
              <div
                key={model.id}
                className={`model-card ${isLoaded ? "loaded" : ""}`}
              >
                <div className="model-info">
                  <h3>{model.name}</h3>
                  <p>{model.description}</p>
                  <span className="model-size">{model.size}</span>
                </div>
                <div className="model-actions">
                  {isDownloadingThis ? (
                    <div className="download-progress-container">
                      <div className="download-progress">
                        <div
                          className="download-bar"
                          style={{
                            width: `${downloadProgress?.percent || 0}%`,
                          }}
                        />
                        <span className="download-percent">
                          {Math.round(downloadProgress?.percent || 0)}%
                        </span>
                      </div>
                      <div className="download-info">
                        <span className="download-size">
                          {formatBytes(downloadProgress?.downloaded_bytes || 0)}
                          {downloadProgress?.total_bytes
                            ? ` / ${formatBytes(downloadProgress.total_bytes)}`
                            : ""}
                        </span>
                        <span className="download-speed">
                          {formatSpeed(downloadProgress?.speed_bps || 0)}
                        </span>
                      </div>
                      <Button
                        variant="ghost"
                        size="small"
                        onClick={cancelDownload}
                        className="download-cancel-btn"
                      >
                        Cancel
                      </Button>
                    </div>
                  ) : isDownloaded ? (
                    <Button
                      variant={isLoaded ? "secondary" : "primary"}
                      size="small"
                      onClick={() =>
                        isLoaded
                          ? handleUnloadModel()
                          : handleLoadModel(model.id)
                      }
                      disabled={!!loading}
                    >
                      {isLoadingThis
                        ? "Loading..."
                        : isLoaded
                          ? "Unload"
                          : "Load"}
                    </Button>
                  ) : (
                    <Button
                      variant="secondary"
                      size="small"
                      onClick={() => handleDownloadModel(model.id)}
                      disabled={isDownloading}
                    >
                      Download
                    </Button>
                  )}
                </div>
              </div>
            );
          })}
        </div>

        <div className="settings-subsection">
          <Button
            variant="ghost"
            size="small"
            onClick={() => setShowOtherModels((v) => !v)}
            className="btn-hover-scale"
          >
            {showOtherModels
              ? "Hide other supported models"
              : "Show other supported models"}
          </Button>
          {showOtherModels && (
            <>
              <p className="setting-note">
                These models are supported for experimentation, but may be less
                reliable for production ticket responses.
              </p>
              <div className="model-list">
                {OTHER_SUPPORTED_MODELS.map((model) => {
                  const isDownloaded = downloadedModels.includes(model.id);
                  const isLoaded = loadedModel === model.id;
                  const isLoadingThis = loading === model.id;
                  const isDownloadingThis =
                    isDownloading && downloadProgress?.model_id === model.id;

                  return (
                    <div
                      key={model.id}
                      className={`model-card ${isLoaded ? "loaded" : ""}`}
                    >
                      <div className="model-info">
                        <h3>{model.name}</h3>
                        <p>{model.description}</p>
                        <span className="model-size">{model.size}</span>
                      </div>
                      <div className="model-actions">
                        {isDownloadingThis ? (
                          <div className="download-progress-container">
                            <div className="download-progress">
                              <div
                                className="download-bar"
                                style={{
                                  width: `${downloadProgress?.percent || 0}%`,
                                }}
                              />
                              <span className="download-percent">
                                {Math.round(downloadProgress?.percent || 0)}%
                              </span>
                            </div>
                            <div className="download-info">
                              <span className="download-size">
                                {formatBytes(
                                  downloadProgress?.downloaded_bytes || 0,
                                )}
                                {downloadProgress?.total_bytes
                                  ? ` / ${formatBytes(downloadProgress.total_bytes)}`
                                  : ""}
                              </span>
                              <span className="download-speed">
                                {formatSpeed(downloadProgress?.speed_bps || 0)}
                              </span>
                            </div>
                            <Button
                              variant="ghost"
                              size="small"
                              onClick={cancelDownload}
                              className="download-cancel-btn"
                            >
                              Cancel
                            </Button>
                          </div>
                        ) : isDownloaded ? (
                          <Button
                            variant={isLoaded ? "secondary" : "primary"}
                            size="small"
                            onClick={() =>
                              isLoaded
                                ? handleUnloadModel()
                                : handleLoadModel(model.id)
                            }
                            disabled={!!loading}
                          >
                            {isLoadingThis
                              ? "Loading..."
                              : isLoaded
                                ? "Unload"
                                : "Load"}
                          </Button>
                        ) : (
                          <Button
                            variant="secondary"
                            size="small"
                            onClick={() => handleDownloadModel(model.id)}
                            disabled={isDownloading}
                          >
                            Download
                          </Button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </>
          )}
        </div>

        <div className="custom-model-section">
          <h3>Custom Model</h3>
          <p className="settings-description">
            Load a GGUF-format model from your computer. Verified models load
            normally. Unverified files are blocked unless you enable the
            advanced override below.
          </p>
          <label className="toggle-label advanced-model-toggle">
            <input
              type="checkbox"
              checked={allowUnverifiedLocalModels}
              onChange={(event) => {
                void handleSetAllowUnverifiedLocalModels(event.target.checked);
              }}
            />
            <span className="toggle-text">
              Allow unverified local models (advanced)
            </span>
          </label>
          <p className="setting-note advanced-model-note">
            Keep this off unless you trust the GGUF file source. If you turn it
            on, AssistSupport still warns and asks for confirmation before
            loading an unverified file.
          </p>
          <Button
            variant="secondary"
            onClick={handleLoadCustomModel}
            disabled={!!loading || isDownloading}
          >
            {loading === "custom" ? "Loading..." : "Select GGUF File..."}
          </Button>
        </div>

        <div className="custom-model-section">
          <h3>AI Status &amp; Guarantees</h3>
          <p className="settings-description">
            AssistSupport runs AI locally and can operate fully offline. These
            signals help operators trust what the AI is doing.
          </p>
          <div className="settings-grid">
            <div className="settings-card">
              <h4>Local Guarantees</h4>
              <ul className="settings-list">
                <li>
                  <strong>Offline-first:</strong> no cloud AI calls
                </li>
                <li>
                  <strong>Copy gating:</strong> citations required (override
                  logs locally)
                </li>
                <li>
                  <strong>Prompts hidden:</strong> operators cannot edit system
                  prompts
                </li>
              </ul>
            </div>
            <div className="settings-card">
              <h4>Runtime Status</h4>
              <ul className="settings-list">
                <li>
                  <strong>Chat model:</strong>{" "}
                  {loadedModel ? loadedModel : "Not loaded"}
                </li>
                <li>
                  <strong>Embeddings:</strong>{" "}
                  {isEmbeddingLoaded ? "Loaded" : "Not loaded"}
                </li>
                <li>
                  <strong>Search API embedding:</strong>{" "}
                  {searchApiEmbeddingStatus?.ready
                    ? "Ready"
                    : searchApiEmbeddingStatus?.installed
                      ? "Installed but not ready"
                      : "Not installed"}
                </li>
                <li>
                  <strong>KB folder:</strong> {kbFolder ? kbFolder : "Not set"}
                </li>
                <li>
                  <strong>MemoryKernel:</strong>{" "}
                  {memoryKernelPreflight
                    ? memoryKernelPreflight.status
                    : "Unavailable"}
                  {memoryKernelPreflight?.service_contract_version
                    ? ` (svc ${memoryKernelPreflight.service_contract_version})`
                    : ""}
                </li>
              </ul>
              <div className="settings-actions-row">
                <Button
                  variant="ghost"
                  size="small"
                  onClick={refreshMemoryKernelStatus}
                  disabled={memoryKernelLoading}
                >
                  {memoryKernelLoading ? "Refreshing..." : "Refresh"}
                </Button>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section className="settings-section">
        <h2>Context Window</h2>
        <p className="settings-description">
          Configure the maximum context length for LLM generation. Larger values
          allow more content but use more memory.
        </p>
        <div className="context-window-config">
          <select
            className="context-window-select"
            aria-label="Context window size"
            value={contextWindowSize ?? ""}
            onChange={(e) => handleContextWindowChange(e.target.value)}
            disabled={!loadedModel}
          >
            {CONTEXT_WINDOW_OPTIONS.map((opt) => (
              <option key={opt.value ?? "default"} value={opt.value ?? ""}>
                {opt.label}
              </option>
            ))}
          </select>
          {!loadedModel && (
            <p className="setting-note">
              Load a model to configure context window.
            </p>
          )}
          <p className="setting-note">
            Higher values require more RAM. The "Model Default" option uses the
            model's training context (capped at 8K).
          </p>
        </div>
      </section>

      <section className="settings-section">
        <h2>Semantic Search Models</h2>
        <p className="settings-description">
          AssistSupport uses two separate local models for semantic search: one
          for the desktop knowledge base and one for the Python search API. Both
          are managed explicitly here and kept offline at runtime.
        </p>

        <div className="settings-grid semantic-model-grid">
          <div className="settings-card semantic-model-card">
            <h3>Desktop Embedding Model</h3>
            <p className="settings-description">
              Used for local knowledge-base embeddings and vector search. Uses{" "}
              <code>nomic-embed-text</code> (768 dimensions, about 550 MB).
            </p>
            <div className="embedding-model-config">
              {isDownloading &&
              downloadProgress?.model_id === "nomic-embed-text" ? (
                <div className="download-progress-container">
                  <div className="download-progress">
                    <div
                      className="download-bar"
                      style={{ width: `${downloadProgress?.percent || 0}%` }}
                    />
                    <span className="download-percent">
                      {Math.round(downloadProgress?.percent || 0)}%
                    </span>
                  </div>
                  <div className="download-info">
                    <span className="download-size">
                      {formatBytes(downloadProgress?.downloaded_bytes || 0)}
                      {downloadProgress?.total_bytes
                        ? ` / ${formatBytes(downloadProgress.total_bytes)}`
                        : ""}
                    </span>
                    <span className="download-speed">
                      {formatSpeed(downloadProgress?.speed_bps || 0)}
                    </span>
                  </div>
                  <Button
                    variant="ghost"
                    size="small"
                    onClick={cancelDownload}
                    className="download-cancel-btn"
                  >
                    Cancel
                  </Button>
                </div>
              ) : !embeddingDownloaded ? (
                <div className="embedding-status">
                  <span className="status-badge not-downloaded">
                    Not Downloaded
                  </span>
                  <Button
                    variant="primary"
                    size="small"
                    onClick={handleDownloadEmbeddingModel}
                    disabled={isDownloading}
                  >
                    Download Model
                  </Button>
                </div>
              ) : !isEmbeddingLoaded ? (
                <div className="embedding-status">
                  <span className="status-badge downloaded">Downloaded</span>
                  <Button
                    variant="primary"
                    size="small"
                    onClick={handleLoadEmbeddingModel}
                    disabled={embeddingLoading}
                  >
                    {embeddingLoading ? "Loading..." : "Load Model"}
                  </Button>
                </div>
              ) : (
                <div className="embedding-status">
                  <span className="status-badge loaded">Loaded</span>
                  <div className="embedding-info">
                    <span className="model-name">
                      {embeddingModelInfo?.name || "nomic-embed-text"}
                    </span>
                    <span className="model-dim">
                      {embeddingModelInfo?.embedding_dim || 768} dimensions
                    </span>
                  </div>
                  <Button
                    variant="secondary"
                    size="small"
                    onClick={handleUnloadEmbeddingModel}
                  >
                    Unload
                  </Button>
                </div>
              )}

              {vectorEnabled && isEmbeddingLoaded && (
                <div className="generate-embeddings-row">
                  <Button
                    variant="ghost"
                    size="small"
                    onClick={handleGenerateEmbeddings}
                    disabled={generatingEmbeddings}
                  >
                    {generatingEmbeddings
                      ? "Generating..."
                      : "Generate Embeddings for KB"}
                  </Button>
                  <p className="setting-note">
                    Creates vector embeddings for all indexed documents.
                  </p>
                </div>
              )}
            </div>
          </div>

          <div className="settings-card semantic-model-card">
            <h3>Search API Embedding Model</h3>
            <p className="settings-description">
              Used by the local Python hybrid search API. This managed install
              is pinned to a specific Hugging Face revision and loaded from
              local disk only.
            </p>
            <div className="embedding-status">
              <span
                className={`status-badge ${searchApiEmbeddingBadge.className}`}
              >
                {searchApiEmbeddingBadge.label}
              </span>
              <div className="embedding-info">
                <span className="model-name">
                  {searchApiEmbeddingStatus?.model_name ??
                    "sentence-transformers/all-MiniLM-L6-v2"}
                </span>
                <span className="model-dim">
                  {searchApiEmbeddingStatus?.local_path
                    ? "Managed local install"
                    : "No managed install detected"}
                </span>
              </div>
              <Button
                variant={
                  searchApiEmbeddingStatus?.ready ? "secondary" : "primary"
                }
                size="small"
                onClick={() => {
                  void handleInstallSearchApiEmbeddingModel();
                }}
                disabled={searchApiEmbeddingLoading}
              >
                {searchApiEmbeddingLoading
                  ? "Installing..."
                  : searchApiEmbeddingStatus?.ready
                    ? "Reinstall"
                    : "Install Model"}
              </Button>
            </div>
            <p className="setting-note semantic-model-note">
              {searchApiEmbeddingBadge.detail}
            </p>
            <div className="settings-actions-row">
              <Button
                variant="ghost"
                size="small"
                onClick={() => {
                  void refreshSearchApiEmbeddingStatus();
                }}
                disabled={searchApiEmbeddingLoading}
              >
                Refresh Status
              </Button>
            </div>
          </div>
        </div>
      </section>

      <section className="settings-section">
        <h2>Knowledge Base</h2>
        <p className="settings-description">
          Configure the folder containing your knowledge base documents.
        </p>

        <div className="kb-config">
          <div className="kb-folder-row">
            <div className="kb-folder-display">
              {kbFolder ? (
                <code>{kbFolder}</code>
              ) : (
                <span className="kb-placeholder">No folder selected</span>
              )}
            </div>
            <Button variant="secondary" onClick={handleSelectKbFolder}>
              {kbFolder ? "Change" : "Select Folder"}
            </Button>
          </div>

          {kbFolder && (
            <div className="kb-stats">
              <div className="stat-item">
                <span className="stat-label">Files indexed</span>
                <span className="stat-value">
                  {indexStats?.total_files ?? "—"}
                </span>
              </div>
              <div className="stat-item">
                <span className="stat-label">Total chunks</span>
                <span className="stat-value">
                  {indexStats?.total_chunks ?? "—"}
                </span>
              </div>
              <Button
                variant="ghost"
                size="small"
                onClick={handleRebuildIndex}
                disabled={loading === "rebuild"}
              >
                {loading === "rebuild" ? "Rebuilding..." : "Rebuild Index"}
              </Button>
            </div>
          )}
        </div>
      </section>

      <section className="settings-section">
        <h2>Advanced Search</h2>
        <p className="settings-description">
          Enable AI-powered semantic search for better knowledge base results.
        </p>
        <div className="vector-consent">
          <label className="toggle-label">
            <input
              type="checkbox"
              checked={vectorEnabled}
              onChange={handleVectorToggle}
            />
            <span className="toggle-text">Enable vector embeddings</span>
          </label>
          <p className="setting-note">
            Creates embeddings of your documents for semantic search. All
            processing happens locally on your machine.
          </p>
        </div>
      </section>

      <section className="settings-section">
        <h2>Template Variables</h2>
        <p className="settings-description">
          Define custom variables to use in response templates. Use as{" "}
          <code>{`{{variable_name}}`}</code> in your prompts.
        </p>

        <div className="variables-container">
          {customVariables.length === 0 ? (
            <p className="variables-empty">No custom variables defined yet.</p>
          ) : (
            <div className="variables-list">
              {customVariables.map((variable) => (
                <div key={variable.id} className="variable-item">
                  <div className="variable-info">
                    <code className="variable-name">{`{{${variable.name}}}`}</code>
                    <span className="variable-value">{variable.value}</span>
                  </div>
                  <div className="variable-actions">
                    <Button
                      variant="ghost"
                      size="small"
                      onClick={() => handleEditVariable(variable)}
                    >
                      Edit
                    </Button>
                    <Button
                      variant="ghost"
                      size="small"
                      onClick={() => handleDeleteVariable(variable.id)}
                    >
                      Delete
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}

          <Button variant="secondary" size="small" onClick={handleAddVariable}>
            + Add Variable
          </Button>
        </div>

        {showVariableForm && (
          <div
            className="variable-form-overlay"
            onClick={handleCancelVariableForm}
          >
            <div
              className="variable-form-modal"
              onClick={(e) => e.stopPropagation()}
            >
              <h3>{editingVariable ? "Edit Variable" : "Add Variable"}</h3>
              {variableFormError && (
                <div className="variable-form-error">{variableFormError}</div>
              )}
              <div className="form-field">
                <label htmlFor="var-name">Name</label>
                <input
                  id="var-name"
                  type="text"
                  placeholder="my_variable"
                  value={variableForm.name}
                  onChange={(e) =>
                    setVariableForm((f) => ({ ...f, name: e.target.value }))
                  }
                  autoFocus
                />
                <p className="field-hint">
                  Letters, numbers, and underscores only
                </p>
              </div>
              <div className="form-field">
                <label htmlFor="var-value">Value</label>
                <textarea
                  id="var-value"
                  placeholder="The value to substitute..."
                  value={variableForm.value}
                  onChange={(e) =>
                    setVariableForm((f) => ({ ...f, value: e.target.value }))
                  }
                  rows={3}
                />
              </div>
              <div className="form-actions">
                <Button variant="ghost" onClick={handleCancelVariableForm}>
                  Cancel
                </Button>
                <Button
                  variant="primary"
                  onClick={handleSaveVariable}
                  disabled={
                    !variableForm.name.trim() || !variableForm.value.trim()
                  }
                >
                  {editingVariable ? "Save" : "Add"}
                </Button>
              </div>
            </div>
          </div>
        )}
      </section>

      <section className="settings-section">
        <h2>Jira Integration</h2>
        <p className="settings-description">
          Connect to Jira Cloud to import tickets directly into your drafts.
        </p>

        {jiraConfigured ? (
          <div className="jira-connected">
            <div className="jira-status">
              <span className="status-icon">&#10003;</span>
              <span>Connected to {jiraConfig?.base_url || "Jira"}</span>
            </div>
            <p className="jira-email">Account: {jiraConfig?.email}</p>
            <Button
              variant="secondary"
              size="small"
              onClick={handleJiraDisconnect}
              disabled={jiraLoading}
            >
              Disconnect
            </Button>
          </div>
        ) : (
          <form className="jira-form" onSubmit={handleJiraConnect}>
            <div className="form-field">
              <label htmlFor="jira-url">Jira URL</label>
              <input
                id="jira-url"
                type="url"
                placeholder="https://your-company.atlassian.net"
                value={jiraForm.baseUrl}
                onChange={(e) =>
                  setJiraForm((f) => ({ ...f, baseUrl: e.target.value }))
                }
                required
              />
            </div>
            <div className="form-field">
              <label htmlFor="jira-email">Email</label>
              <input
                id="jira-email"
                type="email"
                placeholder="your.email@company.com"
                value={jiraForm.email}
                onChange={(e) =>
                  setJiraForm((f) => ({ ...f, email: e.target.value }))
                }
                required
              />
            </div>
            <div className="form-field">
              <label htmlFor="jira-token">API Token</label>
              <input
                id="jira-token"
                type="password"
                placeholder="Your Jira API token"
                value={jiraForm.apiToken}
                onChange={(e) =>
                  setJiraForm((f) => ({ ...f, apiToken: e.target.value }))
                }
                required
              />
              <p className="field-hint">
                Generate at{" "}
                <a
                  href="https://id.atlassian.com/manage/api-tokens"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  id.atlassian.com/manage/api-tokens
                </a>
              </p>
            </div>
            <Button
              type="submit"
              variant="primary"
              disabled={
                jiraLoading ||
                !jiraForm.baseUrl ||
                !jiraForm.email ||
                !jiraForm.apiToken
              }
            >
              {jiraLoading ? "Connecting..." : "Connect"}
            </Button>
          </form>
        )}
      </section>

      <DeploymentSection
        deploymentHealth={deploymentHealth}
        deployPreflightChecks={deployPreflightChecks}
        deployPreflightRunning={deployPreflightRunning}
        integrations={integrations}
        onRunDeploymentPreflight={() => {
          void handleRunDeploymentPreflight();
        }}
        onToggleIntegration={(integrationType, enabled) => {
          void handleToggleIntegration(integrationType, enabled);
        }}
      />

      <BackupSection
        backupLoading={backupLoading}
        onExportBackup={() => {
          void handleExportBackup();
        }}
        onImportBackup={() => {
          void handleImportBackup();
        }}
      />

      <QualityThresholdSection
        qualityThresholds={qualityThresholds}
        qualityThresholdError={qualityThresholdError}
        onThresholdChange={updateQualityThreshold}
        onSave={handleSaveQualityThresholds}
        onReset={handleResetQualityThresholds}
      />

      <AuditLogsSection
        auditLoading={auditLoading}
        auditExporting={auditExporting}
        auditSeverityFilter={auditSeverityFilter}
        auditSearchQuery={auditSearchQuery}
        filteredAuditEntriesCount={filteredAuditEntries.length}
        pagedAuditEntries={pagedAuditEntries}
        auditPage={auditPage}
        auditTotalPages={auditTotalPages}
        formatAuditEvent={formatAuditEvent}
        onRefresh={() => {
          void loadAuditEntries();
        }}
        onExport={() => {
          void handleExportAuditLog();
        }}
        onSeverityChange={(value) => {
          setAuditSeverityFilter(value);
          setAuditPage(1);
        }}
        onSearchQueryChange={(value) => {
          setAuditSearchQuery(value);
          setAuditPage(1);
        }}
        onPreviousPage={() => setAuditPage((p) => Math.max(1, p - 1))}
        onNextPage={() => setAuditPage((p) => Math.min(auditTotalPages, p + 1))}
      />

      <AboutSection versionLabel={formatAppVersion(APP_VERSION)} />
    </div>
  );
}
