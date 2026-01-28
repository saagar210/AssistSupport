import { useState, useEffect, useCallback } from 'react';
import { useAnalytics, AnalyticsSummary, ArticleUsage, LowRatingAnalysis } from '../../hooks/useAnalytics';
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
  const { getSummary, getKbUsage, getLowRatingAnalysis } = useAnalytics();
  const [period, setPeriod] = useState<Period>(30);
  const [summary, setSummary] = useState<AnalyticsSummary | null>(null);
  const [kbUsage, setKbUsage] = useState<ArticleUsage[]>([]);
  const [lowRatingData, setLowRatingData] = useState<LowRatingAnalysis | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedArticleId, setSelectedArticleId] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [summaryData, kbData, lowRating] = await Promise.all([
        getSummary(period ?? undefined),
        getKbUsage(period ?? undefined),
        getLowRatingAnalysis(period ?? undefined).catch(() => null),
      ]);
      setSummary(summaryData);
      setKbUsage(kbData);
      setLowRatingData(lowRating);
    } catch (err) {
      console.error('Failed to load analytics:', err);
      setError(typeof err === 'string' ? err : 'Failed to load analytics data');
    } finally {
      setLoading(false);
    }
  }, [period, getSummary, getKbUsage, getLowRatingAnalysis]);

  useEffect(() => {
    loadData();
  }, [loadData]);

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

