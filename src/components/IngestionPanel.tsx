import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { ChevronDown, ChevronUp, CheckCircle, XCircle, Loader2 } from "lucide-react";
import type { IngestionProgress } from "../types";

interface FileProgress {
  document_id: string;
  filename: string;
  stage: IngestionProgress["stage"];
  chunks_done: number;
  chunks_total: number;
  error?: string;
}

const STAGE_LABELS: Record<string, string> = {
  parsing: "Parsing...",
  chunking: "Chunking...",
  embedding: "Embedding...",
  indexing: "Indexing...",
  complete: "Complete",
  failed: "Failed",
};

export function IngestionPanel() {
  const [files, setFiles] = useState<Map<string, FileProgress>>(new Map());
  const [collapsed, setCollapsed] = useState(false);
  const [dismissTimer, setDismissTimer] = useState<ReturnType<typeof setTimeout> | null>(null);

  const updateFile = useCallback((progress: IngestionProgress) => {
    setFiles((prev) => {
      const next = new Map(prev);
      next.set(progress.document_id, {
        document_id: progress.document_id,
        filename: progress.filename,
        stage: progress.stage,
        chunks_done: progress.chunks_done,
        chunks_total: progress.chunks_total,
        error: progress.error,
      });
      return next;
    });
  }, []);

  useEffect(() => {
    const unlistenProgress = listen<IngestionProgress>("ingestion-progress", (event) => {
      updateFile(event.payload);
    });

    const unlistenComplete = listen("ingestion-all-complete", () => {
      const timer = setTimeout(() => {
        setFiles(new Map());
      }, 3000);
      setDismissTimer(timer);
    });

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, [updateFile]);

  // Clear dismiss timer if new files come in
  useEffect(() => {
    const hasActive = Array.from(files.values()).some(
      (f) => f.stage !== "complete" && f.stage !== "failed"
    );
    if (hasActive && dismissTimer) {
      clearTimeout(dismissTimer);
      setDismissTimer(null);
    }
  }, [files, dismissTimer]);

  if (files.size === 0) return null;

  const fileList = Array.from(files.values());
  const activeCount = fileList.filter(
    (f) => f.stage !== "complete" && f.stage !== "failed"
  ).length;
  const completedCount = fileList.filter((f) => f.stage === "complete").length;
  const failedCount = fileList.filter((f) => f.stage === "failed").length;

  return (
    <div className="fixed bottom-10 right-4 z-40 w-80 rounded-lg border border-border bg-card shadow-xl">
      {/* Header */}
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="flex w-full items-center justify-between px-3 py-2 text-xs font-medium text-card-foreground"
      >
        <span className="flex items-center gap-2">
          {activeCount > 0 && (
            <Loader2 size={12} className="animate-spin text-accent" />
          )}
          {activeCount > 0
            ? `Importing ${activeCount} file${activeCount > 1 ? "s" : ""}...`
            : `Import complete (${completedCount} done${failedCount > 0 ? `, ${failedCount} failed` : ""})`}
        </span>
        {collapsed ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
      </button>

      {/* File list */}
      {!collapsed && (
        <div className="max-h-60 overflow-y-auto border-t border-border px-3 py-2 space-y-2">
          {fileList.map((file) => (
            <div key={file.document_id} className="space-y-1">
              <div className="flex items-center justify-between">
                <span className="truncate text-xs text-foreground" title={file.filename}>
                  {file.filename}
                </span>
                <span className="ml-2 shrink-0">
                  {file.stage === "complete" && (
                    <CheckCircle size={12} className="text-success" />
                  )}
                  {file.stage === "failed" && (
                    <XCircle size={12} className="text-destructive" />
                  )}
                  {file.stage !== "complete" && file.stage !== "failed" && (
                    <Loader2 size={12} className="animate-spin text-muted-foreground" />
                  )}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-[10px] text-muted-foreground">
                  {file.stage === "failed" && file.error
                    ? file.error.slice(0, 50)
                    : STAGE_LABELS[file.stage] ?? file.stage}
                </span>
                {file.stage === "embedding" && file.chunks_total > 0 && (
                  <span className="text-[10px] text-muted-foreground">
                    ({file.chunks_done}/{file.chunks_total})
                  </span>
                )}
              </div>
              {file.stage === "embedding" && file.chunks_total > 0 && (
                <div className="h-1 w-full rounded-full bg-muted">
                  <div
                    className="h-full rounded-full bg-accent transition-all"
                    style={{
                      width: `${(file.chunks_done / file.chunks_total) * 100}%`,
                    }}
                  />
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
