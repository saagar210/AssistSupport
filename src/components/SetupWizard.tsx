import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Loader2, CheckCircle, XCircle, ChevronRight } from "lucide-react";
import type { OllamaModel } from "../types";

interface SetupWizardProps {
  onComplete: () => void;
}

interface OllamaCheckResult {
  connected: boolean;
  version: string;
}

export function SetupWizard({ onComplete }: SetupWizardProps) {
  const [step, setStep] = useState(0);

  // Step 1 state
  const [ollamaConnected, setOllamaConnected] = useState(false);
  const [ollamaChecking, setOllamaChecking] = useState(false);
  const [ollamaVersion, setOllamaVersion] = useState("");

  // Step 2 state
  const [models, setModels] = useState<OllamaModel[]>([]);
  const [embeddingModel, setEmbeddingModel] = useState("");
  const [chatModel, setChatModel] = useState("");
  const [modelsLoading, setModelsLoading] = useState(false);

  const checkOllama = useCallback(async () => {
    setOllamaChecking(true);
    try {
      const result = await invoke<OllamaCheckResult>("check_ollama_connection");
      setOllamaConnected(result.connected);
      setOllamaVersion(result.version);
    } catch {
      setOllamaConnected(false);
      setOllamaVersion("");
    } finally {
      setOllamaChecking(false);
    }
  }, []);

  const fetchModels = useCallback(async () => {
    setModelsLoading(true);
    try {
      const result = await invoke<OllamaModel[]>("list_ollama_models");
      setModels(result);
    } catch {
      setModels([]);
    } finally {
      setModelsLoading(false);
    }
  }, []);

  // Auto-check Ollama when entering step 1
  useEffect(() => {
    if (step === 1) {
      checkOllama();
    }
  }, [step, checkOllama]);

  // Fetch models when entering step 2
  useEffect(() => {
    if (step === 2) {
      fetchModels();
    }
  }, [step, fetchModels]);

  const handleFinish = useCallback(async () => {
    try {
      if (embeddingModel) {
        await invoke("update_setting", { key: "embedding_model", value: embeddingModel });
      }
      if (chatModel) {
        await invoke("update_setting", { key: "chat_model", value: chatModel });
      }
      await invoke("update_setting", { key: "setup_complete", value: "true" });
      onComplete();
    } catch (error) {
      console.error("Failed to save setup settings:", error);
    }
  }, [embeddingModel, chatModel, onComplete]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/95 backdrop-blur-sm">
      <div className="w-full max-w-lg rounded-xl border border-border bg-card p-8 shadow-2xl">
        {/* Progress dots */}
        <div className="mb-8 flex justify-center gap-2">
          {[0, 1, 2, 3].map((i) => (
            <div
              key={i}
              className={`h-2 w-2 rounded-full transition-colors ${
                i === step ? "bg-accent" : i < step ? "bg-accent/50" : "bg-muted"
              }`}
            />
          ))}
        </div>

        {/* Step 0: Welcome */}
        {step === 0 && (
          <div className="text-center">
            <h1 className="mb-3 text-2xl font-bold text-foreground">Welcome to VaultMind</h1>
            <p className="mb-2 text-sm text-muted-foreground">
              Your local-first AI knowledge management system.
            </p>
            <p className="mb-8 text-sm text-muted-foreground">
              Import documents, build a knowledge graph, and chat with your data -- all
              powered by local AI models through Ollama.
            </p>
            <button
              onClick={() => setStep(1)}
              className="inline-flex items-center gap-2 rounded-lg bg-accent px-6 py-2.5 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90"
            >
              Get Started
              <ChevronRight size={16} />
            </button>
          </div>
        )}

        {/* Step 1: Connect to Ollama */}
        {step === 1 && (
          <div className="text-center">
            <h2 className="mb-2 text-xl font-bold text-foreground">Connect to Ollama</h2>
            <p className="mb-6 text-sm text-muted-foreground">
              VaultMind uses Ollama for local AI inference. Make sure Ollama is running.
            </p>

            <div className="mb-6 flex items-center justify-center gap-3 rounded-lg border border-border bg-muted/50 px-4 py-3">
              {ollamaChecking ? (
                <>
                  <Loader2 size={20} className="animate-spin text-accent" />
                  <span className="text-sm text-muted-foreground">Checking connection...</span>
                </>
              ) : ollamaConnected ? (
                <>
                  <CheckCircle size={20} className="text-green-500" />
                  <span className="text-sm text-foreground">
                    Connected{ollamaVersion ? ` (v${ollamaVersion})` : ""}
                  </span>
                </>
              ) : (
                <>
                  <XCircle size={20} className="text-destructive" />
                  <span className="text-sm text-muted-foreground">Not connected</span>
                </>
              )}
            </div>

            {!ollamaConnected && !ollamaChecking && (
              <button
                onClick={checkOllama}
                className="mb-4 text-xs text-accent hover:underline"
              >
                Retry connection
              </button>
            )}

            <div className="flex justify-between">
              <button
                onClick={() => setStep(0)}
                className="rounded-lg px-4 py-2 text-sm text-muted-foreground transition-colors hover:text-foreground"
              >
                Back
              </button>
              <button
                onClick={() => setStep(2)}
                className="inline-flex items-center gap-2 rounded-lg bg-accent px-6 py-2.5 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90"
              >
                {ollamaConnected ? "Next" : "Skip"}
                <ChevronRight size={16} />
              </button>
            </div>
          </div>
        )}

        {/* Step 2: Choose Models */}
        {step === 2 && (
          <div>
            <h2 className="mb-2 text-center text-xl font-bold text-foreground">Choose Your Models</h2>
            <p className="mb-6 text-center text-sm text-muted-foreground">
              Select which Ollama models to use for embedding and chat.
            </p>

            {modelsLoading ? (
              <div className="mb-6 flex items-center justify-center gap-2">
                <Loader2 size={16} className="animate-spin text-accent" />
                <span className="text-sm text-muted-foreground">Loading models...</span>
              </div>
            ) : models.length === 0 ? (
              <p className="mb-6 text-center text-sm text-muted-foreground">
                No models found. You can configure models later in Settings.
              </p>
            ) : (
              <div className="mb-6 space-y-4">
                <div>
                  <label className="mb-1 block text-xs font-medium text-foreground">
                    Embedding Model
                  </label>
                  <select
                    value={embeddingModel}
                    onChange={(e) => setEmbeddingModel(e.target.value)}
                    className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none transition-colors focus:border-accent"
                  >
                    <option value="">Select a model...</option>
                    {models.map((m) => (
                      <option key={m.name} value={m.name}>
                        {m.name}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="mb-1 block text-xs font-medium text-foreground">
                    Chat Model
                  </label>
                  <select
                    value={chatModel}
                    onChange={(e) => setChatModel(e.target.value)}
                    className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground outline-none transition-colors focus:border-accent"
                  >
                    <option value="">Select a model...</option>
                    {models.map((m) => (
                      <option key={m.name} value={m.name}>
                        {m.name}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
            )}

            <div className="flex justify-between">
              <button
                onClick={() => setStep(1)}
                className="rounded-lg px-4 py-2 text-sm text-muted-foreground transition-colors hover:text-foreground"
              >
                Back
              </button>
              <button
                onClick={() => setStep(3)}
                className="inline-flex items-center gap-2 rounded-lg bg-accent px-6 py-2.5 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90"
              >
                Next
                <ChevronRight size={16} />
              </button>
            </div>
          </div>
        )}

        {/* Step 3: Ready */}
        {step === 3 && (
          <div className="text-center">
            <h2 className="mb-2 text-xl font-bold text-foreground">You're All Set!</h2>
            <p className="mb-6 text-sm text-muted-foreground">
              Here's a summary of your configuration:
            </p>

            <div className="mb-6 space-y-2 rounded-lg border border-border bg-muted/50 p-4 text-left text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Ollama</span>
                <span className={ollamaConnected ? "text-green-500" : "text-muted-foreground"}>
                  {ollamaConnected ? "Connected" : "Not connected"}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Embedding Model</span>
                <span className="text-foreground">{embeddingModel || "Default"}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Chat Model</span>
                <span className="text-foreground">{chatModel || "Default"}</span>
              </div>
            </div>

            <div className="flex justify-between">
              <button
                onClick={() => setStep(2)}
                className="rounded-lg px-4 py-2 text-sm text-muted-foreground transition-colors hover:text-foreground"
              >
                Back
              </button>
              <button
                onClick={handleFinish}
                className="rounded-lg bg-accent px-6 py-2.5 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90"
              >
                Start Using VaultMind
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
