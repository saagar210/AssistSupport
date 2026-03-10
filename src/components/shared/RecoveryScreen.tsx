import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { ImportSummary, RepairResult, StartupRecoveryIssue } from '../../types';
import { Button } from './Button';
import { Icon } from './Icon';
import './RecoveryScreen.css';

interface RecoveryScreenProps {
  issue: StartupRecoveryIssue;
}

export function RecoveryScreen({ issue }: RecoveryScreenProps) {
  const [action, setAction] = useState<'repair' | 'restore' | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  async function handleRepair() {
    setAction('repair');
    setError(null);
    setMessage(null);
    try {
      const result = await invoke<RepairResult>('repair_database_cmd');
      setMessage(result.message ?? result.action_taken);
    } catch (err) {
      setError(String(err));
    } finally {
      setAction(null);
    }
  }

  async function handleRestore() {
    setAction('restore');
    setError(null);
    setMessage(null);
    try {
      const result = await invoke<ImportSummary>('import_backup');
      setMessage(
        `Backup restored: ${result.drafts_imported} drafts, ${result.templates_imported} templates, ${result.variables_imported} variables, and ${result.trees_imported} trees imported. Restart AssistSupport to continue.`,
      );
    } catch (err) {
      if (String(err) !== 'Import cancelled') {
        setError(String(err));
      }
    } finally {
      setAction(null);
    }
  }

  return (
    <main className="recovery-screen">
      <section className="recovery-card" aria-labelledby="recovery-title">
        <div className="recovery-header">
          <div className="recovery-icon" aria-hidden="true">
            <Icon name="alertCircle" size={28} />
          </div>
          <div>
            <h1 id="recovery-title">{issue.summary}</h1>
            <p className="recovery-subtitle">
              AssistSupport opened in recovery mode so you can repair the workspace before normal startup continues.
            </p>
          </div>
        </div>

        <div className="recovery-body">
          {issue.details && <p className="recovery-details">{issue.details}</p>}

          <div className="recovery-actions" role="group" aria-label="Recovery actions">
            {issue.can_repair && (
              <Button
                variant="primary"
                onClick={handleRepair}
                disabled={action !== null}
                loading={action === 'repair'}
              >
                Repair Database
              </Button>
            )}
            {issue.can_restore_backup && (
              <Button
                variant="secondary"
                onClick={handleRestore}
                disabled={action !== null}
                loading={action === 'restore'}
              >
                Restore From Backup
              </Button>
            )}
            <Button
              variant="ghost"
              onClick={() => window.location.reload()}
              disabled={action !== null}
            >
              Retry Startup
            </Button>
          </div>

          {message && (
            <div className="recovery-message" role="status">
              <Icon name="checkCircle" size={16} />
              <span>{message}</span>
            </div>
          )}

          {error && (
            <div className="recovery-error" role="alert">
              <Icon name="alertCircle" size={16} />
              <span>{error}</span>
            </div>
          )}

          {issue.migration_conflicts.length > 0 && (
            <div className="recovery-conflicts">
              <h2>Migration Conflicts</h2>
              <ul>
                {issue.migration_conflicts.map((conflict) => (
                  <li key={`${conflict.name}-${conflict.old_path}`}>
                    <strong>{conflict.name}</strong>
                    <span>{conflict.reason}</span>
                    <code>Old: {conflict.old_path}</code>
                    <code>New: {conflict.new_path}</code>
                  </li>
                ))}
              </ul>
            </div>
          )}

          <div className="recovery-notes">
            <h2>What backup restore brings back</h2>
            <ul>
              <li>Drafts, templates, custom variables, custom trees, settings, and knowledge-base folder configuration are restored.</li>
              <li>Knowledge-base source files, attachments, models, and vector embeddings are not included in the backup file.</li>
              <li>After recovery, re-index the knowledge base and regenerate embeddings if you use local semantic search.</li>
            </ul>
          </div>
        </div>
      </section>
    </main>
  );
}
