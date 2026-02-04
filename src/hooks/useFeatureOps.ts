import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  DeploymentArtifactRecord,
  DeploymentHealthSummary,
  EvalRunRecord,
  IntegrationConfigRecord,
  KbGapCandidate,
  RunbookSessionRecord,
  SignedArtifactVerificationResult,
  TriageClusterRecord,
} from '../types';

export interface DeploymentPreflightResult {
  ok: boolean;
  checks: string[];
}

export interface EvalHarnessCase {
  query: string;
  expected_mode?: string;
  min_confidence?: number;
}

export interface EvalHarnessResult {
  run_id: string;
  total_cases: number;
  passed_cases: number;
  avg_confidence: number;
}

export interface TriageTicketInput {
  id: string;
  summary: string;
}

export interface TriageClusterOutput {
  cluster_key: string;
  summary: string;
  ticket_ids: string[];
}

export function useFeatureOps() {
  const getKbGapCandidates = useCallback(async (limit = 20, status = 'open'): Promise<KbGapCandidate[]> => {
    return invoke<KbGapCandidate[]>('get_kb_gap_candidates', { limit, status });
  }, []);

  const updateKbGapStatus = useCallback(async (
    id: string,
    status: 'open' | 'accepted' | 'resolved' | 'ignored',
    resolutionNote?: string,
  ): Promise<void> => {
    await invoke('update_kb_gap_status', {
      id,
      status,
      resolutionNote: resolutionNote ?? null,
    });
  }, []);

  const getDeploymentHealthSummary = useCallback(async (): Promise<DeploymentHealthSummary> => {
    return invoke<DeploymentHealthSummary>('get_deployment_health_summary');
  }, []);

  const listDeploymentArtifacts = useCallback(async (limit = 50): Promise<DeploymentArtifactRecord[]> => {
    return invoke<DeploymentArtifactRecord[]>('list_deployment_artifacts', { limit });
  }, []);

  const runDeploymentPreflight = useCallback(async (targetChannel: string): Promise<DeploymentPreflightResult> => {
    return invoke<DeploymentPreflightResult>('run_deployment_preflight', { targetChannel });
  }, []);

  const recordDeploymentArtifact = useCallback(async (
    artifactType: string,
    version: string,
    channel: string,
    sha256: string,
    isSigned: boolean,
  ): Promise<string> => {
    return invoke<string>('record_deployment_artifact', {
      artifactType,
      version,
      channel,
      sha256,
      isSigned,
    });
  }, []);

  const verifySignedArtifact = useCallback(async (
    artifactId: string,
    expectedSha256?: string,
  ): Promise<SignedArtifactVerificationResult> => {
    return invoke<SignedArtifactVerificationResult>('verify_signed_artifact', {
      artifactId,
      expectedSha256: expectedSha256 ?? null,
    });
  }, []);

  const rollbackDeploymentRun = useCallback(async (runId: string, reason?: string): Promise<void> => {
    await invoke('rollback_deployment_run', {
      runId,
      reason: reason ?? null,
    });
  }, []);

  const runEvalHarness = useCallback(async (
    suiteName: string,
    cases: EvalHarnessCase[],
  ): Promise<EvalHarnessResult> => {
    return invoke<EvalHarnessResult>('run_eval_harness', { suiteName, cases });
  }, []);

  const listEvalRuns = useCallback(async (limit = 50): Promise<EvalRunRecord[]> => {
    return invoke<EvalRunRecord[]>('list_eval_runs', { limit });
  }, []);

  const clusterTicketsForTriage = useCallback(async (
    tickets: TriageTicketInput[],
  ): Promise<TriageClusterOutput[]> => {
    return invoke<TriageClusterOutput[]>('cluster_tickets_for_triage', { tickets });
  }, []);

  const listRecentTriageClusters = useCallback(async (limit = 50): Promise<TriageClusterRecord[]> => {
    return invoke<TriageClusterRecord[]>('list_recent_triage_clusters', { limit });
  }, []);

  const startRunbookSession = useCallback(async (
    scenario: string,
    steps: string[],
  ): Promise<RunbookSessionRecord> => {
    return invoke<RunbookSessionRecord>('start_runbook_session', { scenario, steps });
  }, []);

  const advanceRunbookSession = useCallback(async (
    sessionId: string,
    currentStep: number,
    status?: string,
  ): Promise<void> => {
    await invoke('advance_runbook_session', {
      sessionId,
      currentStep,
      status: status ?? null,
    });
  }, []);

  const listRunbookSessions = useCallback(async (
    limit = 50,
    status?: string,
  ): Promise<RunbookSessionRecord[]> => {
    return invoke<RunbookSessionRecord[]>('list_runbook_sessions', {
      limit,
      status: status ?? null,
    });
  }, []);

  const listIntegrations = useCallback(async (): Promise<IntegrationConfigRecord[]> => {
    return invoke<IntegrationConfigRecord[]>('list_integrations');
  }, []);

  const configureIntegration = useCallback(async (
    integrationType: string,
    enabled: boolean,
    configJson?: string,
  ): Promise<void> => {
    await invoke('configure_integration', {
      integrationType,
      enabled,
      configJson: configJson ?? null,
    });
  }, []);

  return {
    getKbGapCandidates,
    updateKbGapStatus,
    getDeploymentHealthSummary,
    listDeploymentArtifacts,
    runDeploymentPreflight,
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
  };
}
