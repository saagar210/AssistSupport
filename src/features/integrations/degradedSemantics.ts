export interface DegradedClassification {
  status: string;
  reason: string;
  guidance: string;
}

const MEMORY_KERNEL_FALLBACK_HINTS: Record<string, string> = {
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
  unknown: 'MemoryKernel fallback is active. Draft flow remains available.',
};

const SEARCH_API_HINTS: Record<string, string> = {
  offline: 'Search API is offline. Start the local search-api service and retry.',
  timeout: 'Search API timed out. Retry or review local runtime load.',
  'auth-failure':
    'Search API authentication failed. Verify ASSISTSUPPORT_SEARCH_API_KEY and token wiring.',
  'rate-limit':
    'Search API rate-limited the request. Wait briefly and retry.',
  'wrong-service':
    'Port 3000 is serving a different service. Start Search API on localhost:3000.',
  'invalid-response': 'Search API returned an invalid payload. Check API logs and contract version.',
  'invalid-config':
    'Search API base URL configuration is invalid. Ensure a loopback http:// address is used.',
  degraded: 'Search API is degraded. Continue with fallback search behavior until recovered.',
  unknown: 'Search API fallback is active. Retry after validating local service health.',
};

function getSearchApiHint(reason: string): string {
  return SEARCH_API_HINTS[reason] ?? SEARCH_API_HINTS.unknown;
}

export function normalizeDegradedReason(value: string | null | undefined): string {
  const normalized = (value ?? '').trim().toLowerCase();
  return normalized.length > 0 ? normalized : 'unknown';
}

export function getMemoryKernelGuidance(reason: string | null | undefined): string {
  return MEMORY_KERNEL_FALLBACK_HINTS[normalizeDegradedReason(reason)] ?? MEMORY_KERNEL_FALLBACK_HINTS.unknown;
}

export function classifySearchApiFailure(message: string): DegradedClassification {
  const normalized = message.toLowerCase();

  if (normalized.includes('401') || normalized.includes('403') || normalized.includes('unauthorized')) {
    return {
      status: 'degraded',
      reason: 'auth-failure',
      guidance: getSearchApiHint('auth-failure'),
    };
  }
  if (normalized.includes('429') || normalized.includes('rate limit')) {
    return {
      status: 'degraded',
      reason: 'rate-limit',
      guidance: getSearchApiHint('rate-limit'),
    };
  }
  if (normalized.includes('timed out') || normalized.includes('timeout')) {
    return {
      status: 'offline',
      reason: 'timeout',
      guidance: getSearchApiHint('timeout'),
    };
  }
  if (normalized.includes('wrong-service') || normalized.includes('serving html') || normalized.includes('html')) {
    return {
      status: 'degraded',
      reason: 'wrong-service',
      guidance: getSearchApiHint('wrong-service'),
    };
  }
  if (normalized.includes('invalid-response')) {
    return {
      status: 'degraded',
      reason: 'invalid-response',
      guidance: getSearchApiHint('invalid-response'),
    };
  }
  if (normalized.includes('invalid-config')) {
    return {
      status: 'degraded',
      reason: 'invalid-config',
      guidance: getSearchApiHint('invalid-config'),
    };
  }
  if (
    normalized.includes('unavailable')
    || normalized.includes('econnrefused')
    || normalized.includes('connection refused')
    || normalized.includes('offline')
    || normalized.includes('failed to send')
  ) {
    return {
      status: 'offline',
      reason: 'offline',
      guidance: getSearchApiHint('offline'),
    };
  }
  if (normalized.includes('degraded')) {
    return {
      status: 'degraded',
      reason: 'degraded',
      guidance: getSearchApiHint('degraded'),
    };
  }

  return {
    status: 'degraded',
    reason: 'unknown',
    guidance: getSearchApiHint('unknown'),
  };
}

export function classifySearchApiHealthStatus(
  status: string | null | undefined,
  message: string | null | undefined
): DegradedClassification {
  const reason = normalizeDegradedReason(status);

  if (reason === 'ok' || reason === 'healthy') {
    return {
      status: 'healthy',
      reason: 'ok',
      guidance: 'Search API is healthy.',
    };
  }

  if (reason === 'degraded' && message) {
    return classifySearchApiFailure(message);
  }

  const normalizedReason = reason in SEARCH_API_HINTS ? reason : 'unknown';
  return {
    status: normalizedReason === 'offline' ? 'offline' : 'degraded',
    reason: normalizedReason,
    guidance: getSearchApiHint(normalizedReason),
  };
}
