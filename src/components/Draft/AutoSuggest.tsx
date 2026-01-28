import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { SearchResult } from '../../types';
import './AutoSuggest.css';

interface AutoSuggestProps {
  query: string;
  onSelectSuggestion?: (title: string, content: string) => void;
}

export function AutoSuggest({ query, onSelectSuggestion }: AutoSuggestProps) {
  const [suggestions, setSuggestions] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [enabled, setEnabled] = useState<boolean>(() => {
    const stored = localStorage.getItem('auto-suggest-enabled');
    return stored === null ? true : stored === 'true';
  });
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastQueryRef = useRef<string>('');

  const handleToggle = useCallback(() => {
    setEnabled(prev => {
      const next = !prev;
      localStorage.setItem('auto-suggest-enabled', String(next));
      if (!next) {
        setSuggestions([]);
      }
      return next;
    });
  }, []);

  useEffect(() => {
    if (!enabled) {
      setSuggestions([]);
      return;
    }

    const trimmedQuery = query.trim();

    if (!trimmedQuery) {
      setSuggestions([]);
      lastQueryRef.current = '';
      return;
    }

    // Clear existing debounce
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    debounceRef.current = setTimeout(async () => {
      // Avoid duplicate searches for the same query
      if (trimmedQuery === lastQueryRef.current) return;
      lastQueryRef.current = trimmedQuery;

      setLoading(true);
      try {
        const results = await invoke<SearchResult[]>('search_kb', {
          query: trimmedQuery,
          limit: 5,
          namespaceId: null,
        });
        setSuggestions(results);
      } catch (err) {
        console.error('AutoSuggest search failed:', err);
        setSuggestions([]);
      } finally {
        setLoading(false);
      }
    }, 500);

    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, [query, enabled]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

  const handleChipClick = useCallback(
    (result: SearchResult) => {
      if (onSelectSuggestion) {
        onSelectSuggestion(result.title || result.file_path, result.content);
      }
    },
    [onSelectSuggestion]
  );

  const trimmedQuery = query.trim();
  const showNoResults = enabled && trimmedQuery.length > 20 && suggestions.length === 0 && !loading;

  return (
    <div className="auto-suggest">
      <div className="auto-suggest-header">
        <span className="auto-suggest-label">KB Suggestions</span>
        <button
          className={`auto-suggest-toggle ${enabled ? 'active' : ''}`}
          onClick={handleToggle}
          aria-label={enabled ? 'Disable auto-suggest' : 'Enable auto-suggest'}
          title={enabled ? 'Disable auto-suggest' : 'Enable auto-suggest'}
        >
          <span className="auto-suggest-toggle-track">
            <span className="auto-suggest-toggle-thumb" />
          </span>
        </button>
      </div>

      {enabled && loading && (
        <div className="auto-suggest-loading">Searching...</div>
      )}

      {enabled && suggestions.length > 0 && (
        <div className="auto-suggest-chips">
          {suggestions.map(result => (
            <button
              key={result.chunk_id}
              className="auto-suggest-chip"
              onClick={() => handleChipClick(result)}
              title={result.snippet}
            >
              {result.title || result.file_path}
            </button>
          ))}
        </div>
      )}

      {showNoResults && (
        <div className="auto-suggest-empty">No matching articles</div>
      )}
    </div>
  );
}
