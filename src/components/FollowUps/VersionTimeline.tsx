import { Button } from '../shared/Button';
import type { SavedDraft } from '../../types';
import './VersionTimeline.css';

interface VersionTimelineProps {
  versions: SavedDraft[];
  currentDraft: SavedDraft;
  onRestore: (version: SavedDraft) => void;
  onCompare: (versionA: SavedDraft, versionB: SavedDraft) => void;
  loading?: boolean;
}

function formatTimelineDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function truncatePreview(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength) + '...';
}

export function VersionTimeline({ versions, currentDraft, onRestore, onCompare, loading }: VersionTimelineProps) {
  if (loading) {
    return (
      <div className="version-timeline">
        <div className="timeline-loading">Loading versions...</div>
      </div>
    );
  }

  if (versions.length === 0) {
    return (
      <div className="version-timeline">
        <div className="timeline-empty">No previous versions found.</div>
      </div>
    );
  }

  return (
    <div className="version-timeline">
      <h4 className="timeline-title">Version History</h4>

      {/* Current version marker */}
      <div className="timeline-item timeline-item-current">
        <div className="timeline-dot current" />
        <div className="timeline-content">
          <div className="timeline-meta">
            <span className="timeline-date">{formatTimelineDate(currentDraft.created_at)}</span>
            <span className="timeline-badge current-badge">Current</span>
            {currentDraft.model_name && (
              <span className="timeline-model">{currentDraft.model_name}</span>
            )}
          </div>
          <div className="timeline-preview">
            {truncatePreview(currentDraft.response_text || 'No response', 120)}
          </div>
        </div>
      </div>

      {/* Previous versions */}
      {versions.map((version) => (
        <div key={version.id} className="timeline-item">
          <div className="timeline-dot" />
          <div className="timeline-content">
            <div className="timeline-meta">
              <span className="timeline-date">{formatTimelineDate(version.created_at)}</span>
              {version.model_name && (
                <span className="timeline-model">{version.model_name}</span>
              )}
            </div>
            <div className="timeline-preview">
              {truncatePreview(version.response_text || 'No response', 120)}
            </div>
            <div className="timeline-actions">
              <Button
                variant="ghost"
                size="small"
                onClick={() => onCompare(currentDraft, version)}
              >
                Compare
              </Button>
              <Button
                variant="ghost"
                size="small"
                onClick={() => onRestore(version)}
              >
                Restore
              </Button>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}
