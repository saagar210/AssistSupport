import { Button } from '../../shared/Button';
import type { MemoryKernelPreflightStatus } from '../../../types';

interface SettingsHeroProps {
  loadedModel: string | null;
  kbFolder: string | null;
  isEmbeddingLoaded: boolean;
  embeddingDownloaded: boolean;
  memoryKernelPreflight: MemoryKernelPreflightStatus | null;
}

export function SettingsHero({
  loadedModel,
  kbFolder,
  isEmbeddingLoaded,
  embeddingDownloaded,
  memoryKernelPreflight,
}: SettingsHeroProps) {
  return (
    <header className="settings-hero" aria-label="Settings overview">
      <div className="settings-hero__title">
        <h1>Operator console</h1>
        <p className="settings-hero__sub">
          Local-only configuration and health checks. Offline-first by default.
        </p>
      </div>
      <div className="settings-hero__pills" aria-label="System readiness summary">
        <span className={['settings-pill', loadedModel ? 'is-good' : 'is-warn'].join(' ')}>
          LLM: {loadedModel ? 'Loaded' : 'Not loaded'}
        </span>
        <span className={['settings-pill', kbFolder ? 'is-good' : 'is-warn'].join(' ')}>
          KB: {kbFolder ? 'Set' : 'Not set'}
        </span>
        <span className={['settings-pill', isEmbeddingLoaded ? 'is-good' : 'is-warn'].join(' ')}>
          Embeddings: {isEmbeddingLoaded ? 'Loaded' : embeddingDownloaded ? 'Downloaded' : 'Not downloaded'}
        </span>
        <span
          className={[
            'settings-pill',
            memoryKernelPreflight?.status === 'ready' ? 'is-good' : 'is-warn',
          ].join(' ')}
        >
          MemoryKernel: {memoryKernelPreflight ? memoryKernelPreflight.status : 'Unavailable'}
        </span>
      </div>
    </header>
  );
}

interface PolicyGatesSectionProps {
  adminTabsEnabled: boolean;
  networkIngestEnabled: boolean;
}

export function PolicyGatesSection({
  adminTabsEnabled,
  networkIngestEnabled,
}: PolicyGatesSectionProps) {
  return (
    <section className="settings-section" aria-label="Policy gates">
      <h2>Policy Gates</h2>
      <p className="settings-description">
        These switches control whether potentially sensitive UI surfaces can appear. Outside development builds,
        policy flags are environment-variable authoritative (local overrides are ignored).
      </p>
      <div className="settings-grid">
        <div className="settings-card">
          <h4>Admin Tabs</h4>
          <ul className="settings-list">
            <li>
              <strong>Effective (UI):</strong>{' '}
              {adminTabsEnabled ? 'Enabled' : 'Disabled'}
            </li>
            <li>
              <strong>Enable:</strong> set <code>VITE_ASSISTSUPPORT_ENABLE_ADMIN_TABS=1</code>
            </li>
            <li>
              <strong>Default:</strong> disabled
            </li>
          </ul>
        </div>
        <div className="settings-card">
          <h4>Network Ingest</h4>
          <ul className="settings-list">
            <li>
              <strong>Effective (UI):</strong>{' '}
              {networkIngestEnabled ? 'Enabled' : 'Disabled'}
            </li>
            <li>
              <strong>Enable (UI):</strong> set <code>VITE_ASSISTSUPPORT_ENABLE_NETWORK_INGEST=1</code>
            </li>
            <li>
              <strong>Enable (backend):</strong> set <code>ASSISTSUPPORT_ENABLE_NETWORK_INGEST=1</code>
            </li>
            <li>
              <strong>Default:</strong> disabled
            </li>
          </ul>
        </div>
      </div>
    </section>
  );
}

interface MemoryKernelSectionProps {
  memoryKernelPreflight: MemoryKernelPreflightStatus | null;
  memoryKernelLoading: boolean;
  onRefresh: () => void;
}

export function MemoryKernelSection({
  memoryKernelPreflight,
  memoryKernelLoading,
  onRefresh,
}: MemoryKernelSectionProps) {
  return (
    <section className="settings-section" aria-label="MemoryKernel integration">
      <h2>MemoryKernel</h2>
      <p className="settings-description">
        Optional local enrichment. If unavailable, AssistSupport keeps running with deterministic fallback and no runtime cutover.
      </p>
      <div className="settings-grid">
        <div className="settings-card">
          <h4>Integration Status</h4>
          <ul className="settings-list">
            <li><strong>Enabled:</strong> {memoryKernelPreflight?.enabled ? 'Yes' : 'No'}</li>
            <li><strong>Ready:</strong> {memoryKernelPreflight?.ready ? 'Yes' : 'No'}</li>
            <li><strong>Enrichment:</strong> {memoryKernelPreflight?.enrichment_enabled ? 'Enabled' : 'Disabled'}</li>
            <li><strong>Base URL:</strong> {memoryKernelPreflight?.base_url ? <code>{memoryKernelPreflight.base_url}</code> : 'Unavailable'}</li>
          </ul>
          <div className="settings-actions-row">
            <Button
              variant="ghost"
              size="small"
              onClick={onRefresh}
              disabled={memoryKernelLoading}
            >
              {memoryKernelLoading ? 'Refreshing...' : 'Refresh'}
            </Button>
          </div>
        </div>
        <div className="settings-card">
          <h4>Contract Pins</h4>
          <ul className="settings-list">
            <li>
              <strong>Release:</strong>{' '}
              {memoryKernelPreflight ? (
                <>
                  <code>{memoryKernelPreflight.release_tag}</code> · <code>{memoryKernelPreflight.commit_sha}</code>
                </>
              ) : (
                'Unavailable'
              )}
            </li>
            <li>
              <strong>Service contract:</strong>{' '}
              {memoryKernelPreflight?.service_contract_version ? (
                <code>{memoryKernelPreflight.service_contract_version}</code>
              ) : (
                'Unavailable'
              )}
              {' '}
              (expected <code>{memoryKernelPreflight?.expected_service_contract_version ?? '—'}</code>)
            </li>
            <li>
              <strong>API contract:</strong>{' '}
              {memoryKernelPreflight?.api_contract_version ? (
                <code>{memoryKernelPreflight.api_contract_version}</code>
              ) : (
                'Unavailable'
              )}
              {' '}
              (expected <code>{memoryKernelPreflight?.expected_api_contract_version ?? '—'}</code>)
            </li>
            <li>
              <strong>Baseline:</strong>{' '}
              {memoryKernelPreflight ? <code>{memoryKernelPreflight.integration_baseline}</code> : 'Unavailable'}
            </li>
          </ul>
        </div>
      </div>
    </section>
  );
}

interface AppearanceSectionProps {
  theme: 'light' | 'dark' | 'system';
  onThemeChange: (theme: 'light' | 'dark' | 'system') => void;
}

export function AppearanceSection({ theme, onThemeChange }: AppearanceSectionProps) {
  return (
    <section className="settings-section">
      <h2>Appearance</h2>
      <p className="settings-description">
        Choose your preferred color theme.
      </p>
      <div className="theme-selector">
        <label className="theme-option">
          <input
            type="radio"
            name="theme"
            value="light"
            checked={theme === 'light'}
            onChange={() => onThemeChange('light')}
          />
          <span>Light</span>
        </label>
        <label className="theme-option">
          <input
            type="radio"
            name="theme"
            value="dark"
            checked={theme === 'dark'}
            onChange={() => onThemeChange('dark')}
          />
          <span>Dark</span>
        </label>
        <label className="theme-option">
          <input
            type="radio"
            name="theme"
            value="system"
            checked={theme === 'system'}
            onChange={() => onThemeChange('system')}
          />
          <span>System</span>
        </label>
      </div>
    </section>
  );
}

interface AboutSectionProps {
  versionLabel: string;
}

export function AboutSection({ versionLabel }: AboutSectionProps) {
  return (
    <section className="settings-section">
      <h2>About</h2>
      <p className="settings-description">
        AssistSupport - Local AI-powered support ticket assistant
      </p>
      <div className="about-info">
        <p>{versionLabel}</p>
        <p>All processing happens locally on your machine.</p>
      </div>
    </section>
  );
}
