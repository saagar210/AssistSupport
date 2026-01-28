import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './KbHealthPanel.css';

interface KbHealthStats {
  total_documents: number;
  total_chunks: number;
  stale_documents: number;
  namespace_distribution: Array<{
    namespace_id: string;
    namespace_name: string;
    document_count: number;
    chunk_count: number;
  }>;
}

interface KbHealthPanelProps {
  onRefresh?: () => void;
}

export function KbHealthPanel({ onRefresh }: KbHealthPanelProps) {
  const [stats, setStats] = useState<KbHealthStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadStats = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<KbHealthStats>('get_kb_health_stats');
      setStats(data);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadStats();
  }, [loadStats]);

  if (loading) {
    return <div className="kb-health-panel loading">Loading KB health...</div>;
  }

  if (error) {
    return <div className="kb-health-panel error">Failed to load KB health: {error}</div>;
  }

  if (!stats) return null;

  return (
    <div className="kb-health-panel">
      <div className="kb-health-cards">
        <div className="kb-health-card">
          <span className="kb-health-value">{stats.total_documents}</span>
          <span className="kb-health-label">Documents</span>
        </div>
        <div className="kb-health-card">
          <span className="kb-health-value">{stats.total_chunks}</span>
          <span className="kb-health-label">Chunks</span>
        </div>
        <div className={`kb-health-card ${stats.stale_documents > 0 ? 'warning' : ''}`}>
          <span className="kb-health-value">{stats.stale_documents}</span>
          <span className="kb-health-label">Stale</span>
        </div>
        <div className="kb-health-card">
          <span className="kb-health-value">{stats.namespace_distribution.length}</span>
          <span className="kb-health-label">Namespaces</span>
        </div>
      </div>
      {stats.stale_documents > 0 && (
        <div className="kb-health-warning">
          {stats.stale_documents} document{stats.stale_documents !== 1 ? 's' : ''} may need re-indexing.
          {onRefresh && (
            <button className="kb-health-refresh" onClick={onRefresh}>
              Re-index
            </button>
          )}
        </div>
      )}
    </div>
  );
}
