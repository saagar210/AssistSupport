import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '../shared/Button';
import type { ArticleAnalytics } from '../../types';
import './ArticleDetailPanel.css';

interface ArticleDetailPanelProps {
  documentId: string;
  onClose: () => void;
  onEditArticle?: (documentId: string) => void;
}

export function ArticleDetailPanel({ documentId, onClose, onEditArticle }: ArticleDetailPanelProps) {
  const [analytics, setAnalytics] = useState<ArticleAnalytics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadAnalytics = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<ArticleAnalytics>('get_analytics_for_article', { documentId });
      setAnalytics(data);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [documentId]);

  useEffect(() => {
    loadAnalytics();
  }, [loadAnalytics]);

  if (loading) {
    return (
      <div className="article-detail-panel">
        <div className="article-detail-header">
          <h3>Article Analytics</h3>
          <Button variant="ghost" size="small" onClick={onClose}>Close</Button>
        </div>
        <div className="article-detail-loading">Loading analytics...</div>
      </div>
    );
  }

  if (error || !analytics) {
    return (
      <div className="article-detail-panel">
        <div className="article-detail-header">
          <h3>Article Analytics</h3>
          <Button variant="ghost" size="small" onClick={onClose}>Close</Button>
        </div>
        <div className="article-detail-error">{error || 'No data available'}</div>
      </div>
    );
  }

  return (
    <div className="article-detail-panel">
      <div className="article-detail-header">
        <div>
          <h3>{analytics.title}</h3>
          <span className="article-detail-path">{analytics.file_path}</span>
        </div>
        <div className="article-detail-actions">
          {onEditArticle && (
            <Button variant="secondary" size="small" onClick={() => onEditArticle(documentId)}>
              Edit KB
            </Button>
          )}
          <Button variant="ghost" size="small" onClick={onClose}>Close</Button>
        </div>
      </div>

      <div className="article-detail-stats">
        <div className="article-stat">
          <span className="article-stat-value">{analytics.total_uses}</span>
          <span className="article-stat-label">Times Used</span>
        </div>
        <div className="article-stat">
          <span className="article-stat-value">
            {analytics.average_rating != null ? analytics.average_rating.toFixed(1) : '--'}
          </span>
          <span className="article-stat-label">Avg Rating</span>
        </div>
      </div>

      <div className="article-detail-drafts">
        <h4>Responses Using This Article</h4>
        {analytics.draft_references.length === 0 ? (
          <div className="article-detail-empty">No responses have used this article yet.</div>
        ) : (
          <div className="article-drafts-list">
            {analytics.draft_references.map((ref) => (
              <div key={ref.draft_id} className="article-draft-item">
                <div className="article-draft-header">
                  <span className="article-draft-date">
                    {new Date(ref.created_at).toLocaleDateString(undefined, {
                      month: 'short', day: 'numeric', year: 'numeric',
                    })}
                  </span>
                  {ref.rating != null && (
                    <span className="article-draft-rating">
                      {'★'.repeat(ref.rating)}{'☆'.repeat(5 - ref.rating)}
                    </span>
                  )}
                </div>
                <div className="article-draft-input">{ref.input_text.slice(0, 120)}...</div>
                {ref.feedback_text && (
                  <div className="article-draft-feedback">"{ref.feedback_text}"</div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
