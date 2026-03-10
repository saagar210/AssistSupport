import { Button } from '../../shared/Button';
import type {
  AuditEntry,
  DeploymentHealthSummary,
  IntegrationConfigRecord,
} from '../../../types';
import type { ResponseQualityThresholds } from '../../../features/analytics/qualityThresholds';

type AuditSeverityFilter = 'all' | 'info' | 'warning' | 'error' | 'critical';

interface DeploymentSectionProps {
  deploymentHealth: DeploymentHealthSummary | null;
  deployPreflightChecks: string[];
  deployPreflightRunning: boolean;
  integrations: IntegrationConfigRecord[];
  onRunDeploymentPreflight: () => void;
  onToggleIntegration: (integrationType: string, enabled: boolean) => void;
}

export function DeploymentSection({
  deploymentHealth,
  deployPreflightChecks,
  deployPreflightRunning,
  integrations,
  onRunDeploymentPreflight,
  onToggleIntegration,
}: DeploymentSectionProps) {
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
          onClick={onRunDeploymentPreflight}
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

interface BackupSectionProps {
  backupLoading: 'export' | 'import' | null;
  onExportBackup: () => void;
  onImportBackup: () => void;
}

export function BackupSection({
  backupLoading,
  onExportBackup,
  onImportBackup,
}: BackupSectionProps) {
  return (
    <section className="settings-section">
      <h2>Data Backup</h2>
      <p className="settings-description">
        Export or import your drafts, templates, variables, and settings.
      </p>
      <div className="backup-actions">
        <div className="backup-row">
          <div className="backup-info">
            <strong>Export</strong>
            <span>Save drafts, templates, variables, trees, settings, and KB folder configuration to a ZIP backup.</span>
          </div>
          <Button
            variant="secondary"
            size="small"
            onClick={onExportBackup}
            disabled={backupLoading === 'export'}
          >
            {backupLoading === 'export' ? 'Exporting...' : 'Export Data'}
          </Button>
        </div>
        <div className="backup-row">
          <div className="backup-info">
            <strong>Import</strong>
            <span>Restore drafts, templates, variables, trees, settings, and KB folder configuration from a backup ZIP file.</span>
          </div>
          <Button
            variant="secondary"
            size="small"
            onClick={onImportBackup}
            disabled={backupLoading === 'import'}
          >
            {backupLoading === 'import' ? 'Importing...' : 'Import Data'}
          </Button>
        </div>
      </div>
    </section>
  );
}

interface QualityThresholdSectionProps {
  qualityThresholds: ResponseQualityThresholds;
  qualityThresholdError: string | null;
  onThresholdChange: (field: keyof ResponseQualityThresholds, value: number) => void;
  onSave: () => void;
  onReset: () => void;
}

export function QualityThresholdSection({
  qualityThresholds,
  qualityThresholdError,
  onThresholdChange,
  onSave,
  onReset,
}: QualityThresholdSectionProps) {
  return (
    <section className="settings-section">
      <h2>Response Quality Coaching</h2>
      <p className="settings-description">
        Tune coaching severity bands used in Analytics. These thresholds are local to this workspace and can be calibrated for your support team.
      </p>
      <div className="quality-threshold-grid">
        <div className="quality-threshold-card">
          <h3>Edit Ratio (%)</h3>
          <div className="quality-threshold-fields">
            <label>
              Watch
              <input
                type="number"
                min={0}
                max={100}
                step={1}
                value={(qualityThresholds.editRatioWatch * 100).toFixed(0)}
                onChange={(e) => onThresholdChange('editRatioWatch', Number(e.target.value || 0) / 100)}
              />
            </label>
            <label>
              Action
              <input
                type="number"
                min={0}
                max={100}
                step={1}
                value={(qualityThresholds.editRatioAction * 100).toFixed(0)}
                onChange={(e) => onThresholdChange('editRatioAction', Number(e.target.value || 0) / 100)}
              />
            </label>
          </div>
        </div>
        <div className="quality-threshold-card">
          <h3>Time to Draft (seconds)</h3>
          <div className="quality-threshold-fields">
            <label>
              Watch
              <input
                type="number"
                min={1}
                step={5}
                value={Math.round(qualityThresholds.timeToDraftWatchMs / 1000)}
                onChange={(e) => onThresholdChange('timeToDraftWatchMs', Math.max(1, Number(e.target.value || 1)) * 1000)}
              />
            </label>
            <label>
              Action
              <input
                type="number"
                min={1}
                step={5}
                value={Math.round(qualityThresholds.timeToDraftActionMs / 1000)}
                onChange={(e) => onThresholdChange('timeToDraftActionMs', Math.max(1, Number(e.target.value || 1)) * 1000)}
              />
            </label>
          </div>
        </div>
        <div className="quality-threshold-card">
          <h3>Copy per Save (%)</h3>
          <div className="quality-threshold-fields">
            <label>
              Watch
              <input
                type="number"
                min={0}
                max={100}
                step={1}
                value={(qualityThresholds.copyPerSaveWatch * 100).toFixed(0)}
                onChange={(e) => onThresholdChange('copyPerSaveWatch', Number(e.target.value || 0) / 100)}
              />
            </label>
            <label>
              Action
              <input
                type="number"
                min={0}
                max={100}
                step={1}
                value={(qualityThresholds.copyPerSaveAction * 100).toFixed(0)}
                onChange={(e) => onThresholdChange('copyPerSaveAction', Number(e.target.value || 0) / 100)}
              />
            </label>
          </div>
        </div>
        <div className="quality-threshold-card">
          <h3>Edited Save Rate (%)</h3>
          <div className="quality-threshold-fields">
            <label>
              Watch
              <input
                type="number"
                min={0}
                max={100}
                step={1}
                value={(qualityThresholds.editedSaveRateWatch * 100).toFixed(0)}
                onChange={(e) => onThresholdChange('editedSaveRateWatch', Number(e.target.value || 0) / 100)}
              />
            </label>
            <label>
              Action
              <input
                type="number"
                min={0}
                max={100}
                step={1}
                value={(qualityThresholds.editedSaveRateAction * 100).toFixed(0)}
                onChange={(e) => onThresholdChange('editedSaveRateAction', Number(e.target.value || 0) / 100)}
              />
            </label>
          </div>
        </div>
      </div>
      {qualityThresholdError && (
        <div className="settings-error">{qualityThresholdError}</div>
      )}
      <div className="quality-threshold-actions">
        <Button variant="secondary" size="small" onClick={onSave}>
          Save Thresholds
        </Button>
        <Button variant="ghost" size="small" onClick={onReset}>
          Reset Defaults
        </Button>
      </div>
    </section>
  );
}

interface AuditLogsSectionProps {
  auditLoading: boolean;
  auditExporting: boolean;
  auditSeverityFilter: AuditSeverityFilter;
  auditSearchQuery: string;
  filteredAuditEntriesCount: number;
  pagedAuditEntries: AuditEntry[];
  auditPage: number;
  auditTotalPages: number;
  formatAuditEvent: (event: string | Record<string, string>) => string;
  onRefresh: () => void;
  onExport: () => void;
  onSeverityChange: (value: AuditSeverityFilter) => void;
  onSearchQueryChange: (value: string) => void;
  onPreviousPage: () => void;
  onNextPage: () => void;
}

export function AuditLogsSection({
  auditLoading,
  auditExporting,
  auditSeverityFilter,
  auditSearchQuery,
  filteredAuditEntriesCount,
  pagedAuditEntries,
  auditPage,
  auditTotalPages,
  formatAuditEvent,
  onRefresh,
  onExport,
  onSeverityChange,
  onSearchQueryChange,
  onPreviousPage,
  onNextPage,
}: AuditLogsSectionProps) {
  return (
    <section className="settings-section">
      <h2>Audit Logs</h2>
      <p className="settings-description">
        Security and system events recorded locally. Export for review or compliance.
      </p>
      <div className="audit-actions">
        <Button
          variant="secondary"
          size="small"
          onClick={onRefresh}
          disabled={auditLoading}
        >
          {auditLoading ? 'Refreshing...' : 'Refresh'}
        </Button>
        <Button
          variant="secondary"
          size="small"
          onClick={onExport}
          disabled={auditExporting}
        >
          {auditExporting ? 'Exporting...' : 'Export JSON'}
        </Button>
      </div>
      <div className="audit-filters">
        <label className="audit-filter-label">
          Severity
          <select
            aria-label="Audit severity filter"
            value={auditSeverityFilter}
            onChange={(e) => onSeverityChange(e.target.value as AuditSeverityFilter)}
          >
            <option value="all">All</option>
            <option value="info">Info</option>
            <option value="warning">Warning</option>
            <option value="error">Error</option>
            <option value="critical">Critical</option>
          </select>
        </label>
        <input
          className="input"
          value={auditSearchQuery}
          onChange={(e) => onSearchQueryChange(e.target.value)}
          placeholder="Search event/message"
        />
      </div>
      <div className="audit-list">
        {pagedAuditEntries.length === 0 ? (
          <p className="audit-empty">No audit entries yet.</p>
        ) : (
          pagedAuditEntries.map((entry, index) => (
            <div className="audit-row" key={`${entry.timestamp}-${index}`}>
              <span className={`audit-severity ${entry.severity}`}>{entry.severity}</span>
              <span className="audit-event">{formatAuditEvent(entry.event)}</span>
              <span className="audit-message">{entry.message}</span>
              <span className="audit-time">{new Date(entry.timestamp).toLocaleString()}</span>
            </div>
          ))
        )}
      </div>
      <div className="audit-pagination">
        <span className="text-sm text-secondary">
          {filteredAuditEntriesCount} entries • Page {auditPage} of {auditTotalPages}
        </span>
        <div className="audit-pagination-actions">
          <Button
            variant="secondary"
            size="small"
            onClick={onPreviousPage}
            disabled={auditPage <= 1}
          >
            Previous
          </Button>
          <Button
            variant="secondary"
            size="small"
            onClick={onNextPage}
            disabled={auditPage >= auditTotalPages}
          >
            Next
          </Button>
        </div>
      </div>
    </section>
  );
}
