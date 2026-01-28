import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '../shared/Button';
import { Skeleton } from '../shared/Skeleton';
import { useKb } from '../../hooks/useKb';
import { useToastContext } from '../../contexts/ToastContext';
import type { IndexedFile, SearchResult, Namespace } from '../../types';
import './SourcesTab.css';

type SearchMode = 'files' | 'content';

interface SourcesTabProps {
  initialSearchQuery?: string | null;
  onSearchQueryConsumed?: () => void;
}

export function SourcesTab({ initialSearchQuery, onSearchQueryConsumed }: SourcesTabProps = {}) {
  const { getKbFolder, listFiles, rebuildIndex, getIndexStats, search, removeDocument } = useKb();
  const { success: showSuccess, error: showError } = useToastContext();

  const [kbFolder, setKbFolder] = useState<string | null>(null);
  const [files, setFiles] = useState<IndexedFile[]>([]);
  const [stats, setStats] = useState<{ total_chunks: number; total_files: number } | null>(null);
  const [loading, setLoading] = useState(false);
  const [rebuilding, setRebuilding] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchMode, setSearchMode] = useState<SearchMode>('files');
  const [contentResults, setContentResults] = useState<SearchResult[]>([]);
  const [searching, setSearching] = useState(false);
  const [removeConfirm, setRemoveConfirm] = useState<string | null>(null);
  const [removing, setRemoving] = useState(false);
  const [namespaces, setNamespaces] = useState<Namespace[]>([]);
  const [filterNamespace, setFilterNamespace] = useState<string>('');
  const [filterSourceType, setFilterSourceType] = useState<string>('');

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [folder, fileList, indexStats, nsList] = await Promise.all([
        getKbFolder(),
        listFiles().catch(() => []),
        getIndexStats().catch(() => null),
        invoke<Namespace[]>('list_namespaces').catch(() => []),
      ]);
      setKbFolder(folder);
      setFiles(fileList);
      setStats(indexStats);
      setNamespaces(nsList);
    } catch (err) {
      setError(`Failed to load data: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [getKbFolder, listFiles, getIndexStats]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  // Handle navigation from KB suggestion chips
  const [highlightResults, setHighlightResults] = useState(false);
  useEffect(() => {
    if (initialSearchQuery) {
      setSearchQuery(initialSearchQuery);
      setSearchMode('content');
      setHighlightResults(true);
      onSearchQueryConsumed?.();
      // Clear highlight after animation
      const timer = setTimeout(() => setHighlightResults(false), 2000);
      return () => clearTimeout(timer);
    }
  }, [initialSearchQuery, onSearchQueryConsumed]);

  async function handleRebuild() {
    setRebuilding(true);
    setError(null);
    try {
      await rebuildIndex();
      await loadData();
    } catch (err) {
      setError(`Failed to rebuild index: ${err}`);
    } finally {
      setRebuilding(false);
    }
  }

  const handleRemoveFile = useCallback(async (filePath: string) => {
    setRemoving(true);
    try {
      const removed = await removeDocument(filePath);
      if (removed) {
        showSuccess('File removed from knowledge base');
        await loadData();
      }
    } catch (err) {
      showError(`Failed to remove file: ${err}`);
    } finally {
      setRemoving(false);
      setRemoveConfirm(null);
    }
  }, [removeDocument, loadData, showSuccess, showError]);

  const filteredFiles = files.filter(file =>
    file.file_path.toLowerCase().includes(searchQuery.toLowerCase()) ||
    (file.title && file.title.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  const handleContentSearch = useCallback(async (query: string) => {
    if (query.length < 3) {
      setContentResults([]);
      return;
    }
    setSearching(true);
    try {
      const nsFilter = filterNamespace || undefined;
      const results = await search(query, 20, nsFilter);
      // Client-side filter by source type if set
      const filtered = filterSourceType
        ? results.filter(r => r.source_type === filterSourceType)
        : results;
      setContentResults(filtered);
    } catch (err) {
      setError(`Search failed: ${err}`);
    } finally {
      setSearching(false);
    }
  }, [search, filterNamespace, filterSourceType]);

  // Debounced content search
  useEffect(() => {
    if (searchMode !== 'content') return;

    const timeout = setTimeout(() => {
      handleContentSearch(searchQuery);
    }, 300);

    return () => clearTimeout(timeout);
  }, [searchQuery, searchMode, handleContentSearch, filterNamespace, filterSourceType]);

  function formatDate(isoString: string): string {
    return new Date(isoString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  }

  function formatPath(path: string): string {
    if (!kbFolder) return path;
    return path.replace(kbFolder, '').replace(/^\//, '');
  }

  if (!kbFolder) {
    return (
      <div className="sources-tab">
        <div className="sources-empty">
          <h2>No Knowledge Base Configured</h2>
          <p>Go to Settings to select a folder for your knowledge base.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="sources-tab">
      <header className="sources-header">
        <div className="sources-title">
          <h2>Knowledge Base</h2>
          <code className="kb-path">{kbFolder}</code>
        </div>
        <div className="sources-actions">
          <Button
            variant="secondary"
            onClick={handleRebuild}
            disabled={rebuilding}
          >
            {rebuilding ? 'Rebuilding...' : 'Rebuild Index'}
          </Button>
          <Button variant="ghost" onClick={loadData} disabled={loading}>
            Refresh
          </Button>
        </div>
      </header>

      {error && (
        <div className="sources-error">
          <span>{error}</span>
          <Button variant="ghost" size="small" onClick={loadData}>
            Retry
          </Button>
        </div>
      )}

      <div className="sources-stats">
        <div className="stat-card">
          <span className="stat-number">{stats?.total_files ?? 0}</span>
          <span className="stat-label">Files</span>
        </div>
        <div className="stat-card">
          <span className="stat-number">{stats?.total_chunks ?? 0}</span>
          <span className="stat-label">Chunks</span>
        </div>
        <div className="stat-card">
          <span className="stat-number">
            {stats?.total_chunks && stats?.total_files
              ? Math.round(stats.total_chunks / stats.total_files)
              : 0}
          </span>
          <span className="stat-label">Avg Chunks/File</span>
        </div>
      </div>

      <div className="sources-search">
        <div className="search-mode-toggle">
          <button
            className={`toggle-btn ${searchMode === 'files' ? 'active' : ''}`}
            onClick={() => {
              setSearchMode('files');
              setContentResults([]);
            }}
          >
            Filter Files
          </button>
          <button
            className={`toggle-btn ${searchMode === 'content' ? 'active' : ''}`}
            onClick={() => setSearchMode('content')}
          >
            Search Content
          </button>
        </div>
        <input
          type="text"
          placeholder={searchMode === 'files' ? 'Filter files...' : 'Search content...'}
          value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)}
          className="search-input"
        />
        {searchMode === 'content' && (
          <div className="sources-filters">
            <select
              value={filterNamespace}
              onChange={e => setFilterNamespace(e.target.value)}
              className="filter-select"
            >
              <option value="">All Namespaces</option>
              {namespaces.map(ns => (
                <option key={ns.id} value={ns.id}>{ns.name}</option>
              ))}
            </select>
            <select
              value={filterSourceType}
              onChange={e => setFilterSourceType(e.target.value)}
              className="filter-select"
            >
              <option value="">All Types</option>
              <option value="file">File</option>
              <option value="url">URL</option>
              <option value="youtube">YouTube</option>
              <option value="github">GitHub</option>
            </select>
          </div>
        )}
      </div>

      {searchMode === 'content' ? (
        // Content search results
        searching ? (
          <div className="sources-empty-list">Searching...</div>
        ) : searchQuery.length < 3 ? (
          <div className="sources-empty-list">Type at least 3 characters to search content.</div>
        ) : contentResults.length === 0 ? (
          <div className="sources-empty-list">No content matches your search.</div>
        ) : (
          <div className="content-results">
            {contentResults.map((result, index) => (
              <div key={result.chunk_id} className={`content-result-item${highlightResults && index === 0 ? ' highlighted' : ''}`}>
                <div className="result-header">
                  <span className="result-title">{result.title || formatPath(result.file_path)}</span>
                  <div className="result-header-right">
                    {result.source_type && (
                      <span className="result-badge result-badge-type">{result.source_type}</span>
                    )}
                    {result.namespace_id && namespaces.length > 0 && (
                      <span className="result-badge result-badge-ns">
                        {namespaces.find(ns => ns.id === result.namespace_id)?.name ?? result.namespace_id}
                      </span>
                    )}
                    <span className="result-score">{Math.round(result.score * 100)}%</span>
                  </div>
                </div>
                {result.heading_path && (
                  <span className="result-heading">{result.heading_path}</span>
                )}
                <p className="result-snippet">{result.snippet}</p>
                <span className="result-path">{formatPath(result.file_path)}</span>
              </div>
            ))}
          </div>
        )
      ) : loading ? (
        <div className="file-list">
          <div className="file-list-header">
            <span className="col-name">File</span>
            <span className="col-chunks">Chunks</span>
            <span className="col-date">Indexed</span>
          </div>
          {Array.from({ length: 5 }).map((_, i) => (
            <div key={i} className="skeleton-file-item">
              <div className="col-name">
                <Skeleton width="70%" height="1em" />
                <Skeleton width="50%" height="0.85em" />
              </div>
              <Skeleton width="40px" height="1em" />
              <Skeleton width="80px" height="1em" />
            </div>
          ))}
        </div>
      ) : filteredFiles.length === 0 ? (
        <div className="sources-empty-list">
          {searchQuery ? 'No files match your search.' : 'No files indexed yet.'}
        </div>
      ) : (
        <div className="file-list">
          <div className="file-list-header">
            <span className="col-name">File</span>
            <span className="col-chunks">Chunks</span>
            <span className="col-date">Indexed</span>
            <span className="col-actions"></span>
          </div>
          {filteredFiles.map(file => (
            <div key={file.file_path} className="file-item">
              <div className="col-name">
                <span className="file-title">{file.title || formatPath(file.file_path)}</span>
                {file.title && (
                  <span className="file-path">{formatPath(file.file_path)}</span>
                )}
              </div>
              <span className="col-chunks">{file.chunk_count}</span>
              <span className="col-date">{formatDate(file.indexed_at)}</span>
              <div className="col-actions">
                <Button
                  variant="ghost"
                  size="small"
                  onClick={() => setRemoveConfirm(file.file_path)}
                  title="Remove from index"
                >
                  Remove
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Remove Confirmation Modal */}
      {removeConfirm && (
        <div className="modal-overlay" onClick={() => setRemoveConfirm(null)}>
          <div className="modal-content modal-confirm" onClick={(e) => e.stopPropagation()}>
            <h3>Remove from Knowledge Base</h3>
            <p>
              Are you sure you want to remove this file from the index? The original file will not be deleted.
            </p>
            <p className="modal-file-path">{formatPath(removeConfirm)}</p>
            <div className="modal-actions">
              <Button variant="ghost" onClick={() => setRemoveConfirm(null)} disabled={removing}>
                Cancel
              </Button>
              <Button
                variant="primary"
                onClick={() => handleRemoveFile(removeConfirm)}
                disabled={removing}
              >
                {removing ? 'Removing...' : 'Remove'}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
