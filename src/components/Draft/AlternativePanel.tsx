import { Button } from '../shared/Button';
import type { ResponseAlternative } from '../../types';
import './AlternativePanel.css';

interface AlternativePanelProps {
  alternatives: ResponseAlternative[];
  onChoose: (alternativeId: string, choice: 'original' | 'alternative') => void;
  onUseAlternative: (text: string) => void;
}

export function AlternativePanel({ alternatives, onChoose, onUseAlternative }: AlternativePanelProps) {
  if (alternatives.length === 0) return null;

  const latest = alternatives[0];

  return (
    <div className="alternative-panel">
      <h4 className="alternative-panel-title">Response Comparison</h4>
      <div className="comparison-panel">
        <div className={`comparison-column ${latest.chosen === 'original' ? 'chosen' : ''}`}>
          <div className="comparison-column-header">
            <span className="comparison-label">Original</span>
            {latest.chosen === 'original' && <span className="comparison-chosen-badge">Chosen</span>}
          </div>
          <div className="comparison-text">{latest.original_text}</div>
          {!latest.chosen && (
            <div className="comparison-actions">
              <Button
                variant="secondary"
                size="small"
                onClick={() => onChoose(latest.id, 'original')}
              >
                Use This One
              </Button>
            </div>
          )}
        </div>

        <div className={`comparison-column ${latest.chosen === 'alternative' ? 'chosen' : ''}`}>
          <div className="comparison-column-header">
            <span className="comparison-label">Alternative</span>
            {latest.chosen === 'alternative' && <span className="comparison-chosen-badge">Chosen</span>}
          </div>
          <div className="comparison-text">{latest.alternative_text}</div>
          {!latest.chosen && (
            <div className="comparison-actions">
              <Button
                variant="primary"
                size="small"
                onClick={() => {
                  onChoose(latest.id, 'alternative');
                  onUseAlternative(latest.alternative_text);
                }}
              >
                Use This One
              </Button>
            </div>
          )}
        </div>
      </div>

      {alternatives.length > 1 && (
        <div className="alternative-history">
          <span className="alternative-history-label">
            {alternatives.length - 1} previous alternative{alternatives.length > 2 ? 's' : ''}
          </span>
        </div>
      )}
    </div>
  );
}
