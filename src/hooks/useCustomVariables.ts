import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { CustomVariable } from '../types';

export function useCustomVariables() {
  const [variables, setVariables] = useState<CustomVariable[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadVariables = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<CustomVariable[]>('list_custom_variables');
      setVariables(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  const getVariable = useCallback(async (variableId: string): Promise<CustomVariable | null> => {
    try {
      return await invoke<CustomVariable>('get_custom_variable', { variableId });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return null;
    }
  }, []);

  const saveVariable = useCallback(async (
    name: string,
    value: string,
    existingId?: string
  ): Promise<boolean> => {
    try {
      const now = new Date().toISOString();
      const variable: CustomVariable = {
        id: existingId || crypto.randomUUID(),
        name,
        value,
        created_at: now,
      };
      await invoke('save_custom_variable', { variable });
      await loadVariables();
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return false;
    }
  }, [loadVariables]);

  const deleteVariable = useCallback(async (variableId: string): Promise<boolean> => {
    try {
      await invoke('delete_custom_variable', { variableId });
      await loadVariables();
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return false;
    }
  }, [loadVariables]);

  return {
    variables,
    loading,
    error,
    loadVariables,
    getVariable,
    saveVariable,
    deleteVariable,
  };
}
