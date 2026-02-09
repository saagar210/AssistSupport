import { useEffect } from "react";
import { useOllamaStatus } from "../hooks/useOllamaStatus";
import { useDocumentStore } from "../stores/documentStore";
import { useCollectionStore } from "../stores/collectionStore";

export function StatusBar() {
  const { connected, loading } = useOllamaStatus();
  const docCount = useDocumentStore((s) => s.docCount);
  const chunkCount = useDocumentStore((s) => s.chunkCount);
  const fetchStats = useDocumentStore((s) => s.fetchStats);
  const activeCollectionId = useCollectionStore((s) => s.activeCollectionId);

  useEffect(() => {
    if (activeCollectionId) {
      fetchStats(activeCollectionId);
    }
  }, [activeCollectionId, fetchStats]);

  return (
    <div className="flex h-8 items-center justify-between border-t border-border bg-muted px-3 text-xs text-muted-foreground">
      <div className="flex items-center gap-2">
        <span
          className={`inline-block h-2 w-2 rounded-full ${
            loading
              ? "bg-warning"
              : connected
                ? "bg-success"
                : "bg-destructive"
          }`}
        />
        <span>
          {loading
            ? "Checking Ollama..."
            : connected
              ? "Ollama connected"
              : "Ollama disconnected"}
        </span>
      </div>

      <div>
        {docCount} document{docCount !== 1 ? "s" : ""} &middot;{" "}
        {chunkCount} chunk{chunkCount !== 1 ? "s" : ""}
      </div>
    </div>
  );
}
