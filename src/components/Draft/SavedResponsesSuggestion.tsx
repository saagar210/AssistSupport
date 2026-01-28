import type { SavedResponseTemplate } from '../../types';
import './SavedResponsesSuggestion.css';

interface SavedResponsesSuggestionProps {
  suggestions: SavedResponseTemplate[];
  onApply: (content: string, templateId: string) => void;
  onDismiss: () => void;
}

export function SavedResponsesSuggestion({ suggestions, onApply, onDismiss }: SavedResponsesSuggestionProps) {
  if (suggestions.length === 0) return null;

  return (
    <div className="suggestion-banner">
      <span className="suggestion-banner-icon">&#128161;</span>
      <div className="suggestion-banner-content">
        <span className="suggestion-banner-text">
          You've responded to similar tickets before ({suggestions.length} saved template{suggestions.length > 1 ? 's' : ''})
        </span>
        <div className="suggestion-banner-items">
          {suggestions.slice(0, 3).map((s) => (
            <button
              key={s.id}
              className="suggestion-item"
              onClick={() => onApply(s.content, s.id)}
              title={s.content.slice(0, 200)}
            >
              <span className="suggestion-item-name">{s.name}</span>
              {s.use_count > 0 && (
                <span className="suggestion-item-count">used {s.use_count}x</span>
              )}
            </button>
          ))}
        </div>
      </div>
      <button className="suggestion-dismiss" onClick={onDismiss} title="Dismiss">&times;</button>
    </div>
  );
}
