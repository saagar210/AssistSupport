import { useEffect, useRef } from 'react';
import type { ContextSource } from '../../types';
import './ConversationThread.css';

export interface ConversationEntry {
  id: string;
  type: 'input' | 'search-results' | 'streaming' | 'response';
  timestamp: string;
  content: string;
  sources?: ContextSource[];
  metrics?: {
    tokens_per_second: number;
    sources_used: number;
    word_count: number;
  };
}

interface ConversationThreadProps {
  entries: ConversationEntry[];
  streamingText?: string;
  isStreaming?: boolean;
}

export function ConversationThread({ entries, streamingText, isStreaming }: ConversationThreadProps) {
  const bottomRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when entries change
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [entries, streamingText]);

  const formatTime = (timestamp: string) => {
    try {
      return new Date(timestamp).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
    } catch {
      return '';
    }
  };

  return (
    <div className="conversation-thread">
      {entries.length === 0 && !streamingText && (
        <div className="conversation-empty">
          <p>Start a conversation by typing a support query below.</p>
        </div>
      )}

      {entries.map(entry => (
        <div key={entry.id} className={`conversation-entry entry-${entry.type}`}>
          <div className="entry-header">
            <span className="entry-label">
              {entry.type === 'input' ? 'You' : entry.type === 'response' ? 'AI Response' : entry.type === 'search-results' ? 'KB Search' : 'Generating...'}
            </span>
            <span className="entry-time">{formatTime(entry.timestamp)}</span>
          </div>
          <div className="entry-content">
            {entry.type === 'search-results' ? (
              <div className="entry-sources-summary">
                {entry.sources?.length ? `Found ${entry.sources.length} relevant source${entry.sources.length !== 1 ? 's' : ''}` : 'No sources found'}
              </div>
            ) : (
              <div className="entry-text">{entry.content}</div>
            )}
          </div>
          {entry.type === 'response' && entry.sources && entry.sources.length > 0 && (
            <div className="entry-sources">
              {entry.sources.map((source) => (
                <span key={source.chunk_id} className="entry-source-chip">
                  {source.title || source.file_path}
                  <span className="source-chip-score">{(source.score * 100).toFixed(0)}%</span>
                </span>
              ))}
            </div>
          )}
          {entry.type === 'response' && entry.metrics && (
            <div className="entry-metrics">
              <span>{entry.metrics.word_count} words</span>
              <span>{entry.metrics.tokens_per_second.toFixed(1)} tok/s</span>
            </div>
          )}
        </div>
      ))}

      {isStreaming && streamingText && (
        <div className="conversation-entry entry-streaming">
          <div className="entry-header">
            <span className="entry-label">AI Response</span>
          </div>
          <div className="entry-content">
            <div className="entry-text">
              {streamingText}
              <span className="streaming-cursor">|</span>
            </div>
          </div>
        </div>
      )}

      <div ref={bottomRef} />
    </div>
  );
}
