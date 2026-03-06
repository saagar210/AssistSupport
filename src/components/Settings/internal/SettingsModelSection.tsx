import { Button } from '../../shared/Button';
import type { DownloadProgress, MemoryKernelPreflightStatus, ModelInfo } from '../../../types';

interface SettingsModelSectionProps {
  loadedModel: string | null;
  downloadedModels: string[];
  recommendedModels: ModelInfo[];
  otherSupportedModels: ModelInfo[];
  showOtherModels: boolean;
  loading: string | null;
  isDownloading: boolean;
  downloadProgress: DownloadProgress | null;
  isEmbeddingLoaded: boolean;
  kbFolder: string | null;
  memoryKernelPreflight: MemoryKernelPreflightStatus | null;
  memoryKernelLoading: boolean;
  onUnloadModel: () => void;
  onLoadModel: (modelId: string) => void;
  onDownloadModel: (modelId: string) => void;
  onCancelDownload: () => void;
  onToggleOtherModels: () => void;
  onLoadCustomModel: () => void;
  onRefreshMemoryKernelStatus: () => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

function formatSpeed(bps: number): string {
  if (bps === 0) return '';
  return `${formatBytes(bps)}/s`;
}

function ModelCard({
  model,
  loadedModel,
  downloadedModels,
  loading,
  isDownloading,
  downloadProgress,
  onLoadModel,
  onUnloadModel,
  onDownloadModel,
  onCancelDownload,
}: {
  model: ModelInfo;
  loadedModel: string | null;
  downloadedModels: string[];
  loading: string | null;
  isDownloading: boolean;
  downloadProgress: DownloadProgress | null;
  onLoadModel: (modelId: string) => void;
  onUnloadModel: () => void;
  onDownloadModel: (modelId: string) => void;
  onCancelDownload: () => void;
}) {
  const isDownloaded = downloadedModels.includes(model.id);
  const isLoaded = loadedModel === model.id;
  const isLoadingThis = loading === model.id;
  const isDownloadingThis = isDownloading && downloadProgress?.model_id === model.id;

  return (
    <div key={model.id} className={`model-card ${isLoaded ? 'loaded' : ''}`}>
      <div className="model-info">
        <h3>{model.name}</h3>
        <p>{model.description}</p>
        <span className="model-size">{model.size}</span>
      </div>
      <div className="model-actions">
        {isDownloadingThis ? (
          <div className="download-progress-container">
            <div className="download-progress">
              <div
                className="download-bar"
                style={{ width: `${downloadProgress?.percent || 0}%` }}
              />
              <span className="download-percent">{Math.round(downloadProgress?.percent || 0)}%</span>
            </div>
            <div className="download-info">
              <span className="download-size">
                {formatBytes(downloadProgress?.downloaded_bytes || 0)}
                {downloadProgress?.total_bytes ? ` / ${formatBytes(downloadProgress.total_bytes)}` : ''}
              </span>
              <span className="download-speed">{formatSpeed(downloadProgress?.speed_bps || 0)}</span>
            </div>
            <Button
              variant="ghost"
              size="small"
              onClick={onCancelDownload}
              className="download-cancel-btn"
            >
              Cancel
            </Button>
          </div>
        ) : isDownloaded ? (
          <Button
            variant={isLoaded ? 'secondary' : 'primary'}
            size="small"
            onClick={() => (isLoaded ? onUnloadModel() : onLoadModel(model.id))}
            disabled={!!loading}
          >
            {isLoadingThis ? 'Loading...' : isLoaded ? 'Unload' : 'Load'}
          </Button>
        ) : (
          <Button
            variant="secondary"
            size="small"
            onClick={() => onDownloadModel(model.id)}
            disabled={isDownloading}
          >
            Download
          </Button>
        )}
      </div>
    </div>
  );
}

export function SettingsModelSection({
  loadedModel,
  downloadedModels,
  recommendedModels,
  otherSupportedModels,
  showOtherModels,
  loading,
  isDownloading,
  downloadProgress,
  isEmbeddingLoaded,
  kbFolder,
  memoryKernelPreflight,
  memoryKernelLoading,
  onUnloadModel,
  onLoadModel,
  onDownloadModel,
  onCancelDownload,
  onToggleOtherModels,
  onLoadCustomModel,
  onRefreshMemoryKernelStatus,
}: SettingsModelSectionProps) {
  return (
    <section className="settings-section">
      <h2>Language Model</h2>
      <p className="settings-description">
        Select and load a language model for generating responses.
      </p>

      {loadedModel && (
        <div className="loaded-model-banner">
          <span>Currently loaded: <strong>{loadedModel}</strong></span>
          <Button
            variant="secondary"
            size="small"
            onClick={onUnloadModel}
            disabled={loading === 'unload'}
          >
            {loading === 'unload' ? 'Unloading...' : 'Unload'}
          </Button>
        </div>
      )}

      <div className="settings-subsection">
        <h3>Recommended</h3>
        <p className="setting-note">
          For consistent results across operators, AssistSupport recommends a single default model.
        </p>
      </div>
      <div className="model-list">
        {recommendedModels.map((model) => (
          <ModelCard
            key={model.id}
            model={model}
            loadedModel={loadedModel}
            downloadedModels={downloadedModels}
            loading={loading}
            isDownloading={isDownloading}
            downloadProgress={downloadProgress}
            onLoadModel={onLoadModel}
            onUnloadModel={onUnloadModel}
            onDownloadModel={onDownloadModel}
            onCancelDownload={onCancelDownload}
          />
        ))}
      </div>

      <div className="settings-subsection">
        <Button
          variant="ghost"
          size="small"
          onClick={onToggleOtherModels}
          className="btn-hover-scale"
        >
          {showOtherModels ? 'Hide other supported models' : 'Show other supported models'}
        </Button>
        {showOtherModels && (
          <>
            <p className="setting-note">
              These models are supported for experimentation, but may be less reliable for production ticket responses.
            </p>
            <div className="model-list">
              {otherSupportedModels.map((model) => (
                <ModelCard
                  key={model.id}
                  model={model}
                  loadedModel={loadedModel}
                  downloadedModels={downloadedModels}
                  loading={loading}
                  isDownloading={isDownloading}
                  downloadProgress={downloadProgress}
                  onLoadModel={onLoadModel}
                  onUnloadModel={onUnloadModel}
                  onDownloadModel={onDownloadModel}
                  onCancelDownload={onCancelDownload}
                />
              ))}
            </div>
          </>
        )}
      </div>

      <div className="custom-model-section">
        <h3>Custom Model</h3>
        <p className="settings-description">
          Load any GGUF-format model file from your computer.
        </p>
        <Button
          variant="secondary"
          onClick={onLoadCustomModel}
          disabled={!!loading || isDownloading}
        >
          {loading === 'custom' ? 'Loading...' : 'Select GGUF File...'}
        </Button>
      </div>

      <div className="custom-model-section">
        <h3>AI Status &amp; Guarantees</h3>
        <p className="settings-description">
          AssistSupport runs AI locally and can operate fully offline. These signals help operators trust what the AI is doing.
        </p>
        <div className="settings-grid">
          <div className="settings-card">
            <h4>Local Guarantees</h4>
            <ul className="settings-list">
              <li><strong>Offline-first:</strong> no cloud AI calls</li>
              <li><strong>Copy gating:</strong> citations required (override logs locally)</li>
              <li><strong>Prompts hidden:</strong> operators cannot edit system prompts</li>
            </ul>
          </div>
          <div className="settings-card">
            <h4>Runtime Status</h4>
            <ul className="settings-list">
              <li><strong>Chat model:</strong> {loadedModel ? loadedModel : 'Not loaded'}</li>
              <li><strong>Embeddings:</strong> {isEmbeddingLoaded ? 'Loaded' : 'Not loaded'}</li>
              <li><strong>KB folder:</strong> {kbFolder ? kbFolder : 'Not set'}</li>
              <li>
                <strong>MemoryKernel:</strong>{' '}
                {memoryKernelPreflight ? memoryKernelPreflight.status : 'Unavailable'}
                {memoryKernelPreflight?.service_contract_version ? ` (svc ${memoryKernelPreflight.service_contract_version})` : ''}
              </li>
            </ul>
            <div className="settings-actions-row">
              <Button
                variant="ghost"
                size="small"
                onClick={onRefreshMemoryKernelStatus}
                disabled={memoryKernelLoading}
              >
                {memoryKernelLoading ? 'Refreshing...' : 'Refresh'}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
