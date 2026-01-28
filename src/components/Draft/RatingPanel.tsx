import { useState, useEffect, useCallback } from 'react';
import { useRatings, ResponseRating } from '../../hooks/useRatings';
import { Button } from '../shared/Button';
import './RatingPanel.css';

interface RatingPanelProps {
  draftId: string | null;
  onRated?: () => void;
}

const FEEDBACK_CATEGORIES = [
  { value: 'accuracy', label: 'Accuracy' },
  { value: 'relevance', label: 'Relevance' },
  { value: 'tone', label: 'Tone' },
  { value: 'completeness', label: 'Completeness' },
  { value: 'other', label: 'Other' },
];

export function RatingPanel({ draftId, onRated }: RatingPanelProps) {
  const { rateResponse, getDraftRating } = useRatings();

  const [rating, setRating] = useState<number>(0);
  const [hoverRating, setHoverRating] = useState<number>(0);
  const [feedbackCategory, setFeedbackCategory] = useState<string>('');
  const [feedbackText, setFeedbackText] = useState<string>('');
  const [existingRating, setExistingRating] = useState<ResponseRating | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [submitted, setSubmitted] = useState(false);

  // Load existing rating when draftId changes
  useEffect(() => {
    setRating(0);
    setHoverRating(0);
    setFeedbackCategory('');
    setFeedbackText('');
    setExistingRating(null);
    setSubmitted(false);

    if (!draftId) return;

    let cancelled = false;
    getDraftRating(draftId).then((existing) => {
      if (cancelled) return;
      if (existing) {
        setExistingRating(existing);
        setRating(existing.rating);
        setFeedbackCategory(existing.feedback_category || '');
        setFeedbackText(existing.feedback_text || '');
        setSubmitted(true);
      }
    }).catch(() => {
      // Rating fetch failed silently
    });

    return () => { cancelled = true; };
  }, [draftId, getDraftRating]);

  const handleStarClick = useCallback((starValue: number) => {
    if (submitted) return;
    setRating(starValue);
  }, [submitted]);

  const handleSubmit = useCallback(async () => {
    if (!draftId || rating === 0 || submitting) return;

    setSubmitting(true);
    try {
      await rateResponse(
        draftId,
        rating,
        feedbackText.trim() || undefined,
        feedbackCategory || undefined,
      );
      setSubmitted(true);
      onRated?.();
    } catch (err) {
      console.error('Failed to submit rating:', err);
    } finally {
      setSubmitting(false);
    }
  }, [draftId, rating, feedbackText, feedbackCategory, submitting, rateResponse, onRated]);

  if (!draftId) return null;

  const showFeedbackForm = rating > 0 && rating <= 2 && !submitted;
  const displayRating = hoverRating || rating;

  return (
    <div className="rating-panel">
      <div className="rating-header">
        <span className="rating-label">
          {submitted ? 'Your rating' : 'Rate this response'}
        </span>
      </div>

      <div className="rating-stars">
        {[1, 2, 3, 4, 5].map((star) => (
          <button
            key={star}
            className={`star ${displayRating >= star ? 'filled' : ''} ${submitted ? 'disabled' : ''}`}
            onClick={() => handleStarClick(star)}
            onMouseEnter={() => !submitted && setHoverRating(star)}
            onMouseLeave={() => !submitted && setHoverRating(0)}
            disabled={submitted}
            aria-label={`Rate ${star} star${star > 1 ? 's' : ''}`}
            type="button"
          >
            <span className="star-shape" aria-hidden="true" />
          </button>
        ))}
        {submitted && existingRating && (
          <span className="rating-saved-indicator">Saved</span>
        )}
      </div>

      {showFeedbackForm && (
        <div className="rating-feedback">
          <div className="feedback-field">
            <label htmlFor="feedback-category">Category (optional)</label>
            <select
              id="feedback-category"
              value={feedbackCategory}
              onChange={(e) => setFeedbackCategory(e.target.value)}
            >
              <option value="">Select a category...</option>
              {FEEDBACK_CATEGORIES.map((cat) => (
                <option key={cat.value} value={cat.value}>
                  {cat.label}
                </option>
              ))}
            </select>
          </div>

          <div className="feedback-field">
            <label htmlFor="feedback-text">Feedback (optional)</label>
            <textarea
              id="feedback-text"
              value={feedbackText}
              onChange={(e) => setFeedbackText(e.target.value)}
              placeholder="What could be improved?"
              rows={3}
            />
          </div>

          <div className="feedback-actions">
            <Button
              variant="primary"
              size="small"
              onClick={handleSubmit}
              loading={submitting}
              disabled={rating === 0}
            >
              Submit Rating
            </Button>
          </div>
        </div>
      )}

      {rating > 2 && !submitted && (
        <div className="rating-submit-inline">
          <Button
            variant="primary"
            size="small"
            onClick={handleSubmit}
            loading={submitting}
          >
            Submit Rating
          </Button>
        </div>
      )}
    </div>
  );
}
