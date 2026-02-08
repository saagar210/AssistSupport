import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Search,
  FileText,
  Lightbulb,
} from "lucide-react";
import { useCollectionStore } from "../stores/collectionStore";
import { useAppStore } from "../stores/appStore";
import type { SearchResult } from "../types";

type SearchMode = "hybrid" | "semantic" | "keyword";

const SEARCH_COMMANDS: Record<SearchMode, string> = {
  hybrid: "hybrid_search",
  semantic: "vector_search",
  keyword: "keyword_search",
};

const FILE_TYPE_COLORS: Record<string, string> = {
  pdf: "bg-blue-500",
  md: "bg-green-500",
  docx: "bg-orange-500",
  txt: "bg-slate-400",
  html: "bg-purple-500",
  csv: "bg-yellow-500",
  epub: "bg-red-500",
};

function highlightText(text: string, query: string): React.ReactNode[] {
  if (!query.trim()) return [text];

  const words = query.trim().split(/\s+/).filter(Boolean);
  const escaped = words.map((w) => w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  const pattern = new RegExp(`(${escaped.join("|")})`, "gi");
  const parts = text.split(pattern);

  return parts.map((part, i) => {
    const isMatch = words.some(
      (w) => part.toLowerCase() === w.toLowerCase(),
    );
    if (isMatch) {
      return (
        <mark key={i} className="rounded bg-accent/20 px-0.5 text-foreground">
          {part}
        </mark>
      );
    }
    return <span key={i}>{part}</span>;
  });
}

function SkeletonCard() {
  return (
    <div className="animate-pulse rounded-lg border border-border bg-card p-4">
      <div className="mb-2 flex items-center gap-2">
        <div className="h-4 w-24 rounded bg-muted" />
        <div className="h-4 w-12 rounded bg-muted" />
      </div>
      <div className="mb-2 h-3 w-3/4 rounded bg-muted" />
      <div className="space-y-1.5">
        <div className="h-3 w-full rounded bg-muted" />
        <div className="h-3 w-5/6 rounded bg-muted" />
        <div className="h-3 w-2/3 rounded bg-muted" />
      </div>
    </div>
  );
}

export function SearchView() {
  const activeCollectionId = useCollectionStore((s) => s.activeCollectionId);
  const setActiveView = useAppStore((s) => s.setActiveView);
  const setSelectedDocument = useAppStore((s) => s.setSelectedDocument);

  const [query, setQuery] = useState("");
  const [mode, setMode] = useState<SearchMode>("hybrid");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [searched, setSearched] = useState(false);
  const [loading, setLoading] = useState(false);

  const handleSearch = useCallback(async () => {
    if (!activeCollectionId || !query.trim()) return;

    setLoading(true);
    setSearched(true);
    try {
      const command = SEARCH_COMMANDS[mode];
      const searchResults = await invoke<SearchResult[]>(command, {
        collectionId: activeCollectionId,
        query: query.trim(),
      });
      setResults(searchResults);
    } catch (error) {
      console.error("Search failed:", error);
      setResults([]);
    } finally {
      setLoading(false);
    }
  }, [activeCollectionId, query, mode]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") {
        handleSearch();
      }
    },
    [handleSearch],
  );

  const handleResultClick = useCallback(
    (result: SearchResult) => {
      setSelectedDocument(result.document_id);
      setActiveView("document-detail");
    },
    [setSelectedDocument, setActiveView],
  );

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Search Header */}
      <div className="border-b border-border px-4 py-4">
        <div className="mx-auto max-w-3xl">
          <div className="relative">
            <Search
              size={18}
              className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
            />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Search your knowledge base..."
              className="h-11 w-full rounded-lg border border-border bg-background pl-10 pr-4 text-sm text-foreground outline-none transition-colors focus:border-accent focus:ring-1 focus:ring-accent"
              autoFocus
            />
          </div>

          <div className="mt-3 flex items-center gap-4">
            <span className="text-xs text-muted-foreground">Mode:</span>
            {(["hybrid", "semantic", "keyword"] as const).map((m) => (
              <label
                key={m}
                className={`flex cursor-pointer items-center gap-1.5 rounded-md px-2.5 py-1 text-xs transition-colors ${
                  mode === m
                    ? "bg-accent/10 text-accent"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                <input
                  type="radio"
                  name="searchMode"
                  value={m}
                  checked={mode === m}
                  onChange={() => setMode(m)}
                  className="sr-only"
                />
                {m.charAt(0).toUpperCase() + m.slice(1)}
              </label>
            ))}
          </div>
        </div>
      </div>

      {/* Results */}
      <div className="flex-1 overflow-y-auto p-4">
        <div className="mx-auto max-w-3xl">
          {loading ? (
            <div className="space-y-3">
              <SkeletonCard />
              <SkeletonCard />
              <SkeletonCard />
            </div>
          ) : searched && results.length === 0 ? (
            <div className="flex flex-col items-center gap-3 py-12 text-muted-foreground">
              <Search size={32} strokeWidth={1.5} />
              <p className="text-sm">No results found for &quot;{query}&quot;</p>
              <p className="text-xs">Try different keywords or switch search mode</p>
            </div>
          ) : !searched ? (
            <div className="flex flex-col items-center gap-4 py-12 text-muted-foreground">
              <Lightbulb size={32} strokeWidth={1.5} />
              <h3 className="text-sm font-medium text-foreground">Search Tips</h3>
              <ul className="space-y-1.5 text-xs">
                <li>Use natural language for semantic search</li>
                <li>Use specific terms for keyword search</li>
                <li>Hybrid mode combines both for best results</li>
                <li>Press Enter to search</li>
              </ul>
            </div>
          ) : (
            <div className="space-y-3">
              <p className="mb-4 text-xs text-muted-foreground">
                {results.length} result{results.length !== 1 ? "s" : ""} found
              </p>
              {results.map((result) => {
                const colorClass =
                  FILE_TYPE_COLORS[
                    result.document_title.split(".").pop()?.toLowerCase() ?? ""
                  ] ?? "bg-slate-400";
                const scorePercent = Math.round(result.score * 100);

                return (
                  <div
                    key={result.chunk_id}
                    onClick={() => handleResultClick(result)}
                    className="cursor-pointer rounded-lg border border-border bg-card p-4 transition-all hover:border-accent/50 hover:shadow-sm"
                  >
                    <div className="mb-2 flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <FileText size={14} className="text-muted-foreground" />
                        <span className="text-sm font-medium text-card-foreground">
                          {result.document_title}
                        </span>
                        <span
                          className={`inline-flex items-center rounded px-1 py-0.5 text-[9px] font-semibold uppercase text-white ${colorClass}`}
                        >
                          {result.document_title.split(".").pop()}
                        </span>
                      </div>
                      <span className="rounded-full bg-accent/10 px-2 py-0.5 text-[10px] font-medium text-accent">
                        {scorePercent}%
                      </span>
                    </div>

                    {result.section_title && (
                      <p className="mb-1 text-xs text-accent">
                        {result.section_title}
                      </p>
                    )}

                    <p className="text-sm leading-relaxed text-muted-foreground">
                      {highlightText(
                        result.content.length > 300
                          ? result.content.slice(0, 300) + "..."
                          : result.content,
                        query,
                      )}
                    </p>

                    {result.page_number && (
                      <p className="mt-2 text-[10px] text-muted-foreground">
                        Page {result.page_number}
                      </p>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
