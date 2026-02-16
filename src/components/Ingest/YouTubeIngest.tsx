import { useState } from 'react';
import { useIngest } from '../../hooks/useIngest';
import { Button } from '../shared/Button';

interface YouTubeIngestProps {
  namespaceId: string;
  ytdlpAvailable: boolean | null;
  onSuccess: (message: string) => void;
  onError: (message: string) => void;
}

export function YouTubeIngest({ namespaceId, ytdlpAvailable, onSuccess, onError }: YouTubeIngestProps) {
  const { ingestYoutube, ingesting } = useIngest();
  const [url, setUrl] = useState('');

  const handleIngest = async () => {
    if (!url.trim()) return;

    // Basic YouTube URL validation
    if (!url.includes('youtube.com') && !url.includes('youtu.be')) {
      onError('Please enter a valid YouTube URL');
      return;
    }

    try {
      const result = await ingestYoutube(url.trim(), namespaceId);
      onSuccess(`Ingested "${result.title}" (${result.chunk_count} chunks, ${result.word_count} words)`);
      setUrl('');
    } catch (e) {
      onError(`Failed to ingest YouTube video: ${e}`);
    }
  };

  if (ytdlpAvailable === false) {
    return (
      <div className="ingest-form">
        <div className="ingest-form-header">
          <h3>YouTube</h3>
          <p>Ingest transcripts from YouTube videos.</p>
        </div>

        <div className="ingest-warning">
          <strong>yt-dlp Not Installed</strong>
          <p>YouTube ingestion requires yt-dlp to be installed. Install it using Homebrew:</p>
          <code>brew install yt-dlp</code>
          <p>After installing, restart AssistSupport.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="ingest-form">
      <div className="ingest-form-header">
        <h3>YouTube</h3>
        <p>Ingest transcripts/captions from YouTube videos. The transcript will be extracted and stored in your knowledge base.</p>
      </div>

      <div className="ingest-form-field">
        <label htmlFor="youtube-url">YouTube URL</label>
        <input
          id="youtube-url"
          type="url"
          placeholder="https://www.youtube.com/watch?v=..."
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && !ingesting && handleIngest()}
          disabled={ingesting}
        />
      </div>

      <div className="ingest-form-info">
        <ul>
          <li>Videos must have captions/subtitles available</li>
          <li>Auto-generated captions will be used if no manual captions exist</li>
          <li>Private or age-restricted videos cannot be ingested</li>
        </ul>
      </div>

      <div className="ingest-form-actions">
        <Button
          variant="primary"
          onClick={handleIngest}
          disabled={!url.trim() || ingesting || ytdlpAvailable !== true}
        >
          {ingesting ? 'Ingesting...' : 'Ingest Transcript'}
        </Button>
      </div>
    </div>
  );
}
