import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '../shared/Button';
import './Pilot.css';

interface FeedbackFormProps {
  logId: string;
  userId: string;
  query: string;
  onSubmitted?: () => void;
}

export function FeedbackForm({ logId, userId, query: _query, onSubmitted }: FeedbackFormProps) {
  const [accuracy, setAccuracy] = useState(3);
  const [clarity, setClarity] = useState(3);
  const [helpfulness, setHelpfulness] = useState(3);
  const [comment, setComment] = useState('');
  const [submitted, setSubmitted] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = useCallback(async () => {
    if (submitting) return;
    setSubmitting(true);
    setError(null);

    try {
      await invoke('submit_pilot_feedback', {
        queryLogId: logId,
        userId,
        accuracy,
        clarity,
        helpfulness,
        comment: comment.trim() || null,
      });
      setSubmitted(true);
      onSubmitted?.();
    } catch (err) {
      console.error('Failed to submit feedback:', err);
      setError(typeof err === 'string' ? err : 'Failed to submit feedback');
    } finally {
      setSubmitting(false);
    }
  }, [logId, userId, accuracy, clarity, helpfulness, comment, submitting, onSubmitted]);

  if (submitted) {
    return (
      <div className="pilot-feedback-success">
        Feedback recorded. Thank you!
      </div>
    );
  }

  return (
    <div className="pilot-feedback-form">
      <h3 className="pilot-feedback-title">Rate this response</h3>

      <div className="pilot-rating-group">
        <div className="pilot-rating-row">
          <label className="pilot-rating-label">
            Accuracy
            <span className="pilot-rating-hint">Was the response correct?</span>
          </label>
          <div className="pilot-rating-stars">
            {[1, 2, 3, 4, 5].map(n => (
              <button
                key={n}
                className={`pilot-star ${n <= accuracy ? 'active' : ''}`}
                onClick={() => setAccuracy(n)}
                title={`${n}/5`}
              >
                {n <= accuracy ? '\u2605' : '\u2606'}
              </button>
            ))}
            <span className="pilot-rating-value">{accuracy}/5</span>
          </div>
        </div>

        <div className="pilot-rating-row">
          <label className="pilot-rating-label">
            Clarity
            <span className="pilot-rating-hint">Was it easy to understand?</span>
          </label>
          <div className="pilot-rating-stars">
            {[1, 2, 3, 4, 5].map(n => (
              <button
                key={n}
                className={`pilot-star ${n <= clarity ? 'active' : ''}`}
                onClick={() => setClarity(n)}
                title={`${n}/5`}
              >
                {n <= clarity ? '\u2605' : '\u2606'}
              </button>
            ))}
            <span className="pilot-rating-value">{clarity}/5</span>
          </div>
        </div>

        <div className="pilot-rating-row">
          <label className="pilot-rating-label">
            Helpfulness
            <span className="pilot-rating-hint">Did it solve the problem?</span>
          </label>
          <div className="pilot-rating-stars">
            {[1, 2, 3, 4, 5].map(n => (
              <button
                key={n}
                className={`pilot-star ${n <= helpfulness ? 'active' : ''}`}
                onClick={() => setHelpfulness(n)}
                title={`${n}/5`}
              >
                {n <= helpfulness ? '\u2605' : '\u2606'}
              </button>
            ))}
            <span className="pilot-rating-value">{helpfulness}/5</span>
          </div>
        </div>
      </div>

      <div className="pilot-comment-section">
        <label className="pilot-rating-label">
          Comment
          <span className="pilot-rating-hint">Optional — what could be improved?</span>
        </label>
        <textarea
          value={comment}
          onChange={e => setComment(e.target.value)}
          placeholder="e.g., the suggestion didn't apply to our situation..."
          className="pilot-comment-input"
          rows={2}
        />
      </div>

      {error && <div className="pilot-feedback-error">{error}</div>}

      <Button
        variant="primary"
        onClick={handleSubmit}
        disabled={submitting}
      >
        {submitting ? 'Submitting...' : 'Submit Feedback'}
      </Button>
    </div>
  );
}
