import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { MemoryKernelEnrichmentResult } from '../types';

const ENRICHMENT_SECTION_HEADING = 'MemoryKernel Policy Context';
const FALLBACK_HINTS: Record<string, string> = {
  offline: 'MemoryKernel service is offline. Start the local service and retry.',
  timeout: 'MemoryKernel query timed out. Retry or increase the integration timeout.',
  'version-mismatch':
    'MemoryKernel contract mismatch. Align pin/manifest versions before retrying.',
  'schema-unavailable':
    'MemoryKernel schema is unavailable. Run migration checks and retry.',
  'malformed-payload':
    'MemoryKernel returned malformed payload. Verify producer contract alignment.',
  'non-2xx': 'MemoryKernel returned a non-success status. Inspect producer logs and handoff payload.',
  'network-error': 'MemoryKernel network call failed. Validate localhost connectivity and retry.',
  'query-error': 'MemoryKernel query failed. Inspect machine error code and provider logs.',
  degraded: 'MemoryKernel is degraded. Continue in fallback mode until preflight recovers.',
  'feature-disabled': 'MemoryKernel enrichment is disabled by configuration.',
  'adapter-error': 'MemoryKernel adapter encountered an error. Check consumer logs and retry.',
};

export interface MemoryKernelEnrichmentOutcome {
  diagnosticNotes: string | undefined;
  enrichmentApplied: boolean;
  status: string;
  message: string;
  fallbackReason: string | null;
  machineErrorCode: string | null;
}

function joinNotes(existing: string | undefined, enrichmentText: string): string {
  const trimmedExisting = (existing ?? '').trim();
  const section = `${ENRICHMENT_SECTION_HEADING}\n${enrichmentText}`;
  if (!trimmedExisting) {
    return section;
  }
  return `${trimmedExisting}\n\n${section}`;
}

function buildFallbackMessage(
  baseMessage: string,
  fallbackReason: string | null,
  machineErrorCode: string | null
): string {
  const trimmed = baseMessage.trim();
  const reasonKey = (fallbackReason ?? '').trim().toLowerCase();
  const hint = FALLBACK_HINTS[reasonKey] ?? 'MemoryKernel fallback is active. Draft flow remains available.';
  if (machineErrorCode) {
    return `${trimmed} | code=${machineErrorCode} | ${hint}`;
  }
  return `${trimmed} | ${hint}`;
}

export function useMemoryKernelEnrichment() {
  const enrichDiagnosticNotes = useCallback(
    async (userInput: string, diagnosticNotes?: string): Promise<MemoryKernelEnrichmentOutcome> => {
      try {
        const result = await invoke<MemoryKernelEnrichmentResult>('memory_kernel_query_ask', {
          userInput,
        });

        if (result.applied && result.enrichment_text) {
          return {
            diagnosticNotes: joinNotes(diagnosticNotes, result.enrichment_text),
            enrichmentApplied: true,
            status: result.status,
            message: result.message,
            fallbackReason: null,
            machineErrorCode: null,
          };
        }

        return {
          diagnosticNotes: diagnosticNotes?.trim() ? diagnosticNotes : undefined,
          enrichmentApplied: false,
          status: result.status,
          message: buildFallbackMessage(
            result.message,
            result.fallback_reason,
            result.machine_error_code
          ),
          fallbackReason: result.fallback_reason,
          machineErrorCode: result.machine_error_code,
        };
      } catch (err) {
        return {
          diagnosticNotes: diagnosticNotes?.trim() ? diagnosticNotes : undefined,
          enrichmentApplied: false,
          status: 'fallback',
          message: buildFallbackMessage(
            `MemoryKernel enrichment unavailable: ${String(err)}`,
            'adapter-error',
            null
          ),
          fallbackReason: 'adapter-error',
          machineErrorCode: null,
        };
      }
    },
    []
  );

  return { enrichDiagnosticNotes };
}
