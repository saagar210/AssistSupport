import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Search,
  FileText,
  Filter,
  Clock,
  X,
  Sparkles,
  AlertCircle,
  RefreshCw,
} from "lucide-react";
import { useCollectionStore } from "../stores/collectionStore";
import { useAppStore } from "../stores/appStore";
import { getFileTypeBadgeColor } from "../utils/fileTypeColors";
import { ContextualHelp } from "../components/ContextualHelp";
import { SearchSkeleton } from "../components/LoadingSkeleton";
import { EmptyState } from "../components/EmptyState";
import type { SearchResult, SearchHistoryEntry } from "../types";

type SearchMode = "hybrid" | "semantic" | "keyword";

const SEARCH_COMMANDS: Record<SearchMode, string> = {
  hybrid: "hybrid_search",
  semantic: "vector_search",
  keyword: "keyword_search",
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

function getDocFileType(title: string): string {
  const ext = title.split(".").pop()?.toLowerCase() ?? "";
  return ext;
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
  const [searchError, setSearchError] = useState<string | null>(null);

  // Facets
  const [showFilters, setShowFilters] = useState(false);
  const [typeFilters, setTypeFilters] = useState<Set<string>>(new Set());

  // History
  const [history, setHistory] = useState<SearchHistoryEntry[]>([]);

  // Load search history on mount
  useEffect(() => {
    if (!activeCollectionId) return;
    invoke<SearchHistoryEntry[]>("get_search_history", {
      collectionId: activeCollectionId,
      limit: 5,
    })
      .then(setHistory)
      .catch(() => {});
  }, [activeCollectionId]);

  const handleSearch = useCallback(async (searchQuery?: string) => {
    const q = searchQuery ?? query;
    if (!activeCollectionId || !q.trim()) return;

    setLoading(true);
    setSearched(true);
    setSearchError(null);
    if (searchQuery) setQuery(searchQuery);
    try {
      const command = SEARCH_COMMANDS[mode];
      const searchResults = await invoke<SearchResult[]>(command, {
        collectionId: activeCollectionId,
        query: q.trim(),
      });
      setResults(searchResults);

      // Save to history
      invoke("save_search_query", {
        collectionId: activeCollectionId,
        query: q.trim(),
        resultCount: searchResults.length,
      }).catch(() => {});

      // Refresh history
      invoke<SearchHistoryEntry[]>("get_search_history", {
        collectionId: activeCollectionId,
        limit: 5,
      })
        .then(setHistory)
        .catch(() => {});
    } catch (err) {
      console.error("Search failed:", err);
      setSearchError("Search failed: " + String(err));
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

  const handleMoreLikeThis = useCallback(
    async (chunkId: string) => {
      if (!activeCollectionId) return;
      setLoading(true);
      setSearched(true);
      try {
        const similar = await invoke<SearchResult[]>("find_similar_chunks", {
          chunkId,
          collectionId: activeCollectionId,
          topK: 10,
        });
        setResults(similar);
        setQuery("(similar to selected)");
      } catch (error) {
        console.error("Find similar failed:", error);
      } finally {
        setLoading(false);
      }
    },
    [activeCollectionId],
  );

  const handleClearHistory = useCallback(async () => {
    if (!activeCollectionId) return;
    await invoke("clear_search_history", { collectionId: activeCollectionId }).catch(() => {});
    setHistory([]);
  }, [activeCollectionId]);

  const toggleTypeFilter = useCallback((fileType: string) => {
    setTypeFilters((prev) => {
      const next = new Set(prev);
      if (next.has(fileType)) {
        next.delete(fileType);
      } else {
        next.add(fileType);
      }
      return next;
    });
  }, []);

  // Get unique file types from results for facet filtering
  const fileTypes = Array.from(
    new Set(results.map((r) => getDocFileType(r.document_title)).filter(Boolean))
  );

  // Apply client-side filters
  const filteredResults = typeFilters.size > 0
    ? results.filter((r) => typeFilters.has(getDocFileType(r.document_title)))
    : results;

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

          <div className="mt-3 flex items-center justify-between">
            <div className="flex items-center gap-4">
              <ContextualHelp topic="search" />
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

            {searched && results.length > 0 && (
              <button
                onClick={() => setShowFilters(!showFilters)}
                className={`flex items-center gap-1 rounded-md px-2 py-1 text-xs transition-colors ${
                  showFilters || typeFilters.size > 0
                    ? "bg-accent/10 text-accent"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                <Filter size={12} />
                Filters
                {typeFilters.size > 0 && (
                  <span className="ml-1 rounded-full bg-accent px-1.5 text-[10px] text-accent-foreground">
                    {typeFilters.size}
                  </span>
                )}
              </button>
            )}
          </div>

          {/* Search History Chips */}
          {!searched && history.length > 0 && (
            <div className="mt-3 flex items-center gap-2">
              <Clock size={12} className="text-muted-foreground" />
              {history.map((h) => (
                <button
                  key={h.id}
                  onClick={() => handleSearch(h.query)}
                  className="rounded-full border border-border px-2.5 py-0.5 text-xs text-muted-foreground transition-colors hover:border-accent/50 hover:text-foreground"
                >
                  {h.query}
                </button>
              ))}
              <button
                onClick={handleClearHistory}
                className="ml-1 text-[10px] text-muted-foreground hover:text-foreground"
              >
                Clear
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Filter Panel */}
      {showFilters && fileTypes.length > 0 && (
        <div className="border-b border-border bg-muted/30 px-4 py-2">
          <div className="mx-auto flex max-w-3xl items-center gap-3">
            <span className="text-xs text-muted-foreground">File type:</span>
            {fileTypes.map((ft) => {
              const colorClass = getFileTypeBadgeColor(ft);
              const active = typeFilters.has(ft);
              return (
                <button
                  key={ft}
                  onClick={() => toggleTypeFilter(ft)}
                  className={`flex items-center gap-1 rounded-md px-2 py-0.5 text-xs transition-colors ${
                    active
                      ? "bg-accent/10 text-accent ring-1 ring-accent/30"
                      : "text-muted-foreground hover:text-foreground"
                  }`}
                >
                  <span className={`inline-block h-2 w-2 rounded-full ${colorClass}`} />
                  {ft.toUpperCase()}
                </button>
              );
            })}
            {typeFilters.size > 0 && (
              <button
                onClick={() => setTypeFilters(new Set())}
                className="flex items-center gap-0.5 text-[10px] text-muted-foreground hover:text-foreground"
              >
                <X size={10} />
                Clear
              </button>
            )}
          </div>
        </div>
      )}

      {/* Results */}
      <div className="flex-1 overflow-y-auto p-4 scrollbar-thin">
        <div className="mx-auto max-w-3xl">
          <div aria-live="polite" className="sr-only">
            {loading && "Searching..."}
            {!loading && searched && filteredResults.length === 0 && !searchError && `No results found for ${query}`}
            {!loading && searched && filteredResults.length > 0 && `${filteredResults.length} results found`}
            {searchError ?? ""}
          </div>
          {searchError ? (
            <div className="flex flex-col items-center gap-3 py-12 text-muted-foreground">
              <AlertCircle size={32} strokeWidth={1.5} className="text-destructive" />
              <p className="text-sm text-destructive">{searchError}</p>
              <button
                onClick={() => handleSearch()}
                className="flex items-center gap-2 rounded-lg bg-accent px-4 py-2 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90"
              >
                <RefreshCw size={16} />
                Retry
              </button>
            </div>
          ) : loading ? (
            <SearchSkeleton />
          ) : searched && filteredResults.length === 0 ? (
            <div className="flex flex-col items-center gap-3 py-12 text-muted-foreground">
              <Search size={32} strokeWidth={1.5} />
              <p className="text-sm">No results found for &quot;{query}&quot;</p>
              <p className="text-xs">Try different keywords or switch search mode</p>
            </div>
          ) : !searched ? (
            <EmptyState
              icon={Search}
              title="Search your knowledge"
              description="Find anything by meaning, not just keywords. Use hybrid mode for best results."
            />
          ) : (
            <div className="space-y-4">
              <p className="mb-4 text-xs text-muted-foreground">
                {filteredResults.length} result{filteredResults.length !== 1 ? "s" : ""} found
                {typeFilters.size > 0 && ` (filtered from ${results.length})`}
              </p>
              {filteredResults.map((result) => {
                const ft = getDocFileType(result.document_title);
                const colorClass = getFileTypeBadgeColor(ft);
                const scorePercent = Math.round(result.score * 100);

                return (
                  <div
                    key={result.chunk_id}
                    className="rounded-lg border border-border bg-card p-4 transition-all duration-150 hover:border-accent/50 hover:shadow-md"
                  >
                    <div
                      onClick={() => handleResultClick(result)}
                      className="cursor-pointer"
                    >
                      <div className="mb-2 flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <FileText size={14} className="text-muted-foreground" />
                          <span className="min-w-0 truncate text-sm font-medium text-card-foreground">
                            {result.document_title}
                          </span>
                          <span
                            className={`inline-flex items-center rounded px-1 py-0.5 text-[9px] font-semibold uppercase text-white ${colorClass}`}
                          >
                            {ft}
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

                    <div className="mt-2 flex items-center border-t border-border pt-2">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleMoreLikeThis(result.chunk_id);
                        }}
                        className="flex items-center gap-1 rounded px-2 py-0.5 text-[10px] text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                      >
                        <Sparkles size={10} />
                        More like this
                      </button>
                    </div>
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
