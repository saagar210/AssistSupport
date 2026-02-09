import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FeedbackForm } from './FeedbackForm';
import { Button } from '../shared/Button';
import type { SearchResult } from '../../types';
import './Pilot.css';

interface PilotQueryTesterProps {
  pilotLoggingEnabled: boolean;
  policy: { enabled: boolean; retention_days: number; max_rows: number } | null;
  onQueryLogged?: () => void;
}

const OPERATOR_ID_STORAGE_KEY = 'pilot-operator-id';
const LEGACY_USER_ID_STORAGE_KEY = 'pilot-user-id';

function generateOperatorId(): string {
  const uuid = globalThis.crypto?.randomUUID?.();
  if (uuid) return `op-${uuid}`.toLowerCase();
  return `op-${Math.random().toString(16).slice(2)}${Date.now().toString(16)}`.toLowerCase();
}

function isValidOperatorId(value: string): boolean {
  // Must match backend: lowercase letters/digits/hyphen, no leading/trailing hyphen,
  // no '.' or '@' to avoid emails.
  if (!value) return false;
  if (value.length > 64) return false;
  if (value.includes('@') || value.includes('.')) return false;
  if (value.startsWith('-') || value.endsWith('-')) return false;
  return /^[a-z0-9-]+$/.test(value);
}

function getOrInitOperatorId(): string {
  // Drop legacy "name/email" storage to avoid persisting PII across sessions.
  if (localStorage.getItem(LEGACY_USER_ID_STORAGE_KEY)) {
    localStorage.removeItem(LEGACY_USER_ID_STORAGE_KEY);
  }

  const existing = (localStorage.getItem(OPERATOR_ID_STORAGE_KEY) || '').trim().toLowerCase();
  if (isValidOperatorId(existing)) return existing;

  const fresh = generateOperatorId();
  localStorage.setItem(OPERATOR_ID_STORAGE_KEY, fresh);
  return fresh;
}

export function PilotQueryTester({ pilotLoggingEnabled, policy, onQueryLogged }: PilotQueryTesterProps) {
  const [query, setQuery] = useState('');
  const [operatorId, setOperatorId] = useState(() => getOrInitOperatorId());
  const [searching, setSearching] = useState(false);
  const [response, setResponse] = useState<string | null>(null);
  const [logId, setLogId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleSearch = useCallback(async () => {
    if (!query.trim()) return;
    setSearching(true);
    setError(null);
    setResponse(null);
    setLogId(null);

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

      if (pilotLoggingEnabled) {
        // Log the query for pilot tracking (default-off by policy).
        const id = await invoke<string>('log_pilot_query', {
          query: query.trim(),
          response: responseText,
          operatorId,
        });
        setLogId(id);
      }
    } catch (err) {
      console.error('Pilot query failed:', err);
      setError(typeof err === 'string' ? err : 'Search failed');
    } finally {
      setSearching(false);
    }
  }, [query, operatorId, pilotLoggingEnabled]);

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

      {!pilotLoggingEnabled && (
        <div className="pilot-feedback-error">
          Pilot logging is disabled by policy. Set <code>ASSISTSUPPORT_ENABLE_PILOT_LOGGING=1</code> and restart to enable.
          {policy && (
            <> Current defaults: retention {policy.retention_days} days, max {policy.max_rows} rows.</>
          )}
        </div>
      )}

      <div className="pilot-tester-inputs">
        <div className="pilot-operator-row">
          <input
            type="text"
            value={operatorId}
            readOnly
            className="pilot-user-input"
            aria-label="Operator ID"
          />
          <Button
            variant="secondary"
            onClick={() => {
              const next = generateOperatorId();
              setOperatorId(next);
              localStorage.setItem(OPERATOR_ID_STORAGE_KEY, next);
            }}
          >
            Regenerate
          </Button>
        </div>
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
            disabled={searching || !query.trim()}
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

      {pilotLoggingEnabled && logId && (
        <FeedbackForm
          logId={logId}
          operatorId={operatorId}
          query={query.trim()}
          onSubmitted={handleFeedbackSubmitted}
        />
      )}
    </div>
  );
}
