import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { MemoryKernelEnrichmentResult } from '../types';

const ENRICHMENT_SECTION_HEADING = 'MemoryKernel Policy Context';

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
          message: result.message,
          fallbackReason: result.fallback_reason,
          machineErrorCode: result.machine_error_code,
        };
      } catch (err) {
        return {
          diagnosticNotes: diagnosticNotes?.trim() ? diagnosticNotes : undefined,
          enrichmentApplied: false,
          status: 'fallback',
          message: `MemoryKernel enrichment unavailable: ${String(err)}`,
          fallbackReason: 'adapter-error',
          machineErrorCode: null,
        };
      }
    },
    []
  );

  return { enrichDiagnosticNotes };
}
