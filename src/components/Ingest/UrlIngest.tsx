import { useState } from 'react';
import { useIngest } from '../../hooks/useIngest';
import { Button } from '../shared/Button';

interface UrlIngestProps {
  namespaceId: string;
  onSuccess: (message: string) => void;
  onError: (message: string) => void;
}

export function UrlIngest({ namespaceId, onSuccess, onError }: UrlIngestProps) {
  const { ingestUrl, ingesting } = useIngest();
  const [url, setUrl] = useState('');

  const handleIngest = async () => {
    if (!url.trim()) return;

    // Basic URL validation
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      onError('URL must start with http:// or https://');
      return;
    }

    try {
      const result = await ingestUrl(url.trim(), namespaceId);
      onSuccess(`Ingested "${result.title}" (${result.chunk_count} chunks, ${result.word_count} words)`);
      setUrl('');
    } catch (e) {
      onError(`Failed to ingest URL: ${e}`);
    }
  };

  return (
    <div className="ingest-form">
      <div className="ingest-form-header">
        <h3>Web Page</h3>
        <p>Ingest content from a web page URL. The page will be fetched, processed, and stored in your knowledge base.</p>
      </div>

      <div className="ingest-form-field">
        <label htmlFor="url">URL</label>
        <input
          id="url"
          type="url"
          placeholder="https://example.com/docs/page"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && !ingesting && handleIngest()}
          disabled={ingesting}
        />
      </div>

      <div className="ingest-form-info">
        <ul>
          <li>Only public pages can be ingested</li>
          <li>Private/login-protected pages will return an error</li>
          <li>Large pages may take longer to process</li>
        </ul>
      </div>

      <div className="ingest-form-actions">
        <Button
          variant="primary"
          onClick={handleIngest}
          disabled={!url.trim() || ingesting}
        >
          {ingesting ? 'Ingesting...' : 'Ingest Page'}
        </Button>
      </div>
    </div>
  );
}
