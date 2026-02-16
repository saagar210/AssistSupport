import { useState, useEffect, useCallback } from 'react';
import {
  useAnalytics,
  AnalyticsSummary,
  ArticleUsage,
  LowRatingAnalysis,
  ResponseQualityDrilldownExamples,
  ResponseQualitySummary,
} from '../../hooks/useAnalytics';
import { useFeatureOps } from '../../hooks/useFeatureOps';
import { buildResponseQualityCoaching } from '../../features/analytics/qualityCoaching';
import { buildOperatorScorecard } from '../../features/analytics/operatorScorecard';
import { loadQueueHandoffSnapshot } from '../../features/inbox/queueModel';
import {
  getResponseQualityThresholds,
  RESPONSE_QUALITY_THRESHOLDS_UPDATED_EVENT,
  ResponseQualityThresholds,
} from '../../features/analytics/qualityThresholds';
import type { KbGapCandidate } from '../../types';
import { ArticleDetailPanel } from './ArticleDetailPanel';
import './AnalyticsTab.css';

type Period = 7 | 30 | 90 | null; // null = all time

const PERIODS: { label: string; value: Period }[] = [
  { label: '7 days', value: 7 },
  { label: '30 days', value: 30 },
  { label: '90 days', value: 90 },
  { label: 'All time', value: null },
];

export function AnalyticsTab() {
  const {
    getSummary,
    getKbUsage,
    getLowRatingAnalysis,
    getResponseQualitySummary,
    getResponseQualityDrilldownExamples,
  } = useAnalytics();
  const { getKbGapCandidates, updateKbGapStatus } = useFeatureOps();
  const [period, setPeriod] = useState<Period>(30);
  const [summary, setSummary] = useState<AnalyticsSummary | null>(null);
  const [qualitySummary, setQualitySummary] = useState<ResponseQualitySummary | null>(null);
  const [qualityDrilldown, setQualityDrilldown] = useState<ResponseQualityDrilldownExamples | null>(null);
  const [qualityThresholds, setQualityThresholds] = useState<ResponseQualityThresholds>(() => getResponseQualityThresholds());
  const [kbUsage, setKbUsage] = useState<ArticleUsage[]>([]);
  const [lowRatingData, setLowRatingData] = useState<LowRatingAnalysis | null>(null);
  const [gapCandidates, setGapCandidates] = useState<KbGapCandidate[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedArticleId, setSelectedArticleId] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [summaryData, kbData, lowRating, qualityData, qualityDrilldownData] = await Promise.all([
        getSummary(period ?? undefined),
        getKbUsage(period ?? undefined),
        getLowRatingAnalysis(period ?? undefined).catch(() => null),
        getResponseQualitySummary(period ?? undefined).catch(() => null),
        getResponseQualityDrilldownExamples(period ?? undefined, 6).catch(() => null),
      ]);
      const gaps = await getKbGapCandidates(12, 'open').catch(() => []);
      setSummary(summaryData);
      setQualitySummary(qualityData);
      setQualityDrilldown(qualityDrilldownData);
      setKbUsage(kbData);
      setLowRatingData(lowRating);
      setGapCandidates(gaps);
    } catch (err) {
      console.error('Failed to load analytics:', err);
      setError(typeof err === 'string' ? err : 'Failed to load analytics data');
    } finally {
      setLoading(false);
    }
  }, [
    period,
    getSummary,
    getKbUsage,
    getLowRatingAnalysis,
    getResponseQualitySummary,
    getResponseQualityDrilldownExamples,
    getKbGapCandidates,
  ]);

  const handleGapStatus = useCallback(async (id: string, status: 'accepted' | 'resolved' | 'ignored') => {
    await updateKbGapStatus(id, status);
    setGapCandidates(prev => prev.filter(g => g.id !== id));
  }, [updateKbGapStatus]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  useEffect(() => {
    const syncThresholds = () => setQualityThresholds(getResponseQualityThresholds());
    syncThresholds();
    window.addEventListener(RESPONSE_QUALITY_THRESHOLDS_UPDATED_EVENT, syncThresholds);
    window.addEventListener('storage', syncThresholds);
    return () => {
      window.removeEventListener(RESPONSE_QUALITY_THRESHOLDS_UPDATED_EVENT, syncThresholds);
      window.removeEventListener('storage', syncThresholds);
    };
  }, []);

  if (loading) {
    return (
      <div className="analytics-tab">
        <div className="analytics-loading">Loading analytics...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="analytics-tab">
        <div className="analytics-empty">
          <div className="analytics-empty-title">Unable to load analytics</div>
          <div className="analytics-empty-description">{error}</div>
        </div>
      </div>
    );
  }

  if (!summary) {
    return (
      <div className="analytics-tab">
        <div className="analytics-empty">
          <div className="analytics-empty-title">No analytics data</div>
          <div className="analytics-empty-description">
            Start using AssistSupport to see usage statistics here.
          </div>
        </div>
      </div>
    );
  }

  const maxDailyCount = Math.max(...summary.daily_counts.map(d => d.count), 1);

  return (
    <div className="analytics-tab">
      {/* Period Selector */}
      <div className="period-selector">
        {PERIODS.map(p => (
          <button
            key={p.label}
            className={`period-btn ${period === p.value ? 'active' : ''}`}
            onClick={() => setPeriod(p.value)}
          >
            {p.label}
          </button>
        ))}
      </div>

      {/* Stat Cards */}
      <div className="stat-cards">
        <div className="stat-card stat-card-clickable" title="View response details">
          <div className="stat-card-label">Total Responses</div>
          <div className="stat-card-value">{summary.responses_generated}</div>
        </div>
        <div className="stat-card stat-card-clickable" title="View search details">
          <div className="stat-card-label">Searches</div>
          <div className="stat-card-value">{summary.searches_performed}</div>
        </div>
        <div className="stat-card stat-card-clickable" title="View draft details">
          <div className="stat-card-label">Drafts Saved</div>
          <div className="stat-card-value">{summary.drafts_saved}</div>
        </div>
        <div className="stat-card stat-card-clickable" title="View rating details">
          <div className="stat-card-label">Avg Rating</div>
          <div className="stat-card-value">
            {summary.total_ratings > 0
              ? summary.average_rating.toFixed(1)
              : '--'}
          </div>
        </div>
      </div>

      {/* Charts Grid */}
      <div className="charts-grid">
        {/* Daily Activity Bar Chart */}
        <div className="bar-chart">
          <div className="bar-chart-title">Daily Activity</div>
          {summary.daily_counts.length > 0 ? (
            <div className="bar-chart-grid">
              {summary.daily_counts.map((day) => {
                const heightPercent = (day.count / maxDailyCount) * 100;
                const dateLabel = formatDateLabel(day.date);
                return (
                  <div key={day.date} className="bar-col" title={`${day.date}: ${day.count} events`}>
                    <div
                      className="bar-fill"
                      style={{ height: `${heightPercent}%` }}
                    />
                    <div className="bar-label">{dateLabel}</div>
                  </div>
                );
              })}
            </div>
          ) : (
            <div className="analytics-empty">
              <div className="analytics-empty-description">No activity data for this period</div>
            </div>
          )}
        </div>

        {/* Rating Distribution */}
        <RatingDistribution summary={summary} />
      </div>

      <ResponseQualityPanel
        summary={qualitySummary}
        thresholds={qualityThresholds}
        drilldown={qualityDrilldown}
      />

      {/* Quality Alert */}
      {lowRatingData && lowRatingData.low_rating_count > 0 && (
        <div className="low-rating-alert">
          <div className="section-title">Quality Alert</div>
          <div className="low-rating-summary">
            <strong>{lowRatingData.low_rating_count}</strong> low ratings ({lowRatingData.low_rating_percentage.toFixed(1)}% of {lowRatingData.total_rating_count} total)
          </div>
          {lowRatingData.feedback_categories.length > 0 && (
            <div className="feedback-categories">
              <div className="feedback-categories-title">Top Feedback Categories</div>
              {lowRatingData.feedback_categories.map(cat => (
                <div key={cat.category} className="feedback-category-row">
                  <span className="feedback-category-name">{cat.category}</span>
                  <span className="feedback-category-count">{cat.count}</span>
                </div>
              ))}
            </div>
          )}
          {lowRatingData.recent_feedback.length > 0 && (
            <div className="recent-feedback">
              <div className="recent-feedback-title">Recent Feedback</div>
              {lowRatingData.recent_feedback.slice(0, 5).map((fb, i) => (
                <div key={i} className="feedback-item">
                  <span className="feedback-item-rating">{'★'.repeat(fb.rating)}{'☆'.repeat(5 - fb.rating)}</span>
                  <span className="feedback-item-text">{fb.feedback_text}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* KB Usage Table */}
      <KbUsageTable articles={kbUsage} onArticleClick={setSelectedArticleId} />

      {/* KB Gap Detector */}
      <div className="kb-gap-panel">
        <div className="section-title">KB Gap Detector</div>
        {gapCandidates.length === 0 ? (
          <div className="analytics-empty">
            <div className="analytics-empty-description">No open gap candidates detected</div>
          </div>
        ) : (
          <div className="kb-gap-list">
            {gapCandidates.map(gap => (
              <div key={gap.id} className="kb-gap-item">
                <div className="kb-gap-title">{gap.sample_query}</div>
                <div className="kb-gap-meta">
                  <span>Occurrences: {gap.occurrences}</span>
                  <span>Low confidence: {gap.low_confidence_count}</span>
                  <span>Ungrounded: {gap.unsupported_claim_events}</span>
                  {gap.suggested_category && <span>Category: {gap.suggested_category}</span>}
                </div>
                <div className="kb-gap-actions">
                  <button className="kb-gap-btn" onClick={() => handleGapStatus(gap.id, 'accepted')}>Accept</button>
                  <button className="kb-gap-btn" onClick={() => handleGapStatus(gap.id, 'resolved')}>Resolve</button>
                  <button className="kb-gap-btn kb-gap-btn-muted" onClick={() => handleGapStatus(gap.id, 'ignored')}>Ignore</button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Article Detail Panel */}
      {selectedArticleId && (
        <ArticleDetailPanel
          documentId={selectedArticleId}
          onClose={() => setSelectedArticleId(null)}
        />
      )}
    </div>
  );
}

function ResponseQualityPanel({
  summary,
  thresholds,
  drilldown,
}: {
  summary: ResponseQualitySummary | null;
  thresholds: ResponseQualityThresholds;
  drilldown: ResponseQualityDrilldownExamples | null;
}) {
  if (!summary || summary.snapshots_count === 0) {
    return (
      <div className="response-quality-panel">
        <div className="section-title">Response Quality Signals</div>
        <div className="analytics-empty">
          <div className="analytics-empty-description">No response quality snapshots captured yet</div>
        </div>
      </div>
    );
  }

  const avgTimeSeconds = summary.avg_time_to_draft_ms != null
    ? (summary.avg_time_to_draft_ms / 1000).toFixed(1)
    : '--';
  const medianTimeSeconds = summary.median_time_to_draft_ms != null
    ? (summary.median_time_to_draft_ms / 1000).toFixed(1)
    : '--';
  const coaching = buildResponseQualityCoaching(summary, thresholds);
  const scorecard = buildOperatorScorecard(coaching, loadQueueHandoffSnapshot());

  return (
    <div className="response-quality-panel">
      <div className="response-quality-header">
        <div className="section-title">Response Quality Signals</div>
        {coaching && (
          <span className={`quality-severity-badge severity-${coaching.overallSeverity}`}>
            {coaching.overallSeverity === 'healthy' && 'Healthy'}
            {coaching.overallSeverity === 'watch' && 'Watch'}
            {coaching.overallSeverity === 'action' && 'Action'}
          </span>
        )}
      </div>
      <div className="response-quality-grid">
        <div className="response-quality-card">
          <span className="response-quality-label">Snapshots</span>
          <strong className="response-quality-value">{summary.snapshots_count}</strong>
        </div>
        <div className="response-quality-card">
          <span className="response-quality-label">Avg Words</span>
          <strong className="response-quality-value">{Math.round(summary.avg_word_count)}</strong>
        </div>
        <div className="response-quality-card">
          <span className="response-quality-label">Avg Edit Ratio</span>
          <strong className="response-quality-value">{(summary.avg_edit_ratio * 100).toFixed(1)}%</strong>
        </div>
        <div className="response-quality-card">
          <span className="response-quality-label">Edited Save Rate</span>
          <strong className="response-quality-value">{(summary.edited_save_rate * 100).toFixed(1)}%</strong>
        </div>
        <div className="response-quality-card">
          <span className="response-quality-label">Avg Time to Draft</span>
          <strong className="response-quality-value">{avgTimeSeconds}s</strong>
        </div>
        <div className="response-quality-card">
          <span className="response-quality-label">Median Time to Draft</span>
          <strong className="response-quality-value">{medianTimeSeconds}s</strong>
        </div>
        <div className="response-quality-card">
          <span className="response-quality-label">Copy per Save</span>
          <strong className="response-quality-value">{(summary.copy_per_saved_ratio * 100).toFixed(1)}%</strong>
        </div>
        <div className="response-quality-card">
          <span className="response-quality-label">Save Events</span>
          <strong className="response-quality-value">{summary.saved_count}</strong>
        </div>
      </div>
      {scorecard && (
        <div className={`operator-scorecard operator-scorecard-${scorecard.posture}`}>
          <div className="operator-scorecard-header">
            <div>
              <div className="operator-scorecard-title">Operator Scorecard</div>
              <p>{scorecard.summary}</p>
            </div>
            <div className="operator-scorecard-score">
              <strong>{scorecard.score}</strong>
              <span>/100</span>
            </div>
          </div>
          {scorecard.prioritySignals.length > 0 ? (
            <ul className="operator-scorecard-actions">
              {scorecard.prioritySignals.map((signal) => (
                <li key={`score-${signal.id}`}>
                  <strong>{signal.label}:</strong> {signal.guidance}
                </li>
              ))}
            </ul>
          ) : (
            <div className="operator-scorecard-actions-empty">
              No urgent actions this period. Keep current runbooks and monitor trend drift.
            </div>
          )}
        </div>
      )}
      {coaching && (
        <div className="response-quality-coaching">
          <div className="response-quality-coaching-title">Coaching thresholds</div>
          <ul className="response-quality-coaching-list">
            {coaching.signals.map((signal) => (
              <li key={signal.id} className={`response-quality-coaching-item severity-${signal.severity}`}>
                <div className="response-quality-coaching-item-head">
                  <strong>{signal.label}</strong>
                  <span>{signal.value}</span>
                </div>
                <p>{signal.guidance}</p>
                <p className="response-quality-coaching-hint">{signal.drilldownHint}</p>
                <small>{signal.threshold}</small>
                {signal.severity !== 'healthy' && (
                  <QualityDrilldownExamples signalId={signal.id} drilldown={drilldown} />
                )}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

function QualityDrilldownExamples({
  signalId,
  drilldown,
}: {
  signalId: 'edit_ratio' | 'time_to_draft' | 'copy_per_save' | 'edited_save_rate';
  drilldown: ResponseQualityDrilldownExamples | null;
}) {
  if (!drilldown) {
    return null;
  }
  const items = drilldown[signalId].slice(0, 3);
  if (items.length === 0) {
    return (
      <div className="quality-drilldown-empty">
        No matching draft examples captured yet for this period.
      </div>
    );
  }

  return (
    <div className="quality-drilldown">
      <div className="quality-drilldown-title">Draft examples to review</div>
      <ul className="quality-drilldown-list">
        {items.map((item) => (
          <li key={`${signalId}-${item.draft_id}-${item.created_at}`}>
            <div className="quality-drilldown-head">
              <code>{item.draft_id}</code>
              <span>{formatDrilldownMetric(signalId, item.metric_value)}</span>
            </div>
            {item.draft_excerpt && <p>{item.draft_excerpt}</p>}
          </li>
        ))}
      </ul>
    </div>
  );
}

function formatDrilldownMetric(
  signalId: 'edit_ratio' | 'time_to_draft' | 'copy_per_save' | 'edited_save_rate',
  metricValue: number,
): string {
  switch (signalId) {
    case 'edit_ratio':
      return `${(metricValue * 100).toFixed(1)}% edit ratio`;
    case 'time_to_draft':
      return `${(metricValue / 1000).toFixed(1)}s to draft`;
    case 'copy_per_save':
      return 'Saved without copy';
    case 'edited_save_rate':
      return `${(metricValue * 100).toFixed(1)}% edit ratio`;
    default:
      return String(metricValue);
  }
}

function RatingDistribution({ summary }: { summary: AnalyticsSummary }) {
  // Derive rating distribution from summary data.
  // The backend provides average_rating and total_ratings.
  // We display a placeholder distribution based on available data.
  const totalRatings = summary.total_ratings;

  // If we have no ratings, show empty state
  if (totalRatings === 0) {
    return (
      <div className="rating-distribution">
        <div className="section-title">Rating Distribution</div>
        <div className="analytics-empty">
          <div className="analytics-empty-description">No ratings yet</div>
        </div>
      </div>
    );
  }

  // Use real per-star counts from the backend
  const distribution = [5, 4, 3, 2, 1].map(stars => ({
    stars,
    count: summary.rating_distribution[stars - 1] ?? 0,
  }));

  const maxCount = Math.max(...distribution.map(d => d.count), 1);

  return (
    <div className="rating-distribution">
      <div className="section-title">Rating Distribution</div>
      {distribution.map(({ stars, count }) => (
        <div key={stars} className="rating-row">
          <div className="rating-label">{stars} star{stars !== 1 ? 's' : ''}</div>
          <div className="rating-bar-track">
            <div
              className="rating-bar-fill"
              style={{ width: `${(count / maxCount) * 100}%` }}
            />
          </div>
          <div className="rating-count">{count}</div>
        </div>
      ))}
    </div>
  );
}

function KbUsageTable({ articles, onArticleClick }: { articles: ArticleUsage[]; onArticleClick?: (id: string) => void }) {
  if (articles.length === 0) {
    return (
      <div className="kb-usage-table">
        <div className="kb-usage-header">
          <div>Article</div>
          <div style={{ textAlign: 'right' }}>Uses</div>
        </div>
        <div className="analytics-empty">
          <div className="analytics-empty-description">No article usage data yet</div>
        </div>
      </div>
    );
  }

  return (
    <div className="kb-usage-table">
      <div className="kb-usage-header">
        <div>Article</div>
        <div style={{ textAlign: 'right' }}>Uses</div>
      </div>
      {articles.map((article) => (
        <div
          key={article.document_id}
          className={`kb-usage-row ${onArticleClick ? 'kb-usage-row-clickable' : ''}`}
          onClick={() => onArticleClick?.(article.document_id)}
          role={onArticleClick ? 'button' : undefined}
          tabIndex={onArticleClick ? 0 : undefined}
        >
          <div className="kb-usage-title" title={article.title}>
            {article.title}
          </div>
          <div className="kb-usage-count">{article.usage_count}</div>
        </div>
      ))}
    </div>
  );
}

/** Format a date string (YYYY-MM-DD) into a short label (e.g., "Jan 5") */
function formatDateLabel(dateStr: string): string {
  try {
    const date = new Date(dateStr + 'T00:00:00');
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  } catch {
    return dateStr;
  }
}
