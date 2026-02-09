import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle, CheckCircle, XCircle, Loader2, RotateCw } from "lucide-react";
import { useSettingsStore } from "../stores/settingsStore";
import { useCollectionStore } from "../stores/collectionStore";
import { useToastStore } from "../stores/toastStore";
import { useTheme } from "../hooks/useTheme";
import type { OllamaModel } from "../types";

export function SettingsView() {
  const settings = useSettingsStore((state) => state.settings);
  const updateSetting = useSettingsStore((state) => state.updateSetting);
  const activeCollectionId = useCollectionStore((s) => s.activeCollectionId);
  const addToast = useToastStore((s) => s.addToast);
  const { theme, setTheme } = useTheme();

  const [chunkSettingsChanged, setChunkSettingsChanged] = useState(false);
  const [reingesting, setReingesting] = useState(false);
  const initialChunkSize = useRef(settings.chunk_size);
  const initialChunkOverlap = useRef(settings.chunk_overlap);

  const [ollamaHost, setOllamaHost] = useState("");
  const [ollamaPort, setOllamaPort] = useState("");
  const [testStatus, setTestStatus] = useState<
    "idle" | "testing" | "success" | "error"
  >("idle");
  const [testMessage, setTestMessage] = useState("");
  const [models, setModels] = useState<OllamaModel[]>([]);

  useEffect(() => {
    setOllamaHost(settings.ollama_host ?? "localhost");
    setOllamaPort(settings.ollama_port ?? "11434");
  }, [settings.ollama_host, settings.ollama_port]);

  useEffect(() => {
    if (initialChunkSize.current === undefined) {
      initialChunkSize.current = settings.chunk_size;
      initialChunkOverlap.current = settings.chunk_overlap;
      return;
    }
    const sizeChanged = settings.chunk_size !== initialChunkSize.current;
    const overlapChanged = settings.chunk_overlap !== initialChunkOverlap.current;
    setChunkSettingsChanged(sizeChanged || overlapChanged);
  }, [settings.chunk_size, settings.chunk_overlap]);

  const handleReingestAll = useCallback(async () => {
    if (!activeCollectionId) {
      addToast("error", "No collection selected");
      return;
    }
    setReingesting(true);
    try {
      await invoke("reingest_collection", { collectionId: activeCollectionId });
      addToast("success", "Re-ingestion started for all documents");
      setChunkSettingsChanged(false);
      initialChunkSize.current = settings.chunk_size;
      initialChunkOverlap.current = settings.chunk_overlap;
    } catch (error) {
      addToast("error", "Failed to start re-ingestion: " + String(error));
    } finally {
      setReingesting(false);
    }
  }, [activeCollectionId, addToast, settings.chunk_size, settings.chunk_overlap]);

  const fetchModels = useCallback(async () => {
    try {
      const result = await invoke<OllamaModel[]>("list_ollama_models");
      setModels(result);
    } catch {
      setModels([]);
    }
  }, []);

  useEffect(() => {
    fetchModels();
  }, [fetchModels]);

  async function handleTestConnection() {
    setTestStatus("testing");
    try {
      const result = await invoke<[boolean, string]>("test_ollama_connection", {
        host: ollamaHost,
        port: ollamaPort,
      });
      if (result[0]) {
        setTestStatus("success");
        setTestMessage(`Connected - v${result[1]}`);
        await updateSetting("ollama_host", ollamaHost);
        await updateSetting("ollama_port", ollamaPort);
        fetchModels();
      } else {
        setTestStatus("error");
        setTestMessage("Connection failed");
      }
    } catch {
      setTestStatus("error");
      setTestMessage("Connection failed");
    }
  }

  return (
    <div className="flex-1 overflow-y-auto p-6 scrollbar-thin">
      <h1 className="mb-6 text-xl font-semibold text-foreground">Settings</h1>

      <div className="mx-auto max-w-2xl space-y-8">
        {/* Ollama Connection */}
        <section className="rounded-lg border border-border bg-card p-5">
          <h2 className="mb-4 text-sm font-semibold text-card-foreground">
            Ollama Connection
          </h2>
          <div className="space-y-4">
            <div className="flex gap-3">
              <div className="flex-1">
                <label className="mb-1 block text-xs text-muted-foreground">
                  Host
                </label>
                <input
                  type="text"
                  value={ollamaHost}
                  onChange={(e) => setOllamaHost(e.target.value)}
                  className="h-9 w-full rounded-md border border-border bg-background px-3 text-sm text-foreground outline-none focus:ring-1 focus:ring-accent"
                />
              </div>
              <div className="w-28">
                <label className="mb-1 block text-xs text-muted-foreground">
                  Port
                </label>
                <input
                  type="text"
                  value={ollamaPort}
                  onChange={(e) => setOllamaPort(e.target.value)}
                  className="h-9 w-full rounded-md border border-border bg-background px-3 text-sm text-foreground outline-none focus:ring-1 focus:ring-accent"
                />
              </div>
            </div>

            <div className="flex items-center gap-3">
              <button
                onClick={handleTestConnection}
                disabled={testStatus === "testing"}
                className="flex h-9 items-center gap-2 rounded-md bg-accent px-4 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90 disabled:opacity-50"
              >
                {testStatus === "testing" && (
                  <Loader2 size={14} className="animate-spin" />
                )}
                Test Connection
              </button>
              {testStatus === "success" && (
                <span className="flex items-center gap-1 text-xs text-success">
                  <CheckCircle size={14} /> {testMessage}
                </span>
              )}
              {testStatus === "error" && (
                <span className="flex items-center gap-1 text-xs text-destructive">
                  <XCircle size={14} /> {testMessage}
                </span>
              )}
            </div>

            <div className="flex gap-3">
              <div className="flex-1">
                <label className="mb-1 block text-xs text-muted-foreground">
                  Embedding Model
                </label>
                <select
                  value={settings.embedding_model ?? ""}
                  onChange={(e) =>
                    updateSetting("embedding_model", e.target.value)
                  }
                  className="h-9 w-full rounded-md border border-border bg-background px-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-accent"
                >
                  <option value="">Select model...</option>
                  {models.map((m) => (
                    <option key={m.name} value={m.name}>
                      {m.name}
                    </option>
                  ))}
                </select>
              </div>
              <div className="flex-1">
                <label className="mb-1 block text-xs text-muted-foreground">
                  Chat Model
                </label>
                <select
                  value={settings.chat_model ?? ""}
                  onChange={(e) => updateSetting("chat_model", e.target.value)}
                  className="h-9 w-full rounded-md border border-border bg-background px-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-accent"
                >
                  <option value="">Select model...</option>
                  {models.map((m) => (
                    <option key={m.name} value={m.name}>
                      {m.name}
                    </option>
                  ))}
                </select>
              </div>
            </div>
          </div>
        </section>

        {/* Chunking */}
        <section className="rounded-lg border border-border bg-card p-5">
          <h2 className="mb-4 text-sm font-semibold text-card-foreground">
            Chunking
          </h2>
          <div className="mb-4 flex items-start gap-2 rounded-md bg-warning/10 p-3 text-xs text-warning">
            <AlertTriangle size={14} className="mt-0.5 shrink-0" />
            <span>
              Changing these settings requires re-ingesting documents
            </span>
          </div>
          {chunkSettingsChanged && (
            <div className="mb-4 flex items-center justify-between rounded-md bg-accent/10 p-3">
              <span className="text-xs text-accent">
                Chunk settings changed. Re-ingest documents to apply.
              </span>
              <button
                onClick={handleReingestAll}
                disabled={reingesting}
                className="flex items-center gap-1 rounded-md bg-accent px-3 py-1 text-xs font-medium text-accent-foreground hover:bg-accent/90 disabled:opacity-50"
              >
                <RotateCw size={12} className={reingesting ? "animate-spin" : ""} />
                Re-ingest All
              </button>
            </div>
          )}
          <div className="flex gap-3">
            <div className="flex-1">
              <label className="mb-1 block text-xs text-muted-foreground">
                Chunk Size (tokens)
              </label>
              <select
                value={settings.chunk_size ?? "512"}
                onChange={(e) => updateSetting("chunk_size", e.target.value)}
                className="h-9 w-full rounded-md border border-border bg-background px-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-accent"
              >
                <option value="256">256</option>
                <option value="512">512</option>
                <option value="1024">1024</option>
                <option value="2048">2048</option>
              </select>
            </div>
            <div className="flex-1">
              <label className="mb-1 block text-xs text-muted-foreground">
                Chunk Overlap (tokens)
              </label>
              <select
                value={settings.chunk_overlap ?? "64"}
                onChange={(e) => updateSetting("chunk_overlap", e.target.value)}
                className="h-9 w-full rounded-md border border-border bg-background px-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-accent"
              >
                <option value="0">0</option>
                <option value="64">64</option>
                <option value="128">128</option>
                <option value="256">256</option>
              </select>
            </div>
          </div>
        </section>

        {/* Appearance */}
        <section className="rounded-lg border border-border bg-card p-5">
          <h2 className="mb-4 text-sm font-semibold text-card-foreground">
            Appearance
          </h2>
          <div className="flex gap-3">
            {(["light", "dark", "system"] as const).map((option) => (
              <label
                key={option}
                className={`flex cursor-pointer items-center gap-2 rounded-md border px-4 py-2 text-sm transition-colors ${
                  theme === option
                    ? "border-accent bg-accent/10 text-accent"
                    : "border-border text-muted-foreground hover:border-accent/50"
                }`}
              >
                <input
                  type="radio"
                  name="theme"
                  value={option}
                  checked={theme === option}
                  onChange={() => setTheme(option)}
                  className="sr-only"
                />
                {option.charAt(0).toUpperCase() + option.slice(1)}
              </label>
            ))}
          </div>
        </section>

        {/* Data */}
        <section className="rounded-lg border border-border bg-card p-5">
          <h2 className="mb-4 text-sm font-semibold text-card-foreground">
            Data
          </h2>
          <div className="space-y-3">
            <div>
              <label className="mb-1 block text-xs text-muted-foreground">
                Database Path
              </label>
              <p className="text-sm text-foreground">
                {settings.db_path ?? "~/Library/Application Support/VaultMind/vaultmind.db"}
              </p>
            </div>
            <button
              disabled
              className="rounded-md border border-destructive px-4 py-2 text-sm text-destructive opacity-50"
            >
              Clear All Data
            </button>
          </div>
        </section>
      </div>
    </div>
  );
}
