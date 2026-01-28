import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FeedbackForm } from './FeedbackForm';
import { Button } from '../shared/Button';
import type { SearchResult } from '../../types';
import './Pilot.css';

interface PilotQueryTesterProps {
  onQueryLogged?: () => void;
}

export function PilotQueryTester({ onQueryLogged }: PilotQueryTesterProps) {
  const [query, setQuery] = useState('');
  const [userId, setUserId] = useState(() => localStorage.getItem('pilot-user-id') || '');
  const [searching, setSearching] = useState(false);
  const [response, setResponse] = useState<string | null>(null);
  const [logId, setLogId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleSearch = useCallback(async () => {
    if (!query.trim() || !userId.trim()) return;
    setSearching(true);
    setError(null);
    setResponse(null);
    setLogId(null);

    // Persist user ID
    localStorage.setItem('pilot-user-id', userId.trim());

    try {
      // Run KB search
      const results = await invoke<SearchResult[]>('search_kb', {
        query: query.trim(),
        limit: 5,
        namespaceId: null,
      });

      // Format response from search results
      const responseText = results.length > 0
        ? results.map((r, i) =>
            `[${i + 1}] ${r.title || 'Untitled'} (${r.source || 'unknown'}, score: ${r.score.toFixed(2)})\n${r.snippet || r.content.substring(0, 200)}`
          ).join('\n\n')
        : 'No results found for this query.';

      setResponse(responseText);

      // Log the query for pilot tracking
      const id = await invoke<string>('log_pilot_query', {
        query: query.trim(),
        response: responseText,
        userId: userId.trim(),
      });
      setLogId(id);
    } catch (err) {
      console.error('Pilot query failed:', err);
      setError(typeof err === 'string' ? err : 'Search failed');
    } finally {
      setSearching(false);
    }
  }, [query, userId]);

  const handleFeedbackSubmitted = useCallback(() => {
    onQueryLogged?.();
    // Reset for next query
    setTimeout(() => {
      setQuery('');
      setResponse(null);
      setLogId(null);
    }, 1500);
  }, [onQueryLogged]);

  return (
    <div className="pilot-tester">
      <h3 className="pilot-section-title">Test a Query</h3>

      <div className="pilot-tester-inputs">
        <input
          type="text"
          value={userId}
          onChange={e => setUserId(e.target.value)}
          placeholder="Your name or email"
          className="pilot-user-input"
        />
        <div className="pilot-query-row">
          <input
            type="text"
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && handleSearch()}
            placeholder="e.g., Can I use a flash drive?"
            className="pilot-query-input"
            disabled={searching}
          />
          <Button
            variant="primary"
            onClick={handleSearch}
            disabled={searching || !query.trim() || !userId.trim()}
          >
            {searching ? 'Searching...' : 'Search'}
          </Button>
        </div>
      </div>

      {error && <div className="pilot-feedback-error">{error}</div>}

      {response && (
        <div className="pilot-response">
          <h4 className="pilot-response-title">Response</h4>
          <pre className="pilot-response-text">{response}</pre>
        </div>
      )}

      {logId && (
        <FeedbackForm
          logId={logId}
          userId={userId.trim()}
          query={query.trim()}
          onSubmitted={handleFeedbackSubmitted}
        />
      )}
    </div>
  );
}
