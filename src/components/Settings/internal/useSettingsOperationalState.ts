import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  DatabaseStats,
  DeploymentHealthSummary,
  IntegrationConfigRecord,
  MemoryKernelPreflightStatus,
  RepairResult,
} from '../../../types';

interface UseSettingsOperationalStateParams {
  getDeploymentHealthSummary: () => Promise<DeploymentHealthSummary | null>;
  runDeploymentPreflight: (targetChannel: string) => Promise<{ ok: boolean; checks: string[] }>;
  listIntegrations: () => Promise<IntegrationConfigRecord[]>;
  configureIntegration: (integrationType: string, enabled: boolean) => Promise<void>;
  showSuccess: (message: string) => void;
  showError: (message: string) => void;
}

export function useSettingsOperationalState({
  getDeploymentHealthSummary,
  runDeploymentPreflight,
  listIntegrations,
  configureIntegration,
  showSuccess,
  showError,
}: UseSettingsOperationalStateParams) {
  const [deploymentHealth, setDeploymentHealth] = useState<DeploymentHealthSummary | null>(null);
  const [deployPreflightChecks, setDeployPreflightChecks] = useState<string[]>([]);
  const [deployPreflightRunning, setDeployPreflightRunning] = useState(false);
  const [integrations, setIntegrations] = useState<IntegrationConfigRecord[]>([]);
  const [memoryKernelPreflight, setMemoryKernelPreflight] = useState<MemoryKernelPreflightStatus | null>(null);
  const [memoryKernelLoading, setMemoryKernelLoading] = useState(false);
  const [databaseStats, setDatabaseStats] = useState<DatabaseStats | null>(null);
  const [databaseStatsLoading, setDatabaseStatsLoading] = useState(false);
  const [databaseMaintenanceRunning, setDatabaseMaintenanceRunning] = useState(false);
  const [databaseMaintenanceResult, setDatabaseMaintenanceResult] = useState<string | null>(null);

  const refreshMemoryKernelStatus = useCallback(async () => {
    setMemoryKernelLoading(true);
    try {
      const status = await invoke<MemoryKernelPreflightStatus>('get_memory_kernel_preflight_status');
      setMemoryKernelPreflight(status);
    } catch {
      // Non-blocking: show as unavailable rather than failing settings load.
      setMemoryKernelPreflight(null);
    } finally {
      setMemoryKernelLoading(false);
    }
  }, []);

  const refreshDatabaseStats = useCallback(async () => {
    setDatabaseStatsLoading(true);
    try {
      const stats = await invoke<DatabaseStats>('get_database_stats_cmd');
      setDatabaseStats(stats);
    } catch (err) {
      showError(`Failed to load database diagnostics: ${err}`);
    } finally {
      setDatabaseStatsLoading(false);
    }
  }, [showError]);

  const handleRunDatabaseMaintenance = useCallback(async () => {
    setDatabaseMaintenanceRunning(true);
    try {
      const result = await invoke<RepairResult>('run_database_maintenance_cmd');
      setDatabaseMaintenanceResult(
        result.success
          ? `Maintenance completed: ${result.action_taken}`
          : `Maintenance reported issues: ${result.action_taken}`,
      );
      if (result.success) {
        showSuccess('Database maintenance completed');
      } else {
        showError('Database maintenance completed with warnings');
      }
      await refreshDatabaseStats();
    } catch (err) {
      showError(`Database maintenance failed: ${err}`);
    } finally {
      setDatabaseMaintenanceRunning(false);
    }
  }, [refreshDatabaseStats, showError, showSuccess]);

  const refreshDeploymentAndIntegrations = useCallback(async () => {
    const [deployHealth, integrationsList] = await Promise.all([
      getDeploymentHealthSummary().catch(() => null),
      listIntegrations().catch(() => []),
    ]);
    setDeploymentHealth(deployHealth);
    setIntegrations(integrationsList ?? []);
  }, [getDeploymentHealthSummary, listIntegrations]);

  const handleRunDeploymentPreflight = useCallback(async () => {
    setDeployPreflightRunning(true);
    try {
      const result = await runDeploymentPreflight('stable');
      setDeployPreflightChecks(result.checks);
      const latest = await getDeploymentHealthSummary().catch(() => null);
      setDeploymentHealth(latest);
      if (result.ok) {
        showSuccess('Deployment preflight passed');
      } else {
        showError('Deployment preflight reported failures');
      }
    } catch (err) {
      showError(`Deployment preflight failed: ${err}`);
    } finally {
      setDeployPreflightRunning(false);
    }
  }, [runDeploymentPreflight, getDeploymentHealthSummary, showSuccess, showError]);

  const handleToggleIntegration = useCallback(async (integrationType: string, enabled: boolean) => {
    try {
      await configureIntegration(integrationType, enabled);
      const updated = await listIntegrations();
      setIntegrations(updated ?? []);
    } catch (err) {
      showError(`Failed to update ${integrationType}: ${err}`);
    }
  }, [configureIntegration, listIntegrations, showError]);

  useEffect(() => {
    void refreshMemoryKernelStatus();
    void refreshDatabaseStats();
  }, [refreshMemoryKernelStatus, refreshDatabaseStats]);

  return {
    deploymentHealth,
    deployPreflightChecks,
    deployPreflightRunning,
    integrations,
    memoryKernelPreflight,
    memoryKernelLoading,
    databaseStats,
    databaseStatsLoading,
    databaseMaintenanceRunning,
    databaseMaintenanceResult,
    refreshMemoryKernelStatus,
    refreshDatabaseStats,
    handleRunDatabaseMaintenance,
    refreshDeploymentAndIntegrations,
    handleRunDeploymentPreflight,
    handleToggleIntegration,
  };
}
