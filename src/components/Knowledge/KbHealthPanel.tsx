import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { DocumentReviewInfo } from '../../types';
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
  const [needsReview, setNeedsReview] = useState<DocumentReviewInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadStats = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [data, reviewDocs] = await Promise.all([
        invoke<KbHealthStats>('get_kb_health_stats'),
        invoke<DocumentReviewInfo[]>('get_documents_needing_review', {
          staleDays: 30,
          limit: 3,
        }).catch(() => [] as DocumentReviewInfo[]),
      ]);
      setStats(data);
      setNeedsReview(reviewDocs);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadStats();
  }, [loadStats]);

  const handleMarkReviewed = useCallback(async (documentId: string) => {
    try {
      await invoke('mark_document_reviewed', { documentId, reviewedBy: null });
      setNeedsReview(prev => prev.filter(d => d.id !== documentId));
    } catch (err) {
      console.error('Failed to mark as reviewed:', err);
    }
  }, []);

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
        {needsReview.length > 0 && (
          <div className="kb-health-card warning">
            <span className="kb-health-value">{needsReview.length}+</span>
            <span className="kb-health-label">Needs Review</span>
          </div>
        )}
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
      {needsReview.length > 0 && (
        <div className="kb-health-review-list">
          <div className="kb-health-review-title">Top stale articles:</div>
          {needsReview.map((doc) => (
            <div key={doc.id} className="kb-health-review-item">
              <span className="kb-health-review-name">{doc.title || doc.file_path}</span>
              <button
                className="kb-health-review-btn"
                onClick={() => handleMarkReviewed(doc.id)}
              >
                Mark Reviewed
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
