import { Button } from '../../shared/Button';
import type {
  DatabaseStats,
  DeploymentHealthSummary,
  IntegrationConfigRecord,
  MemoryKernelPreflightStatus,
} from '../../../types';

interface SettingsHeroSectionProps {
  loadedModel: string | null;
  kbFolder: string | null;
  isEmbeddingLoaded: boolean;
  embeddingDownloaded: boolean;
  memoryKernelPreflight: MemoryKernelPreflightStatus | null;
}

export function SettingsHeroSection({
  loadedModel,
  kbFolder,
  isEmbeddingLoaded,
  embeddingDownloaded,
  memoryKernelPreflight,
}: SettingsHeroSectionProps) {
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
  revampFlags: {
    ASSISTSUPPORT_ENABLE_ADMIN_TABS: boolean;
    ASSISTSUPPORT_ENABLE_NETWORK_INGEST: boolean;
  };
}

export function PolicyGatesSection({ revampFlags }: PolicyGatesSectionProps) {
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
              {revampFlags.ASSISTSUPPORT_ENABLE_ADMIN_TABS ? 'Enabled' : 'Disabled'}
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
              {revampFlags.ASSISTSUPPORT_ENABLE_NETWORK_INGEST ? 'Enabled' : 'Disabled'}
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
  setTheme: (theme: 'light' | 'dark' | 'system') => void;
}

export function AppearanceSection({ theme, setTheme }: AppearanceSectionProps) {
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
            onChange={() => setTheme('light')}
          />
          <span>Light</span>
        </label>
        <label className="theme-option">
          <input
            type="radio"
            name="theme"
            value="dark"
            checked={theme === 'dark'}
            onChange={() => setTheme('dark')}
          />
          <span>Dark</span>
        </label>
        <label className="theme-option">
          <input
            type="radio"
            name="theme"
            value="system"
            checked={theme === 'system'}
            onChange={() => setTheme('system')}
          />
          <span>System</span>
        </label>
      </div>
    </section>
  );
}

interface DeploymentIntegrationsSectionProps {
  deploymentHealth: DeploymentHealthSummary | null;
  deployPreflightChecks: string[];
  deployPreflightRunning: boolean;
  integrations: IntegrationConfigRecord[];
  onRunPreflight: () => void;
  onToggleIntegration: (integrationType: string, enabled: boolean) => void;
}

export function DeploymentIntegrationsSection({
  deploymentHealth,
  deployPreflightChecks,
  deployPreflightRunning,
  integrations,
  onRunPreflight,
  onToggleIntegration,
}: DeploymentIntegrationsSectionProps) {
  return (
    <section className="settings-section">
      <h2>Deployment &amp; Integrations</h2>
      <p className="settings-description">
        Deployment health, preflight validation, and integration toggles for ServiceNow/Slack/Teams.
      </p>
      <div className="settings-row">
        <Button
          variant="secondary"
          size="small"
          onClick={onRunPreflight}
          disabled={deployPreflightRunning}
        >
          {deployPreflightRunning ? 'Running preflight...' : 'Run Deployment Preflight'}
        </Button>
      </div>
      {deploymentHealth && (
        <div className="startup-metrics">
          <p className="text-sm text-secondary">
            Signed artifacts: {deploymentHealth.signed_artifacts}/{deploymentHealth.total_artifacts}
          </p>
          {deploymentHealth.last_run && (
            <p className="text-sm text-secondary">
              Last run: {deploymentHealth.last_run.status} ({deploymentHealth.last_run.target_channel})
            </p>
          )}
        </div>
      )}
      {deployPreflightChecks.length > 0 && (
        <ul className="audit-list">
          {deployPreflightChecks.map((check, idx) => (
            <li key={`${check}-${idx}`} className="audit-row">{check}</li>
          ))}
        </ul>
      )}
      <div className="settings-row">
        {['servicenow', 'slack', 'teams'].map(type => {
          const current = integrations.find(i => i.integration_type === type);
          const enabled = current?.enabled ?? false;
          return (
            <label key={type} className="toggle-option">
              <input
                type="checkbox"
                checked={enabled}
                onChange={(e) => onToggleIntegration(type, e.target.checked)}
              />
              <span>{type.charAt(0).toUpperCase() + type.slice(1)}</span>
            </label>
          );
        })}
      </div>
    </section>
  );
}

interface AboutSectionProps {
  appVersionLabel: string;
}

export function AboutSection({ appVersionLabel }: AboutSectionProps) {
  return (
    <section className="settings-section">
      <h2>About</h2>
      <p className="settings-description">
        AssistSupport - Local AI-powered support ticket assistant
      </p>
      <div className="about-info">
        <p>{appVersionLabel}</p>
        <p>All processing happens locally on your machine.</p>
      </div>
    </section>
  );
}

interface MaintenanceDiagnosticsSectionProps {
  databaseStats: DatabaseStats | null;
  databaseStatsLoading: boolean;
  databaseMaintenanceRunning: boolean;
  databaseMaintenanceResult: string | null;
  onRefreshDatabaseStats: () => void;
  onRunDatabaseMaintenance: () => void;
}

function formatDatabaseBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(1)} ${units[unitIndex]}`;
}

function formatCount(value: number): string {
  return Number.isFinite(value) ? value.toLocaleString() : '0';
}

function formatMaybeTimestamp(value: string | null): string {
  if (!value) return 'Never';
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  return parsed.toLocaleString();
}

export function MaintenanceDiagnosticsSection({
  databaseStats,
  databaseStatsLoading,
  databaseMaintenanceRunning,
  databaseMaintenanceResult,
  onRefreshDatabaseStats,
  onRunDatabaseMaintenance,
}: MaintenanceDiagnosticsSectionProps) {
  return (
    <section className="settings-section" aria-label="Database maintenance diagnostics">
      <h2>Maintenance Diagnostics</h2>
      <p className="settings-description">
        SQLite maintenance telemetry for optimize/checkpoint cadence and current WAL posture.
      </p>
      <div className="settings-actions-row">
        <Button
          variant="secondary"
          size="small"
          onClick={onRefreshDatabaseStats}
          disabled={databaseStatsLoading}
        >
          {databaseStatsLoading ? 'Refreshing...' : 'Refresh Stats'}
        </Button>
        <Button
          variant="ghost"
          size="small"
          onClick={onRunDatabaseMaintenance}
          disabled={databaseMaintenanceRunning}
        >
          {databaseMaintenanceRunning ? 'Running maintenance...' : 'Run Maintenance Now'}
        </Button>
      </div>
      {databaseMaintenanceResult && (
        <p className="setting-note">{databaseMaintenanceResult}</p>
      )}
      {databaseStats ? (
        <div className="settings-grid">
          <div className="settings-card">
            <h4>Storage</h4>
            <ul className="settings-list">
              <li><strong>File size:</strong> {formatDatabaseBytes(databaseStats.file_size_bytes)}</li>
              <li><strong>Page count:</strong> {formatCount(databaseStats.page_count)}</li>
              <li><strong>Freelist pages:</strong> {formatCount(databaseStats.freelist_count)}</li>
              <li><strong>Journal mode:</strong> <code>{databaseStats.journal_mode}</code></li>
            </ul>
          </div>
          <div className="settings-card">
            <h4>Maintenance Cadence</h4>
            <ul className="settings-list">
              <li><strong>Interval:</strong> Every {formatCount(databaseStats.maintenance_interval_hours)}h</li>
              <li><strong>Last maintenance:</strong> {formatMaybeTimestamp(databaseStats.last_maintenance_at)}</li>
              <li><strong>Last optimize:</strong> {formatMaybeTimestamp(databaseStats.last_optimize)}</li>
              <li><strong>Last WAL checkpoint:</strong> {formatMaybeTimestamp(databaseStats.last_wal_checkpoint)}</li>
              <li><strong>Last vacuum:</strong> {formatMaybeTimestamp(databaseStats.last_vacuum)}</li>
              <li>
                <strong>Policy status:</strong>{' '}
                {databaseStats.maintenance_due ? 'Due now' : 'Within cadence'} ({databaseStats.maintenance_due_reason})
              </li>
            </ul>
          </div>
          <div className="settings-card">
            <h4>WAL Observability</h4>
            <ul className="settings-list">
              <li><strong>Busy pages:</strong> {formatCount(databaseStats.wal_checkpoint_busy)}</li>
              <li><strong>Log frames:</strong> {formatCount(databaseStats.wal_log_frames)}</li>
              <li><strong>Checkpointed:</strong> {formatCount(databaseStats.wal_checkpointed_frames)}</li>
              <li>
                <strong>Progress:</strong>{' '}
                {databaseStats.wal_log_frames >= databaseStats.wal_checkpointed_frames ? 'Expected' : 'Review'}
              </li>
            </ul>
          </div>
        </div>
      ) : (
        <p className="setting-note">No database stats loaded yet.</p>
      )}
    </section>
  );
}
