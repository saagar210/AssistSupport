import { useCallback, useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AuditEntry } from '../../../types';

const AUDIT_PAGE_SIZE = 50;

function formatAuditEvent(event: string | Record<string, string>): string {
  if (typeof event === 'string') return event;
  if (typeof event === 'object' && event !== null) {
    const key = Object.keys(event)[0];
    return key ? `${key}: ${event[key]}` : JSON.stringify(event);
  }
  return String(event);
}

interface UseSettingsAuditLogsParams {
  showSuccess: (message: string) => void;
  showError: (message: string) => void;
}

export function useSettingsAuditLogs({ showSuccess, showError }: UseSettingsAuditLogsParams) {
  const [auditEntries, setAuditEntries] = useState<AuditEntry[]>([]);
  const [auditLoading, setAuditLoading] = useState(false);
  const [auditExporting, setAuditExporting] = useState(false);
  const [auditSeverityFilter, setAuditSeverityFilter] = useState<'all' | 'info' | 'warning' | 'error' | 'critical'>('all');
  const [auditSearchQuery, setAuditSearchQuery] = useState('');
  const [auditPage, setAuditPage] = useState(1);

  const loadAuditEntries = useCallback(async () => {
    setAuditLoading(true);
    try {
      const entries = await invoke<AuditEntry[]>('get_audit_entries', { limit: 200 });
      setAuditEntries(entries ?? []);
      setAuditPage(1);
    } catch (err) {
      showError(`Failed to load audit logs: ${err}`);
    } finally {
      setAuditLoading(false);
    }
  }, [showError]);

  const handleExportAuditLog = useCallback(async () => {
    setAuditExporting(true);
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const path = await save({
        title: 'Export Audit Log',
        defaultPath: 'assist-support-audit.json',
        filters: [{ name: 'JSON', extensions: ['json'] }],
      });
      if (!path) {
        setAuditExporting(false);
        return;
      }
      const output = await invoke<string>('export_audit_log', { exportPath: path });
      showSuccess(`Audit log exported to ${output}`);
    } catch (err) {
      if (String(err) !== 'Export cancelled') {
        showError(`Audit export failed: ${err}`);
      }
    } finally {
      setAuditExporting(false);
    }
  }, [showSuccess, showError]);

  const filteredAuditEntries = useMemo(() => {
    const normalized = auditEntries.slice().reverse();
    const query = auditSearchQuery.trim().toLowerCase();
    return normalized.filter((entry) => {
      if (auditSeverityFilter !== 'all' && entry.severity !== auditSeverityFilter) {
        return false;
      }
      if (!query) {
        return true;
      }
      const eventText = formatAuditEvent(entry.event).toLowerCase();
      return eventText.includes(query) || entry.message.toLowerCase().includes(query);
    });
  }, [auditEntries, auditSearchQuery, auditSeverityFilter]);

  const auditTotalPages = useMemo(
    () => Math.max(1, Math.ceil(filteredAuditEntries.length / AUDIT_PAGE_SIZE)),
    [filteredAuditEntries.length],
  );

  useEffect(() => {
    setAuditPage(prev => Math.min(prev, auditTotalPages));
  }, [auditTotalPages]);

  const pagedAuditEntries = useMemo(() => {
    const start = (auditPage - 1) * AUDIT_PAGE_SIZE;
    return filteredAuditEntries.slice(start, start + AUDIT_PAGE_SIZE);
  }, [filteredAuditEntries, auditPage]);

  const setSeverityFilter = useCallback((severity: 'all' | 'info' | 'warning' | 'error' | 'critical') => {
    setAuditSeverityFilter(severity);
    setAuditPage(1);
  }, []);

  const setSearchQuery = useCallback((query: string) => {
    setAuditSearchQuery(query);
    setAuditPage(1);
  }, []);

  return {
    auditLoading,
    auditExporting,
    auditSeverityFilter,
    auditSearchQuery,
    auditPage,
    auditTotalPages,
    filteredAuditEntries,
    pagedAuditEntries,
    setAuditPage,
    setSeverityFilter,
    setSearchQuery,
    loadAuditEntries,
    handleExportAuditLog,
    formatAuditEvent,
  };
}
