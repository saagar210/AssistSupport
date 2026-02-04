import { useCallback, useEffect, useMemo, useState } from 'react';
import { Button } from '../shared/Button';
import { useToastContext } from '../../contexts/ToastContext';
import {
  useFeatureOps,
  type EvalHarnessCase,
  type TriageTicketInput,
} from '../../hooks/useFeatureOps';
import type {
  DeploymentArtifactRecord,
  DeploymentHealthSummary,
  EvalRunRecord,
  IntegrationConfigRecord,
  RunbookSessionRecord,
  SignedArtifactVerificationResult,
  TriageClusterRecord,
} from '../../types';
import './OpsTab.css';

type OpsView = 'deployment' | 'eval' | 'triage' | 'runbook' | 'integrations';

const INTEGRATION_TYPES = ['servicenow', 'slack', 'teams'] as const;

function parseEvalCases(input: string): EvalHarnessCase[] {
  return input
    .split('\n')
    .map(line => line.trim())
    .filter(Boolean)
    .map(line => {
      const [query, expectedMode, minConfidenceRaw] = line.split('|').map(v => v?.trim());
      const minConfidence = minConfidenceRaw ? Number(minConfidenceRaw) : undefined;
      return {
        query,
        expected_mode: expectedMode || undefined,
        min_confidence: Number.isFinite(minConfidence) ? minConfidence : undefined,
      };
    })
    .filter(c => !!c.query);
}

function parseTriageTickets(input: string): TriageTicketInput[] {
  return input
    .split('\n')
    .map(line => line.trim())
    .filter(Boolean)
    .map((line, idx) => {
      const [id, summary] = line.split('|').map(v => v?.trim());
      if (summary) {
        return { id: id || `ticket-${idx + 1}`, summary };
      }
      return { id: `ticket-${idx + 1}`, summary: line };
    });
}

function safeParseJson<T>(raw: string, fallback: T): T {
  try {
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

export function OpsTab() {
  const { success: showSuccess, error: showError } = useToastContext();
  const {
    getDeploymentHealthSummary,
    runDeploymentPreflight,
    listDeploymentArtifacts,
    recordDeploymentArtifact,
    verifySignedArtifact,
    rollbackDeploymentRun,
    runEvalHarness,
    listEvalRuns,
    clusterTicketsForTriage,
    listRecentTriageClusters,
    startRunbookSession,
    advanceRunbookSession,
    listRunbookSessions,
    listIntegrations,
    configureIntegration,
  } = useFeatureOps();

  const [view, setView] = useState<OpsView>('deployment');

  // Deployment state
  const [deploymentHealth, setDeploymentHealth] = useState<DeploymentHealthSummary | null>(null);
  const [preflightChecks, setPreflightChecks] = useState<string[]>([]);
  const [artifacts, setArtifacts] = useState<DeploymentArtifactRecord[]>([]);
  const [verification, setVerification] = useState<SignedArtifactVerificationResult | null>(null);
  const [deployBusy, setDeployBusy] = useState(false);
  const [artifactForm, setArtifactForm] = useState({
    artifactType: 'app_bundle',
    version: '1.0.0',
    channel: 'stable',
    sha256: '',
    isSigned: true,
  });
  const [rollbackReason, setRollbackReason] = useState('Release validation failure');
  const [showRollbackConfirm, setShowRollbackConfirm] = useState(false);

  // Eval state
  const [evalSuiteName, setEvalSuiteName] = useState('ops-regression-suite');
  const [evalInput, setEvalInput] = useState('Can I use a flash drive?|answer|0.6\nUrgent outage on VPN|clarify|0.5');
  const [evalResult, setEvalResult] = useState<string>('');
  const [evalRuns, setEvalRuns] = useState<EvalRunRecord[]>([]);
  const [evalBusy, setEvalBusy] = useState(false);

  // Triage state
  const [triageInput, setTriageInput] = useState('INC-1001|VPN disconnects every morning\nINC-1002|VPN timeout when connecting');
  const [triageOutput, setTriageOutput] = useState<string>('');
  const [triageHistory, setTriageHistory] = useState<TriageClusterRecord[]>([]);
  const [triageBusy, setTriageBusy] = useState(false);

  // Runbook state
  const [runbookScenario, setRunbookScenario] = useState('security-incident');
  const [runbookStepsInput, setRunbookStepsInput] = useState('Acknowledge incident\nCollect impact details\nContain affected access\nNotify stakeholders');
  const [runbookSessions, setRunbookSessions] = useState<RunbookSessionRecord[]>([]);
  const [runbookBusy, setRunbookBusy] = useState(false);

  // Integration state
  const [integrations, setIntegrations] = useState<IntegrationConfigRecord[]>([]);
  const [integrationConfigDraft, setIntegrationConfigDraft] = useState<Record<string, string>>({});
  const [integrationBusyType, setIntegrationBusyType] = useState<string | null>(null);

  const refreshDeployment = useCallback(async () => {
    const [health, deploymentArtifacts] = await Promise.all([
      getDeploymentHealthSummary().catch(() => null),
      listDeploymentArtifacts(50).catch(() => []),
    ]);
    setDeploymentHealth(health);
    setArtifacts(deploymentArtifacts);
  }, [getDeploymentHealthSummary, listDeploymentArtifacts]);

  const refreshEval = useCallback(async () => {
    const runs = await listEvalRuns(50).catch(() => []);
    setEvalRuns(runs);
  }, [listEvalRuns]);

  const refreshTriage = useCallback(async () => {
    const history = await listRecentTriageClusters(50).catch(() => []);
    setTriageHistory(history);
  }, [listRecentTriageClusters]);

  const refreshRunbooks = useCallback(async () => {
    const sessions = await listRunbookSessions(50).catch(() => []);
    setRunbookSessions(sessions);
  }, [listRunbookSessions]);

  const refreshIntegrations = useCallback(async () => {
    const next = await listIntegrations().catch(() => []);
    setIntegrations(next);
    const nextDraft: Record<string, string> = {};
    for (const item of next) {
      nextDraft[item.integration_type] = item.config_json ?? '';
    }
    setIntegrationConfigDraft(nextDraft);
  }, [listIntegrations]);

  useEffect(() => {
    void Promise.all([
      refreshDeployment(),
      refreshEval(),
      refreshTriage(),
      refreshRunbooks(),
      refreshIntegrations(),
    ]);
  }, [refreshDeployment, refreshEval, refreshTriage, refreshRunbooks, refreshIntegrations]);

  const runDeploymentChecks = useCallback(async () => {
    setDeployBusy(true);
    try {
      const result = await runDeploymentPreflight('stable');
      setPreflightChecks(result.checks);
      await refreshDeployment();
      if (result.ok) {
        showSuccess('Deployment preflight passed');
      } else {
        showError('Deployment preflight reported failures');
      }
    } catch (e) {
      showError(`Failed to run deployment preflight: ${e}`);
    } finally {
      setDeployBusy(false);
    }
  }, [runDeploymentPreflight, refreshDeployment, showSuccess, showError]);

  const submitArtifact = useCallback(async () => {
    if (!artifactForm.sha256.trim()) return;
    setDeployBusy(true);
    try {
      await recordDeploymentArtifact(
        artifactForm.artifactType,
        artifactForm.version,
        artifactForm.channel,
        artifactForm.sha256,
        artifactForm.isSigned,
      );
      setArtifactForm(prev => ({ ...prev, sha256: '' }));
      await refreshDeployment();
      showSuccess('Deployment artifact recorded');
    } catch (e) {
      showError(`Failed to record artifact: ${e}`);
    } finally {
      setDeployBusy(false);
    }
  }, [artifactForm, recordDeploymentArtifact, refreshDeployment, showSuccess, showError]);

  const verifyArtifact = useCallback(async (artifactId: string) => {
    setDeployBusy(true);
    try {
      const result = await verifySignedArtifact(artifactId);
      setVerification(result);
      if (result.status === 'verified') {
        showSuccess('Artifact verification passed');
      } else {
        showError(`Verification result: ${result.status}`);
      }
    } catch (e) {
      showError(`Failed to verify artifact: ${e}`);
    } finally {
      setDeployBusy(false);
    }
  }, [verifySignedArtifact, showSuccess, showError]);

  const runRollback = useCallback(async () => {
    if (!deploymentHealth?.last_run?.id) return;
    setDeployBusy(true);
    try {
      await rollbackDeploymentRun(deploymentHealth.last_run.id, rollbackReason);
      await refreshDeployment();
      showSuccess('Rollback marked successfully');
    } catch (e) {
      showError(`Rollback failed: ${e}`);
    } finally {
      setDeployBusy(false);
      setShowRollbackConfirm(false);
    }
  }, [deploymentHealth?.last_run?.id, rollbackReason, rollbackDeploymentRun, refreshDeployment, showSuccess, showError]);

  const runEval = useCallback(async () => {
    setEvalBusy(true);
    try {
      const parsedCases = parseEvalCases(evalInput);
      if (parsedCases.length === 0) {
        showError('Add at least one eval case');
        return;
      }
      const result = await runEvalHarness(evalSuiteName, parsedCases);
      setEvalResult(`Run ${result.run_id}: ${result.passed_cases}/${result.total_cases} passed, avg confidence ${(result.avg_confidence * 100).toFixed(1)}%`);
      await refreshEval();
      showSuccess('Eval harness run completed');
    } catch (e) {
      showError(`Eval run failed: ${e}`);
    } finally {
      setEvalBusy(false);
    }
  }, [evalInput, evalSuiteName, runEvalHarness, refreshEval, showSuccess, showError]);

  const runTriage = useCallback(async () => {
    setTriageBusy(true);
    try {
      const tickets = parseTriageTickets(triageInput);
      if (tickets.length === 0) {
        showError('Add at least one ticket for clustering');
        return;
      }
      const clusters = await clusterTicketsForTriage(tickets);
      setTriageOutput(JSON.stringify(clusters, null, 2));
      await refreshTriage();
      showSuccess('Ticket clustering complete');
    } catch (e) {
      showError(`Clustering failed: ${e}`);
    } finally {
      setTriageBusy(false);
    }
  }, [triageInput, clusterTicketsForTriage, refreshTriage, showSuccess, showError]);

  const startRunbook = useCallback(async () => {
    setRunbookBusy(true);
    try {
      const steps = runbookStepsInput
        .split('\n')
        .map(s => s.trim())
        .filter(Boolean);
      if (steps.length === 0) {
        showError('Add at least one runbook step');
        return;
      }
      await startRunbookSession(runbookScenario, steps);
      await refreshRunbooks();
      showSuccess('Runbook session started');
    } catch (e) {
      showError(`Failed to start runbook: ${e}`);
    } finally {
      setRunbookBusy(false);
    }
  }, [runbookStepsInput, runbookScenario, startRunbookSession, refreshRunbooks, showSuccess, showError]);

  const advanceRunbook = useCallback(async (session: RunbookSessionRecord, finish = false) => {
    const steps = safeParseJson<string[]>(session.steps_json, []);
    const nextStep = finish ? session.current_step : session.current_step + 1;
    const status = finish || nextStep >= steps.length ? 'completed' : undefined;
    setRunbookBusy(true);
    try {
      await advanceRunbookSession(session.id, nextStep, status);
      await refreshRunbooks();
      showSuccess(status === 'completed' ? 'Runbook marked complete' : 'Runbook advanced');
    } catch (e) {
      showError(`Failed to update runbook: ${e}`);
    } finally {
      setRunbookBusy(false);
    }
  }, [advanceRunbookSession, refreshRunbooks, showSuccess, showError]);

  const saveIntegration = useCallback(async (integrationType: string, enabled: boolean) => {
    setIntegrationBusyType(integrationType);
    try {
      await configureIntegration(integrationType, enabled, integrationConfigDraft[integrationType] || undefined);
      await refreshIntegrations();
      showSuccess(`${integrationType} integration updated`);
    } catch (e) {
      showError(`Failed to update integration: ${e}`);
    } finally {
      setIntegrationBusyType(null);
    }
  }, [configureIntegration, integrationConfigDraft, refreshIntegrations, showSuccess, showError]);

  const integrationMap = useMemo(() => {
    const map = new Map<string, IntegrationConfigRecord>();
    for (const i of integrations) map.set(i.integration_type, i);
    return map;
  }, [integrations]);

  return (
    <div className="ops-tab">
      <div className="ops-nav">
        {([
          ['deployment', 'Deployment'],
          ['eval', 'Eval Harness'],
          ['triage', 'Triage'],
          ['runbook', 'Runbook'],
          ['integrations', 'Integrations'],
        ] as [OpsView, string][]).map(([id, label]) => (
          <button
            key={id}
            className={`ops-nav-btn ${view === id ? 'active' : ''}`}
            onClick={() => setView(id)}
          >
            {label}
          </button>
        ))}
      </div>

      {view === 'deployment' && (
        <section className="ops-section">
          <h3>Deployment Rollback & Signed Pack Verification</h3>
          <div className="ops-row">
            <Button variant="secondary" size="small" onClick={runDeploymentChecks} loading={deployBusy}>Run Preflight</Button>
            <Button
              variant="danger"
              size="small"
              onClick={() => setShowRollbackConfirm(true)}
              disabled={!deploymentHealth?.last_run || deploymentHealth.last_run.status === 'rolled_back'}
              loading={deployBusy}
            >
              Roll Back Last Run
            </Button>
            <input
              className="ops-input"
              value={rollbackReason}
              onChange={e => setRollbackReason(e.target.value)}
              placeholder="Rollback reason"
            />
          </div>

          {deploymentHealth && (
            <div className="ops-kpis">
              <div>Artifacts: {deploymentHealth.total_artifacts}</div>
              <div>Signed: {deploymentHealth.signed_artifacts}</div>
              <div>Unsigned: {deploymentHealth.unsigned_artifacts}</div>
              <div>Last status: {deploymentHealth.last_run?.status ?? 'none'}</div>
            </div>
          )}

          {preflightChecks.length > 0 && (
            <ul className="ops-list">
              {preflightChecks.map((check, i) => <li key={`${check}-${i}`}>{check}</li>)}
            </ul>
          )}

          <div className="ops-grid">
            <input className="ops-input" value={artifactForm.artifactType} onChange={e => setArtifactForm(p => ({ ...p, artifactType: e.target.value }))} placeholder="artifact type" />
            <input className="ops-input" value={artifactForm.version} onChange={e => setArtifactForm(p => ({ ...p, version: e.target.value }))} placeholder="version" />
            <input className="ops-input" value={artifactForm.channel} onChange={e => setArtifactForm(p => ({ ...p, channel: e.target.value }))} placeholder="channel" />
            <input className="ops-input" value={artifactForm.sha256} onChange={e => setArtifactForm(p => ({ ...p, sha256: e.target.value }))} placeholder="sha256" />
            <label className="ops-checkbox"><input type="checkbox" checked={artifactForm.isSigned} onChange={e => setArtifactForm(p => ({ ...p, isSigned: e.target.checked }))} /> Signed</label>
            <Button size="small" onClick={submitArtifact} disabled={!artifactForm.sha256.trim()} loading={deployBusy}>Record Artifact</Button>
          </div>

          <div className="ops-card-list">
            {artifacts.length === 0 && <div className="ops-empty">No deployment artifacts recorded yet.</div>}
            {artifacts.map(artifact => (
              <div key={artifact.id} className="ops-card">
                <div className="ops-card-title">{artifact.artifact_type} {artifact.version}</div>
                <div className="ops-card-meta">{artifact.channel} • {artifact.is_signed ? 'Signed' : 'Unsigned'}</div>
                <code className="ops-code">{artifact.sha256}</code>
                <Button size="small" variant="ghost" onClick={() => verifyArtifact(artifact.id)} loading={deployBusy}>Verify</Button>
              </div>
            ))}
          </div>

          {verification && (
            <div className={`ops-banner ${verification.status}`}>
              Verification: {verification.status} ({verification.artifact.version})
            </div>
          )}
        </section>
      )}

      {view === 'eval' && (
        <section className="ops-section">
          <h3>Eval Harness</h3>
          <div className="ops-row">
            <input className="ops-input" value={evalSuiteName} onChange={e => setEvalSuiteName(e.target.value)} placeholder="suite name" />
            <Button size="small" onClick={runEval} loading={evalBusy}>Run Eval</Button>
          </div>
          <textarea className="ops-textarea" value={evalInput} onChange={e => setEvalInput(e.target.value)} />
          {evalResult && <div className="ops-banner info">{evalResult}</div>}
          <div className="ops-card-list">
            {evalRuns.length === 0 && <div className="ops-empty">No eval runs yet.</div>}
            {evalRuns.map(run => (
              <div key={run.id} className="ops-card">
                <div className="ops-card-title">{run.suite_name}</div>
                <div className="ops-card-meta">{run.passed_cases}/{run.total_cases} • {(run.avg_confidence * 100).toFixed(1)}%</div>
                <div className="ops-card-meta">{new Date(run.created_at).toLocaleString()}</div>
              </div>
            ))}
          </div>
        </section>
      )}

      {view === 'triage' && (
        <section className="ops-section">
          <h3>Ticket Triage Autopilot</h3>
          <div className="ops-row">
            <Button size="small" onClick={runTriage} loading={triageBusy}>Cluster Tickets</Button>
          </div>
          <textarea className="ops-textarea" value={triageInput} onChange={e => setTriageInput(e.target.value)} />
          {triageOutput && <pre className="ops-pre">{triageOutput}</pre>}
          <div className="ops-card-list">
            {triageHistory.length === 0 && <div className="ops-empty">No triage clusters yet.</div>}
            {triageHistory.map(item => (
              <div key={item.id} className="ops-card">
                <div className="ops-card-title">{item.summary}</div>
                <div className="ops-card-meta">{item.cluster_key} • {item.ticket_count} tickets</div>
                <div className="ops-card-meta">{new Date(item.created_at).toLocaleString()}</div>
              </div>
            ))}
          </div>
        </section>
      )}

      {view === 'runbook' && (
        <section className="ops-section">
          <h3>Runbook Mode</h3>
          <div className="ops-row">
            <input className="ops-input" value={runbookScenario} onChange={e => setRunbookScenario(e.target.value)} placeholder="scenario" />
            <Button size="small" onClick={startRunbook} loading={runbookBusy}>Start Runbook</Button>
          </div>
          <textarea className="ops-textarea" value={runbookStepsInput} onChange={e => setRunbookStepsInput(e.target.value)} />
          <div className="ops-card-list">
            {runbookSessions.length === 0 && <div className="ops-empty">No runbook sessions yet.</div>}
            {runbookSessions.map(session => {
              const steps = safeParseJson<string[]>(session.steps_json, []);
              return (
                <div key={session.id} className="ops-card">
                  <div className="ops-card-title">{session.scenario}</div>
                  <div className="ops-card-meta">Step {session.current_step + 1}/{Math.max(steps.length, 1)} • {session.status}</div>
                  {steps.length > 0 && (
                    <div className="ops-card-meta">Current: {steps[Math.min(session.current_step, steps.length - 1)]}</div>
                  )}
                  <div className="ops-row">
                    <Button size="small" variant="ghost" onClick={() => advanceRunbook(session)} loading={runbookBusy} disabled={session.status === 'completed'}>Next Step</Button>
                    <Button size="small" variant="secondary" onClick={() => advanceRunbook(session, true)} loading={runbookBusy} disabled={session.status === 'completed'}>Mark Complete</Button>
                  </div>
                </div>
              );
            })}
          </div>
        </section>
      )}

      {view === 'integrations' && (
        <section className="ops-section">
          <h3>Integrations</h3>
          <div className="ops-card-list">
            {INTEGRATION_TYPES.map(type => {
              const current = integrationMap.get(type);
              const enabled = current?.enabled ?? false;
              return (
                <div key={type} className="ops-card">
                  <div className="ops-card-title">{type}</div>
                  <label className="ops-checkbox">
                    <input
                      type="checkbox"
                      checked={enabled}
                      onChange={(e) => void saveIntegration(type, e.target.checked)}
                    />
                    Enabled
                  </label>
                  <textarea
                    className="ops-textarea"
                    value={integrationConfigDraft[type] ?? ''}
                    onChange={(e) => setIntegrationConfigDraft(prev => ({ ...prev, [type]: e.target.value }))}
                    placeholder='{"endpoint":"..."}'
                  />
                  <Button
                    size="small"
                    onClick={() => void saveIntegration(type, enabled)}
                    loading={integrationBusyType === type}
                  >
                    Save Config
                  </Button>
                </div>
              );
            })}
          </div>
        </section>
      )}

      {showRollbackConfirm && (
        <div className="ops-modal-overlay" role="presentation">
          <div className="ops-modal" role="dialog" aria-modal="true" aria-label="Confirm rollback">
            <h4>Confirm rollback?</h4>
            <p>This marks the latest deployment run as rolled back.</p>
            <div className="ops-row">
              <Button size="small" variant="danger" onClick={runRollback} loading={deployBusy}>Confirm Rollback</Button>
              <Button size="small" variant="secondary" onClick={() => setShowRollbackConfirm(false)} disabled={deployBusy}>Cancel</Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
