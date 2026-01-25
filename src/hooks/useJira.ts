import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface JiraConfig {
  base_url: string;
  email: string;
}

export interface JiraTicket {
  key: string;
  summary: string;
  description: string | null;
  status: string;
  priority: string | null;
  assignee: string | null;
  reporter: string;
  created: string;
  updated: string;
  issue_type: string;
}

export function useJira() {
  const [configured, setConfigured] = useState<boolean | null>(null);
  const [config, setConfig] = useState<JiraConfig | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkConfiguration = useCallback(async (): Promise<boolean> => {
    try {
      const result = await invoke<boolean>('is_jira_configured');
      setConfigured(result);
      if (result) {
        const cfg = await invoke<JiraConfig | null>('get_jira_config');
        setConfig(cfg);
      }
      return result;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return false;
    }
  }, []);

  const configure = useCallback(async (
    baseUrl: string,
    email: string,
    apiToken: string
  ): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await invoke('configure_jira', { baseUrl, email, apiToken });
      setConfigured(true);
      setConfig({ base_url: baseUrl, email });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const disconnect = useCallback(async (): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await invoke('clear_jira_config');
      setConfigured(false);
      setConfig(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const getTicket = useCallback(async (ticketKey: string): Promise<JiraTicket> => {
    setLoading(true);
    setError(null);
    try {
      const ticket = await invoke<JiraTicket>('get_jira_ticket', { ticketKey });
      return ticket;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    configured,
    config,
    loading,
    error,
    checkConfiguration,
    configure,
    disconnect,
    getTicket,
  };
}
