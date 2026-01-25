import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  SystemHealth,
  RepairResult,
  FailureMode,
  QuickHealthResult,
} from '../types';

export interface DiagnosticsState {
  systemHealth: SystemHealth | null;
  quickHealth: QuickHealthResult | null;
  failureModes: FailureMode[];
  loading: boolean;
  error: string | null;
}

export function useDiagnostics() {
  const [state, setState] = useState<DiagnosticsState>({
    systemHealth: null,
    quickHealth: null,
    failureModes: [],
    loading: false,
    error: null,
  });

  // Get comprehensive system health
  const getSystemHealth = useCallback(async (): Promise<SystemHealth> => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      const health = await invoke<SystemHealth>('get_system_health');
      setState(prev => ({ ...prev, systemHealth: health, loading: false }));
      return health;
    } catch (e) {
      const error = String(e);
      setState(prev => ({ ...prev, loading: false, error }));
      throw e;
    }
  }, []);

  // Run a quick health check
  const runQuickHealthCheck = useCallback(async (): Promise<QuickHealthResult> => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      const result = await invoke<QuickHealthResult>('run_quick_health_check');
      setState(prev => ({ ...prev, quickHealth: result, loading: false }));
      return result;
    } catch (e) {
      const error = String(e);
      setState(prev => ({ ...prev, loading: false, error }));
      throw e;
    }
  }, []);

  // Repair database
  const repairDatabase = useCallback(async (): Promise<RepairResult> => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      const result = await invoke<RepairResult>('repair_database_cmd');
      setState(prev => ({ ...prev, loading: false }));
      return result;
    } catch (e) {
      const error = String(e);
      setState(prev => ({ ...prev, loading: false, error }));
      throw e;
    }
  }, []);

  // Get vector store rebuild guidance
  const getVectorRebuildGuidance = useCallback(async (): Promise<RepairResult> => {
    try {
      return await invoke<RepairResult>('rebuild_vector_store');
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Get known failure modes
  const getFailureModes = useCallback(async (): Promise<FailureMode[]> => {
    try {
      const modes = await invoke<FailureMode[]>('get_failure_modes_cmd');
      setState(prev => ({ ...prev, failureModes: modes }));
      return modes;
    } catch (e) {
      setState(prev => ({ ...prev, error: String(e) }));
      throw e;
    }
  }, []);

  // Refresh all diagnostics
  const refreshAll = useCallback(async () => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      const [health, quick, modes] = await Promise.all([
        invoke<SystemHealth>('get_system_health'),
        invoke<QuickHealthResult>('run_quick_health_check'),
        invoke<FailureMode[]>('get_failure_modes_cmd'),
      ]);
      setState({
        systemHealth: health,
        quickHealth: quick,
        failureModes: modes,
        loading: false,
        error: null,
      });
    } catch (e) {
      setState(prev => ({ ...prev, loading: false, error: String(e) }));
    }
  }, []);

  return {
    ...state,
    getSystemHealth,
    runQuickHealthCheck,
    repairDatabase,
    getVectorRebuildGuidance,
    getFailureModes,
    refreshAll,
  };
}
