import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import {
  FileText,
  Plus,
  Upload,
  ArrowUpDown,
  Calendar,
  Type,
  Trash2,
  Loader2,
  AlertCircle,
  CheckCircle2,
  File,
  RefreshCw,
} from "lucide-react";
import { useCollectionStore } from "../stores/collectionStore";
import { useDocumentStore } from "../stores/documentStore";
import { useAppStore } from "../stores/appStore";
import { useToastStore } from "../stores/toastStore";
import { ContextualHelp } from "../components/ContextualHelp";
import { getFileTypeBadgeColor } from "../utils/fileTypeColors";
import { DocumentGridSkeleton } from "../components/LoadingSkeleton";
import { EmptyState } from "../components/EmptyState";
import type { Document, IngestionProgress } from "../types";

type SortKey = "name" | "date" | "type";

const SUPPORTED_EXTENSIONS = [
  { name: "All Supported", extensions: ["pdf", "md", "html", "txt", "docx", "csv", "epub"] },
  { name: "PDF", extensions: ["pdf"] },
  { name: "Markdown", extensions: ["md"] },
  { name: "HTML", extensions: ["html"] },
  { name: "Text", extensions: ["txt"] },
  { name: "Word", extensions: ["docx"] },
  { name: "CSV", extensions: ["csv"] },
  { name: "EPUB", extensions: ["epub"] },
];

function getFileTypeBadge(fileType: string) {
  const colorClass = getFileTypeBadgeColor(fileType);
  return (
    <span
      className={`inline-flex items-center rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase text-white ${colorClass}`}
    >
      {fileType}
    </span>
  );
}

function getStatusBadge(status: string) {
  switch (status) {
    case "completed":
      return (
        <span className="inline-flex items-center gap-1 rounded-full bg-success/10 px-2 py-0.5 text-[10px] font-medium text-success">
          <CheckCircle2 size={10} /> Completed
        </span>
      );
    case "processing":
      return (
        <span className="inline-flex items-center gap-1 rounded-full bg-warning/10 px-2 py-0.5 text-[10px] font-medium text-warning">
          <Loader2 size={10} className="animate-spin" /> Processing
        </span>
      );
    case "failed":
      return (
        <span className="inline-flex items-center gap-1 rounded-full bg-destructive/10 px-2 py-0.5 text-[10px] font-medium text-destructive">
          <AlertCircle size={10} /> Failed
        </span>
      );
    default:
      return (
        <span className="inline-flex items-center rounded-full bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground">
          {status}
        </span>
      );
  }
}

function sortDocuments(docs: Document[], sortKey: SortKey): Document[] {
  return [...docs].sort((a, b) => {
    switch (sortKey) {
      case "name":
        return a.filename.localeCompare(b.filename);
      case "date":
        return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      case "type":
        return a.file_type.localeCompare(b.file_type);
    }
  });
}

export function DocumentsView() {
  const activeCollectionId = useCollectionStore((s) => s.activeCollectionId);
  const documents = useDocumentStore((s) => s.documents);
  const loading = useDocumentStore((s) => s.loading);
  const error = useDocumentStore((s) => s.error);
  const fetchDocuments = useDocumentStore((s) => s.fetchDocuments);
  const deleteDocument = useDocumentStore((s) => s.deleteDocument);
  const fetchStats = useDocumentStore((s) => s.fetchStats);
  const setActiveView = useAppStore((s) => s.setActiveView);
  const setSelectedDocument = useAppStore((s) => s.setSelectedDocument);
  const addToast = useToastStore((s) => s.addToast);

  const [dragging, setDragging] = useState(false);
  const [sortKey, setSortKey] = useState<SortKey>("date");
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const [ingesting, setIngesting] = useState(false);
  const dragCounter = useRef(0);

  useEffect(() => {
    if (activeCollectionId) {
      fetchDocuments(activeCollectionId);
      fetchStats(activeCollectionId);
    }
  }, [activeCollectionId, fetchDocuments, fetchStats]);

  useEffect(() => {
    const unlisten = listen<IngestionProgress>("ingestion-progress", (event) => {
      const progress = event.payload;
      if (progress.stage === "complete" || progress.stage === "failed") {
        if (activeCollectionId) {
          fetchDocuments(activeCollectionId);
          fetchStats(activeCollectionId);
        }
        if (progress.stage === "failed" && progress.error) {
          addToast("error", `Ingestion failed: ${progress.error}`);
        }
        if (progress.stage === "complete") {
          addToast("success", "Document ingested successfully");
        }
        setIngesting(false);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [activeCollectionId, fetchDocuments, fetchStats, addToast]);

  const handleFileDrop = useCallback(
    async (filePaths: string[]) => {
      if (!activeCollectionId || filePaths.length === 0) return;
      setIngesting(true);
      try {
        await invoke("ingest_files", {
          collectionId: activeCollectionId,
          filePaths,
        });
        addToast("info", `Ingesting ${filePaths.length} file(s)...`);
      } catch (error) {
        console.error("Failed to ingest files:", error);
        addToast("error", `Ingestion error: ${String(error)}`);
        setIngesting(false);
      }
    },
    [activeCollectionId, addToast],
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter.current += 1;
    setDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter.current = Math.max(0, dragCounter.current - 1);
    if (dragCounter.current === 0) {
      setDragging(false);
    }
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setDragging(false);
      dragCounter.current = 0;

      // HTML5 drag API does not expose full file paths, only file names.
      // Use the "Add Documents" button (Tauri file dialog) to get proper paths.
      addToast("info", "Please use the 'Add Documents' button to import files");
    },
    [addToast],
  );

  const handleOpenDialog = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: SUPPORTED_EXTENSIONS,
      });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        handleFileDrop(paths);
      }
    } catch (error) {
      console.error("File dialog error:", error);
    }
  }, [handleFileDrop]);

  const handleDeleteDocument = useCallback(
    async (id: string) => {
      if (!activeCollectionId) return;
      await deleteDocument(id, activeCollectionId);
      setDeleteConfirmId(null);
      addToast("success", "Document deleted");
    },
    [activeCollectionId, deleteDocument, addToast],
  );

  const handleDocumentClick = useCallback(
    (doc: Document) => {
      setSelectedDocument(doc.id);
      setActiveView("document-detail");
    },
    [setSelectedDocument, setActiveView],
  );

  const sorted = sortDocuments(documents, sortKey);
  const hasDocuments = documents.length > 0;

  if (!activeCollectionId) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-4 text-muted-foreground">
        <FileText size={48} strokeWidth={1.5} />
        <p className="text-sm">Select or create a collection to get started</p>
      </div>
    );
  }

  if (loading && documents.length === 0) {
    return <DocumentGridSkeleton />;
  }

  if (error && documents.length === 0) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-4 text-muted-foreground">
        <AlertCircle size={48} strokeWidth={1.5} className="text-destructive" />
        <p className="text-sm text-destructive">{error}</p>
        <button
          onClick={() => activeCollectionId && fetchDocuments(activeCollectionId)}
          className="flex items-center gap-2 rounded-lg bg-accent px-4 py-2 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90"
        >
          <RefreshCw size={16} />
          Retry
        </button>
        <div aria-live="polite" className="sr-only">
          {error}
        </div>
      </div>
    );
  }

  if (!hasDocuments) {
    return (
      <div
        className={`flex flex-1 flex-col items-center justify-center gap-4 transition-colors ${
          dragging ? "bg-accent/5" : ""
        }`}
        onDragOver={handleDragOver}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <EmptyState
          icon={Upload}
          title={dragging ? "Drop files to import" : "No documents yet"}
          description="Drop files here or click Import to get started. Supports PDF, Markdown, HTML, TXT, DOCX, CSV, EPUB."
          action={
            <button
              onClick={handleOpenDialog}
              className="flex items-center gap-2 rounded-lg bg-accent px-4 py-2 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90"
            >
              <Plus size={16} />
              Add Documents
            </button>
          }
        />
      </div>
    );
  }

  return (
    <div
      className={`flex flex-1 flex-col overflow-hidden transition-colors ${
        dragging ? "bg-accent/5" : ""
      }`}
      onDragOver={handleDragOver}
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* Toolbar */}
      <div className="flex items-center justify-between border-b border-border px-4 py-2">
        <div className="flex items-center gap-2">
          <ContextualHelp topic="documents" />
          <button
            onClick={handleOpenDialog}
            disabled={ingesting}
            className="flex items-center gap-2 rounded-lg bg-accent px-3 py-1.5 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90 disabled:opacity-50"
          >
            {ingesting ? (
              <Loader2 size={14} className="animate-spin" />
            ) : (
              <Plus size={14} />
            )}
            Add Documents
          </button>
          <span aria-live="polite" className="text-sm text-accent font-medium">
            {dragging ? "Drop files to import" : ingesting ? "Ingesting documents..." : ""}
          </span>
        </div>
        <div className="flex items-center gap-1">
          <span className="mr-2 text-xs text-muted-foreground">Sort:</span>
          {(["name", "date", "type"] as const).map((key) => {
            const icons = { name: ArrowUpDown, date: Calendar, type: Type };
            const Icon = icons[key];
            return (
              <button
                key={key}
                onClick={() => setSortKey(key)}
                className={`flex items-center gap-1 rounded px-2 py-1 text-xs transition-colors ${
                  sortKey === key
                    ? "bg-accent/10 text-accent"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                <Icon size={12} />
                {key.charAt(0).toUpperCase() + key.slice(1)}
              </button>
            );
          })}
        </div>
      </div>

      {/* Document Grid */}
      <div className="flex-1 overflow-y-auto p-4 scrollbar-thin">
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {sorted.map((doc) => (
            <div
              key={doc.id}
              role="button"
              tabIndex={0}
              onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); handleDocumentClick(doc); } }}
              className="group relative cursor-pointer rounded-lg border border-border bg-card p-4 transition-all duration-150 hover:border-accent/50 hover:shadow-md"
              onClick={() => handleDocumentClick(doc)}
            >
              <div className="mb-3 flex items-start justify-between">
                <div className="flex items-center gap-2">
                  <File size={16} className="shrink-0 text-muted-foreground" />
                  {getFileTypeBadge(doc.file_type)}
                </div>
                {getStatusBadge(doc.status)}
              </div>

              <h3 className="mb-1 truncate text-sm font-medium text-card-foreground">
                {doc.filename}
              </h3>

              <div className="flex items-center gap-3 text-xs text-muted-foreground">
                <span>{doc.word_count.toLocaleString()} words</span>
                <span>{doc.chunk_count} chunks</span>
              </div>

              <button
                onClick={(e) => {
                  e.stopPropagation();
                  setDeleteConfirmId(doc.id);
                }}
                aria-label={`Delete ${doc.filename}`}
                className="absolute right-2 top-2 hidden rounded p-1 text-muted-foreground transition-colors hover:bg-destructive/10 hover:text-destructive group-hover:block"
              >
                <Trash2 size={14} />
              </button>
            </div>
          ))}
        </div>
      </div>

      {/* Delete Confirmation Modal */}
      {deleteConfirmId && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-80 rounded-lg border border-border bg-background p-5 shadow-lg">
            <h3 className="mb-2 text-sm font-semibold text-foreground">
              Delete Document
            </h3>
            <p className="mb-4 text-sm text-muted-foreground">
              This will permanently delete this document and all its chunks. This action cannot be undone.
            </p>
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setDeleteConfirmId(null)}
                className="rounded-md px-3 py-1.5 text-sm text-muted-foreground hover:bg-muted"
              >
                Cancel
              </button>
              <button
                onClick={() => handleDeleteDocument(deleteConfirmId)}
                className="rounded-md bg-destructive px-3 py-1.5 text-sm text-white hover:bg-destructive/90"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
