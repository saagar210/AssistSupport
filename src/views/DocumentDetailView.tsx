import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  ArrowLeft,
  Trash2,
  FileText,
  Hash,
  Calendar,
  Loader2,
  ExternalLink,
  RotateCw,
  Sparkles,
} from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { useCollectionStore } from "../stores/collectionStore";
import { useDocumentStore } from "../stores/documentStore";
import { useToastStore } from "../stores/toastStore";
import { getFileTypeBadgeColor } from "../utils/fileTypeColors";
import type { Document, Chunk } from "../types";

export function DocumentDetailView() {
  const selectedDocumentId = useAppStore((s) => s.selectedDocumentId);
  const setActiveView = useAppStore((s) => s.setActiveView);
  const setSelectedDocument = useAppStore((s) => s.setSelectedDocument);
  const activeCollectionId = useCollectionStore((s) => s.activeCollectionId);
  const deleteDocument = useDocumentStore((s) => s.deleteDocument);
  const addToast = useToastStore((s) => s.addToast);

  const [document, setDocument] = useState<Document | null>(null);
  const [chunks, setChunks] = useState<Chunk[]>([]);
  const [loading, setLoading] = useState(true);
  const [deleteConfirm, setDeleteConfirm] = useState(false);
  const [activeSectionIndex, setActiveSectionIndex] = useState<number | null>(null);
  const chunkRefs = useRef<Map<number, HTMLDivElement>>(new Map());

  useEffect(() => {
    if (!selectedDocumentId) return;

    async function loadDocument() {
      setLoading(true);
      try {
        const doc = await invoke<Document>("get_document", {
          id: selectedDocumentId,
        });
        setDocument(doc);

        const docChunks = await invoke<Chunk[]>("get_document_chunks", {
          documentId: selectedDocumentId,
        });
        setChunks(docChunks);
      } catch (error) {
        console.error("Failed to load document:", error);
        addToast("error", "Failed to load document details");
      } finally {
        setLoading(false);
      }
    }

    loadDocument();
  }, [selectedDocumentId, addToast]);

  const [reingesting, setReingesting] = useState(false);

  const handleReingest = useCallback(async () => {
    if (!selectedDocumentId) return;
    setReingesting(true);
    try {
      await invoke("reingest_document", { documentId: selectedDocumentId });
      addToast("success", "Re-ingestion started");
    } catch (error) {
      addToast("error", "Failed to start re-ingestion: " + String(error));
    } finally {
      setReingesting(false);
    }
  }, [selectedDocumentId, addToast]);

  const handleFindSimilar = useCallback(async () => {
    if (!chunks.length || !activeCollectionId) return;
    // Use first chunk's ID to find similar documents
    try {
      const results = await invoke("find_similar_chunks", {
        chunkId: chunks[0].id,
        collectionId: activeCollectionId,
        topK: 10,
      });
      if (results) {
        setActiveView("search");
      }
    } catch (error) {
      addToast("error", "Failed to find similar: " + String(error));
    }
  }, [chunks, activeCollectionId, setActiveView, addToast]);

  const handleBack = useCallback(() => {
    setSelectedDocument(null);
    setActiveView("documents");
  }, [setSelectedDocument, setActiveView]);

  const handleDelete = useCallback(async () => {
    if (!selectedDocumentId || !activeCollectionId) return;
    await deleteDocument(selectedDocumentId, activeCollectionId);
    addToast("success", "Document deleted");
    handleBack();
  }, [selectedDocumentId, activeCollectionId, deleteDocument, addToast, handleBack]);

  const scrollToChunk = useCallback((index: number) => {
    const el = chunkRefs.current.get(index);
    if (el) {
      el.scrollIntoView({ behavior: "smooth", block: "start" });
      setActiveSectionIndex(index);
    }
  }, []);

  const setChunkRef = useCallback((index: number, el: HTMLDivElement | null) => {
    if (el) {
      chunkRefs.current.set(index, el);
    } else {
      chunkRefs.current.delete(index);
    }
  }, []);

  if (loading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <Loader2 size={32} className="animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!document) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-4 text-muted-foreground">
        <FileText size={48} strokeWidth={1.5} />
        <p className="text-sm">Document not found</p>
        <button
          onClick={handleBack}
          className="text-sm text-accent hover:underline"
        >
          Back to documents
        </button>
      </div>
    );
  }

  const sections = chunks
    .filter((c) => c.section_title)
    .reduce<Array<{ title: string; index: number }>>((acc, chunk) => {
      if (chunk.section_title && !acc.some((s) => s.title === chunk.section_title)) {
        acc.push({ title: chunk.section_title, index: chunk.chunk_index });
      }
      return acc;
    }, []);

  const colorClass = getFileTypeBadgeColor(document.file_type);

  return (
    <div className="flex flex-1 overflow-hidden">
      {/* Section Navigation Sidebar */}
      {sections.length > 0 && (
        <div className="w-56 shrink-0 overflow-y-auto border-r border-border bg-muted/30 p-3">
          <h3 className="mb-2 text-xs font-semibold uppercase text-muted-foreground">
            Sections
          </h3>
          <nav className="space-y-0.5">
            {sections.map((section) => (
              <button
                key={section.index}
                onClick={() => scrollToChunk(section.index)}
                className={`block w-full truncate rounded px-2 py-1.5 text-left text-xs transition-colors ${
                  activeSectionIndex === section.index
                    ? "bg-accent/10 text-accent"
                    : "text-muted-foreground hover:bg-muted hover:text-foreground"
                }`}
              >
                {section.title}
              </button>
            ))}
          </nav>
        </div>
      )}

      {/* Main Content */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Header */}
        <div className="flex items-center gap-3 border-b border-border px-4 py-3">
          <button
            onClick={handleBack}
            className="flex items-center gap-1 rounded p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <ArrowLeft size={16} />
          </button>

          <div className="flex-1">
            <div className="flex items-center gap-2">
              <h1 className="text-sm font-semibold text-foreground">
                {document.title || document.filename}
              </h1>
              <span
                className={`inline-flex items-center rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase text-white ${colorClass}`}
              >
                {document.file_type}
              </span>
            </div>
            <div className="mt-0.5 flex items-center gap-4 text-xs text-muted-foreground">
              <span className="flex items-center gap-1">
                <Hash size={10} />
                {document.word_count.toLocaleString()} words
              </span>
              <span className="flex items-center gap-1">
                <FileText size={10} />
                {document.chunk_count} chunks
              </span>
              <span className="flex items-center gap-1">
                <Calendar size={10} />
                {new Date(document.created_at).toLocaleDateString()}
              </span>
            </div>
          </div>

          <div className="flex items-center gap-1">
            <button
              onClick={() => {
                invoke("open_file", { path: document.file_path }).catch(() => {
                  addToast("error", "Could not open source file");
                });
              }}
              className="flex items-center gap-1 rounded px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
              title="Open source file"
            >
              <ExternalLink size={12} />
              Open
            </button>
            <button
              onClick={handleFindSimilar}
              disabled={chunks.length === 0}
              className="flex items-center gap-1 rounded px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:opacity-50"
              title="Find similar documents"
            >
              <Sparkles size={12} />
              Find Similar
            </button>
            <button
              onClick={handleReingest}
              disabled={reingesting}
              className="flex items-center gap-1 rounded px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:opacity-50"
              title="Re-ingest with current settings"
            >
              <RotateCw size={12} className={reingesting ? "animate-spin" : ""} />
              Re-ingest
            </button>
            <button
              onClick={() => setDeleteConfirm(true)}
              className="flex items-center gap-1 rounded px-2 py-1 text-xs text-destructive transition-colors hover:bg-destructive/10"
            >
              <Trash2 size={12} />
              Delete
            </button>
          </div>
        </div>

        {/* Document Metadata */}
        <div className="border-b border-border bg-muted/30 px-4 py-2">
          <div className="flex flex-wrap gap-4 text-xs text-muted-foreground">
            <span>
              File hash: <code className="text-foreground/70">{document.file_hash.slice(0, 16)}...</code>
            </span>
            <span>
              Size: {(document.file_size / 1024).toFixed(1)} KB
            </span>
            {document.author && <span>Author: {document.author}</span>}
            {document.page_count && <span>Pages: {document.page_count}</span>}
          </div>
        </div>

        {/* Chunks */}
        <div className="flex-1 overflow-y-auto p-4">
          {chunks.length === 0 ? (
            <p className="text-center text-sm text-muted-foreground">
              No chunks available
            </p>
          ) : (
            <div className="space-y-1">
              {chunks.map((chunk, idx) => (
                <div
                  key={chunk.id}
                  ref={(el) => setChunkRef(chunk.chunk_index, el)}
                  className={`rounded-lg border border-border p-4 ${
                    idx % 2 === 0 ? "bg-card" : "bg-background"
                  } ${activeSectionIndex === chunk.chunk_index ? "ring-2 ring-accent/30" : ""}`}
                >
                  <div className="mb-2 flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span className="text-[10px] font-mono text-muted-foreground">
                        Chunk #{chunk.chunk_index + 1}
                      </span>
                      {chunk.section_title && (
                        <span className="text-[10px] text-accent">
                          {chunk.section_title}
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-2 text-[10px] text-muted-foreground">
                      {chunk.page_number && <span>Page {chunk.page_number}</span>}
                      <span>{chunk.token_count} tokens</span>
                    </div>
                  </div>
                  <p className="whitespace-pre-wrap text-sm text-card-foreground leading-relaxed">
                    {chunk.content}
                  </p>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Delete Confirmation Modal */}
      {deleteConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-80 rounded-lg border border-border bg-background p-5 shadow-lg">
            <h3 className="mb-2 text-sm font-semibold text-foreground">
              Delete Document
            </h3>
            <p className="mb-4 text-sm text-muted-foreground">
              This will permanently delete &quot;{document.filename}&quot; and all its chunks.
            </p>
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setDeleteConfirm(false)}
                className="rounded-md px-3 py-1.5 text-sm text-muted-foreground hover:bg-muted"
              >
                Cancel
              </button>
              <button
                onClick={handleDelete}
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
