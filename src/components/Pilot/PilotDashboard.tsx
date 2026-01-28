import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '../shared/Button';
import './Pilot.css';

interface CategoryStat {
  category: string;
  query_count: number;
  feedback_count: number;
  accuracy_avg: number;
  clarity_avg: number;
  helpfulness_avg: number;
}

interface PilotStats {
  total_queries: number;
  total_feedback: number;
  accuracy_pct: number;
  clarity_avg: number;
  helpfulness_avg: number;
  by_category: CategoryStat[];
}

interface QueryLog {
  id: string;
  query: string;
  response: string;
  category: string;
  user_id: string;
  created_at: string;
}

export function PilotDashboard() {
  const [stats, setStats] = useState<PilotStats | null>(null);
  const [logs, setLogs] = useState<QueryLog[]>([]);
  const [showLogs, setShowLogs] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    try {
      const [statsData, logsData] = await Promise.all([
        invoke<PilotStats>('get_pilot_stats'),
        invoke<QueryLog[]>('get_pilot_query_logs'),
      ]);
      setStats(statsData);
      setLogs(logsData);
      setError(null);
    } catch (err) {
      console.error('Failed to load pilot data:', err);
      setError(typeof err === 'string' ? err : 'Failed to load pilot data');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 10000);
    return () => clearInterval(interval);
  }, [loadData]);

  const handleExport = useCallback(async () => {
    try {
      const home = await invoke<string>('get_home_dir').catch(() => '/tmp');
      const path = `${home}/pilot_export_${Date.now()}.csv`;
      const count = await invoke<number>('export_pilot_data', { path });
      alert(`Exported ${count} records to ${path}`);
    } catch (err) {
      console.error('Export failed:', err);
      alert('Export failed: ' + (typeof err === 'string' ? err : 'Unknown error'));
    }
  }, []);

  if (loading) {
    return (
      <div className="pilot-dashboard">
        <div className="pilot-loading">Loading pilot data...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="pilot-dashboard">
        <div className="pilot-error">{error}</div>
        <Button variant="secondary" onClick={loadData}>Retry</Button>
      </div>
    );
  }

  if (!stats) return null;

  const accuracyClass = stats.accuracy_pct >= 90 ? 'pilot-stat-good'
    : stats.accuracy_pct >= 80 ? 'pilot-stat-warn' : 'pilot-stat-bad';

  return (
    <div className="pilot-dashboard">
      <div className="pilot-header">
        <h2 className="pilot-title">Pilot Progress</h2>
        <div className="pilot-actions">
          <Button variant="secondary" onClick={loadData}>Refresh</Button>
          {stats.total_queries > 0 && (
            <Button variant="secondary" onClick={handleExport}>Export CSV</Button>
          )}
        </div>
      </div>

      <div className="pilot-stats-grid">
        <div className="pilot-stat-card">
          <div className="pilot-stat-value">{stats.total_queries}</div>
          <div className="pilot-stat-label">Queries Tested</div>
        </div>
        <div className="pilot-stat-card">
          <div className="pilot-stat-value">{stats.total_feedback}</div>
          <div className="pilot-stat-label">Feedback Received</div>
        </div>
        <div className={`pilot-stat-card ${accuracyClass}`}>
          <div className="pilot-stat-value">
            {stats.total_feedback > 0 ? `${Math.round(stats.accuracy_pct)}%` : '--'}
          </div>
          <div className="pilot-stat-label">Accuracy (4-5 star)</div>
        </div>
        <div className="pilot-stat-card">
          <div className="pilot-stat-value">
            {stats.total_feedback > 0 ? stats.clarity_avg.toFixed(1) : '--'}
          </div>
          <div className="pilot-stat-label">Clarity Avg</div>
        </div>
        <div className="pilot-stat-card">
          <div className="pilot-stat-value">
            {stats.total_feedback > 0 ? stats.helpfulness_avg.toFixed(1) : '--'}
          </div>
          <div className="pilot-stat-label">Helpfulness Avg</div>
        </div>
      </div>

      {stats.by_category.length > 0 && (
        <div className="pilot-category-section">
          <h3 className="pilot-section-title">By Category</h3>
          <table className="pilot-category-table">
            <thead>
              <tr>
                <th>Category</th>
                <th>Queries</th>
                <th>Feedback</th>
                <th>Accuracy</th>
                <th>Clarity</th>
                <th>Helpfulness</th>
              </tr>
            </thead>
            <tbody>
              {stats.by_category.map(cat => (
                <tr key={cat.category}>
                  <td className="pilot-category-name">{cat.category}</td>
                  <td>{cat.query_count}</td>
                  <td>{cat.feedback_count}</td>
                  <td>{cat.feedback_count > 0 ? cat.accuracy_avg.toFixed(1) : '--'}</td>
                  <td>{cat.feedback_count > 0 ? cat.clarity_avg.toFixed(1) : '--'}</td>
                  <td>{cat.feedback_count > 0 ? cat.helpfulness_avg.toFixed(1) : '--'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {logs.length > 0 && (
        <div className="pilot-logs-section">
          <button
            className="pilot-logs-toggle"
            onClick={() => setShowLogs(!showLogs)}
          >
            {showLogs ? 'Hide' : 'Show'} Query Log ({logs.length})
          </button>
          {showLogs && (
            <div className="pilot-logs-list">
              {logs.map(log => (
                <div key={log.id} className="pilot-log-entry">
                  <div className="pilot-log-meta">
                    <span className={`pilot-log-category cat-${log.category}`}>
                      {log.category}
                    </span>
                    <span className="pilot-log-user">{log.user_id}</span>
                    <span className="pilot-log-time">
                      {new Date(log.created_at).toLocaleString()}
                    </span>
                  </div>
                  <div className="pilot-log-query">{log.query}</div>
                  <div className="pilot-log-response">{log.response.substring(0, 200)}{log.response.length > 200 ? '...' : ''}</div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {stats.total_queries === 0 && (
        <div className="pilot-empty">
          No pilot data yet. Queries will appear here as team members test the system.
        </div>
      )}
    </div>
  );
}
