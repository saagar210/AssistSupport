import { useState, useEffect, useCallback, useRef } from 'react';
import { useHybridSearch } from '../../hooks/useHybridSearch';
import { useToastContext } from '../../contexts/ToastContext';
import { Button } from '../shared/Button';
import { Icon } from '../shared/Icon';
import type { HybridSearchResult, SearchApiStatsData } from '../../types';
import './HybridSearchTab.css';

const CATEGORY_COLORS: Record<string, string> = {
  POLICY: 'var(--error)',
  PROCEDURE: 'var(--info)',
  REFERENCE: 'var(--accent-primary)',
};

const CATEGORY_BG: Record<string, string> = {
  POLICY: 'var(--error-lighter)',
  PROCEDURE: 'var(--info-lighter)',
  REFERENCE: 'var(--accent-lighter)',
};

export function HybridSearchTab() {
  const {
    response,
    searching,
    error,
    apiHealthy,
    search,
    submitFeedback,
    getStats,
    checkHealth,
    clearResults,
  } = useHybridSearch();

  const { success: showSuccess, error: showError } = useToastContext();
  const [query, setQuery] = useState('');
  const [ratings, setRatings] = useState<Record<number, string>>({});
  const [stats, setStats] = useState<SearchApiStatsData | null>(null);
  const [showStats, setShowStats] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // Check API health on mount
  useEffect(() => {
    checkHealth();
  }, [checkHealth]);

  const handleSearch = useCallback(async (e?: React.FormEvent) => {
    e?.preventDefault();
    const trimmed = query.trim();
    if (!trimmed) return;
    setRatings({});
    await search(trimmed);
  }, [query, search]);

  const handleFeedback = useCallback(async (
    result: HybridSearchResult,
    rating: 'helpful' | 'not_helpful' | 'incorrect',
  ) => {
    if (!response?.query_id) return;
    const ok = await submitFeedback(response.query_id, result.rank, rating);
    if (ok) {
      setRatings(prev => ({ ...prev, [result.rank]: rating }));
      showSuccess('Feedback recorded');
    } else {
      showError('Failed to submit feedback');
    }
  }, [response, submitFeedback, showSuccess, showError]);

  const handleShowStats = useCallback(async () => {
    const data = await getStats();
    if (data) {
      setStats(data);
      setShowStats(true);
    }
  }, [getStats]);

  const handleClear = useCallback(() => {
    setQuery('');
    setRatings({});
    clearResults();
    inputRef.current?.focus();
  }, [clearResults]);

  return (
    <div className="hybrid-search-tab">
      <div className="hybrid-search-header">
        <div className="hybrid-search-title-row">
          <h2>Hybrid Search</h2>
          <div className="hybrid-search-status">
            <span className={`api-status-dot ${apiHealthy === true ? 'healthy' : apiHealthy === false ? 'unhealthy' : 'unknown'}`} />
            <span className="api-status-text">
              {apiHealthy === true ? 'PostgreSQL API' : apiHealthy === false ? 'API Offline' : 'Checking...'}
            </span>
          </div>
        </div>
        <p className="hybrid-search-subtitle">
          BM25 keyword + HNSW vector search across 3,536 knowledge base articles
        </p>
      </div>

      <form className="hybrid-search-form" onSubmit={handleSearch}>
        <div className="hybrid-search-input-row">
          <div className="hybrid-search-input-wrapper">
            <Icon name="search" size={16} className="search-icon" />
            <input
              ref={inputRef}
              type="text"
              className="hybrid-search-input"
              placeholder="Ask about policies, procedures, or search for information..."
              value={query}
              onChange={e => setQuery(e.target.value)}
              disabled={searching}
              autoFocus
            />
            {query && (
              <button type="button" className="search-clear-btn" onClick={handleClear}>
                <Icon name="x" size={14} />
              </button>
            )}
          </div>
          <Button
            variant="primary"
            type="submit"
            disabled={searching || !query.trim()}
          >
            {searching ? 'Searching...' : 'Search'}
          </Button>
          <Button variant="ghost" type="button" onClick={handleShowStats} title="View stats">
            <Icon name="sparkles" size={16} />
          </Button>
        </div>
      </form>

      {error && (
        <div className="hybrid-search-error">
          <Icon name="x" size={16} />
          <span>{error}</span>
        </div>
      )}

      {showStats && stats && (
        <div className="hybrid-search-stats-panel">
          <div className="stats-header">
            <h3>Search Statistics (24h)</h3>
            <button className="stats-close" onClick={() => setShowStats(false)}>
              <Icon name="x" size={14} />
            </button>
          </div>
          <div className="stats-grid">
            <div className="stat-card">
              <span className="stat-value">{stats.queries_total}</span>
              <span className="stat-label">Total Queries</span>
            </div>
            <div className="stat-card">
              <span className="stat-value">{stats.queries_24h}</span>
              <span className="stat-label">Last 24h</span>
            </div>
            <div className="stat-card">
              <span className="stat-value">{stats.latency_ms.avg.toFixed(0)}ms</span>
              <span className="stat-label">Avg Latency</span>
            </div>
            <div className="stat-card">
              <span className="stat-value">{stats.latency_ms.p95.toFixed(0)}ms</span>
              <span className="stat-label">p95 Latency</span>
            </div>
            <div className="stat-card">
              <span className="stat-value">{stats.feedback_stats.helpful}</span>
              <span className="stat-label">Helpful</span>
            </div>
            <div className="stat-card">
              <span className="stat-value">{Object.keys(stats.intent_distribution).length}</span>
              <span className="stat-label">Intents</span>
            </div>
          </div>
        </div>
      )}

      {response && (
        <div className="hybrid-search-results">
          <div className="results-meta">
            <span className="results-count">{response.results_count} results</span>
            <span className="results-intent" style={{
              color: CATEGORY_COLORS[response.intent.toUpperCase()] ?? 'var(--text-secondary)',
              backgroundColor: CATEGORY_BG[response.intent.toUpperCase()] ?? 'var(--bg-tertiary)',
            }}>
              {response.intent}
              <span className="intent-confidence">
                {(response.intent_confidence * 100).toFixed(0)}%
              </span>
            </span>
            <span className="results-latency">{response.metrics.latency_ms.toFixed(0)}ms</span>
          </div>

          {response.results.length === 0 ? (
            <div className="no-results">
              <p>No results found for &quot;{response.query}&quot;</p>
              <p className="no-results-hint">Try different keywords or a broader search term.</p>
            </div>
          ) : (
            <div className="results-list">
              {response.results.map(result => (
                <ResultCard
                  key={`${response.query_id}-${result.rank}`}
                  result={result}
                  currentRating={ratings[result.rank] ?? null}
                  onFeedback={handleFeedback}
                />
              ))}
            </div>
          )}
        </div>
      )}

      {!response && !error && !searching && (
        <div className="hybrid-search-empty">
          <Icon name="search" size={32} className="empty-icon" />
          <p>Search across policies, procedures, and reference articles</p>
          <div className="example-queries">
            {['Can I use a USB flash drive?', 'How do I reset my password?', 'VPN setup instructions'].map(q => (
              <button
                key={q}
                className="example-query"
                onClick={() => {
                  setQuery(q);
                  search(q);
                }}
              >
                {q}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

interface ResultCardProps {
  result: HybridSearchResult;
  currentRating: string | null;
  onFeedback: (result: HybridSearchResult, rating: 'helpful' | 'not_helpful' | 'incorrect') => void;
}

function ResultCard({ result, currentRating, onFeedback }: ResultCardProps) {
  const categoryColor = CATEGORY_COLORS[result.category] ?? 'var(--text-secondary)';
  const categoryBg = CATEGORY_BG[result.category] ?? 'var(--bg-tertiary)';

  return (
    <div className="result-card">
      <div className="result-card-header">
        <span className="result-rank">#{result.rank}</span>
        <h3 className="result-title">{result.title}</h3>
        <span className="result-category" style={{ color: categoryColor, backgroundColor: categoryBg }}>
          {result.category}
        </span>
      </div>

      {result.section && (
        <div className="result-section">{result.section}</div>
      )}

      <p className="result-preview">{result.preview}</p>

      {result.scores && (
        <div className="result-scores">
          <span className="score">BM25: {result.scores.bm25.toFixed(3)}</span>
          <span className="score">Vector: {result.scores.vector.toFixed(3)}</span>
          <span className="score score-fused">Fused: {result.scores.fused.toFixed(3)}</span>
        </div>
      )}

      <div className="result-feedback">
        {(['helpful', 'not_helpful', 'incorrect'] as const).map(rating => (
          <button
            key={rating}
            className={`feedback-btn ${currentRating === rating ? 'active' : ''} ${currentRating && currentRating !== rating ? 'dimmed' : ''}`}
            onClick={() => onFeedback(result, rating)}
            disabled={!!currentRating}
            title={rating === 'helpful' ? 'This result was helpful' : rating === 'not_helpful' ? 'This result was not helpful' : 'This result is incorrect'}
          >
            {rating === 'helpful' ? 'Helpful' : rating === 'not_helpful' ? 'Not Helpful' : 'Incorrect'}
          </button>
        ))}
      </div>
    </div>
  );
}

export default HybridSearchTab;
