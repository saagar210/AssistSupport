export type RevampFlagId =
  | 'ASSISTSUPPORT_REVAMP_APP_SHELL'
  | 'ASSISTSUPPORT_REVAMP_INBOX'
  | 'ASSISTSUPPORT_REVAMP_WORKSPACE'
  | 'ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2'
  | 'ASSISTSUPPORT_LLM_ROUTER_V2'
  | 'ASSISTSUPPORT_ENABLE_ADMIN_TABS'
  | 'ASSISTSUPPORT_ENABLE_NETWORK_INGEST';

interface RevampFlagDefinition {
  id: RevampFlagId;
  envKey: string;
  storageKey: string;
  defaultValue: boolean;
}

const REVAMP_FLAG_DEFINITIONS: Record<RevampFlagId, RevampFlagDefinition> = {
  ASSISTSUPPORT_REVAMP_APP_SHELL: {
    id: 'ASSISTSUPPORT_REVAMP_APP_SHELL',
    envKey: 'VITE_ASSISTSUPPORT_REVAMP_APP_SHELL',
    storageKey: 'assistsupport.flag.ASSISTSUPPORT_REVAMP_APP_SHELL',
    defaultValue: false,
  },
  ASSISTSUPPORT_REVAMP_INBOX: {
    id: 'ASSISTSUPPORT_REVAMP_INBOX',
    envKey: 'VITE_ASSISTSUPPORT_REVAMP_INBOX',
    storageKey: 'assistsupport.flag.ASSISTSUPPORT_REVAMP_INBOX',
    defaultValue: false,
  },
  ASSISTSUPPORT_REVAMP_WORKSPACE: {
    id: 'ASSISTSUPPORT_REVAMP_WORKSPACE',
    envKey: 'VITE_ASSISTSUPPORT_REVAMP_WORKSPACE',
    storageKey: 'assistsupport.flag.ASSISTSUPPORT_REVAMP_WORKSPACE',
    defaultValue: false,
  },
  ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2: {
    id: 'ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2',
    envKey: 'VITE_ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2',
    storageKey: 'assistsupport.flag.ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2',
    defaultValue: false,
  },
  ASSISTSUPPORT_LLM_ROUTER_V2: {
    id: 'ASSISTSUPPORT_LLM_ROUTER_V2',
    envKey: 'VITE_ASSISTSUPPORT_LLM_ROUTER_V2',
    storageKey: 'assistsupport.flag.ASSISTSUPPORT_LLM_ROUTER_V2',
    defaultValue: false,
  },
  ASSISTSUPPORT_ENABLE_ADMIN_TABS: {
    id: 'ASSISTSUPPORT_ENABLE_ADMIN_TABS',
    envKey: 'VITE_ASSISTSUPPORT_ENABLE_ADMIN_TABS',
    storageKey: 'assistsupport.flag.ASSISTSUPPORT_ENABLE_ADMIN_TABS',
    defaultValue: false,
  },
  ASSISTSUPPORT_ENABLE_NETWORK_INGEST: {
    id: 'ASSISTSUPPORT_ENABLE_NETWORK_INGEST',
    envKey: 'VITE_ASSISTSUPPORT_ENABLE_NETWORK_INGEST',
    storageKey: 'assistsupport.flag.ASSISTSUPPORT_ENABLE_NETWORK_INGEST',
    defaultValue: false,
  },
};

export type RevampFlags = Record<RevampFlagId, boolean>;

interface ResolveRevampFlagsOptions {
  env?: Record<string, unknown>;
  storage?: Pick<Storage, 'getItem'> | null;
}

function parseBooleanLike(value: unknown): boolean | null {
  if (typeof value === 'boolean') {
    return value;
  }

  if (typeof value !== 'string') {
    return null;
  }

  const normalized = value.trim().toLowerCase();

  if (['1', 'true', 'yes', 'on', 'enabled'].includes(normalized)) {
    return true;
  }

  if (['0', 'false', 'no', 'off', 'disabled'].includes(normalized)) {
    return false;
  }

  return null;
}

function resolveStorage(
  providedStorage?: Pick<Storage, 'getItem'> | null,
): Pick<Storage, 'getItem'> | null {
  if (providedStorage !== undefined) {
    return providedStorage;
  }

  if (typeof window === 'undefined') {
    return null;
  }

  return window.localStorage;
}

export function resolveRevampFlags({ env, storage }: ResolveRevampFlagsOptions = {}): RevampFlags {
  const envValues = env ?? (import.meta.env as unknown as Record<string, unknown>);
  const resolvedStorage = resolveStorage(storage);
  const mode = String(envValues.MODE ?? '').trim().toLowerCase();
  const devFlag = parseBooleanLike(envValues.DEV);
  const isDevelopment = devFlag ?? (mode === 'development' || mode === 'dev');
  const policyFlagIds: ReadonlySet<RevampFlagId> = new Set([
    'ASSISTSUPPORT_ENABLE_ADMIN_TABS',
    'ASSISTSUPPORT_ENABLE_NETWORK_INGEST',
  ]);

  return (Object.values(REVAMP_FLAG_DEFINITIONS) as RevampFlagDefinition[]).reduce<RevampFlags>(
    (acc, definition) => {
      const envValue = parseBooleanLike(envValues[definition.envKey]);

      let storageValue: boolean | null = null;
      try {
        storageValue = parseBooleanLike(resolvedStorage?.getItem(definition.storageKey));
      } catch {
        storageValue = null;
      }

      // Storage overrides are useful for local rehearsal, but policy flags must be env-authoritative
      // outside development builds so we don't "accidentally" enable admin/network surfaces.
      const effectiveStorageValue =
        !isDevelopment && policyFlagIds.has(definition.id) ? null : storageValue;

      acc[definition.id] = effectiveStorageValue ?? envValue ?? definition.defaultValue;
      return acc;
    },
    {
      ASSISTSUPPORT_REVAMP_APP_SHELL: false,
      ASSISTSUPPORT_REVAMP_INBOX: false,
      ASSISTSUPPORT_REVAMP_WORKSPACE: false,
      ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2: false,
      ASSISTSUPPORT_LLM_ROUTER_V2: false,
      ASSISTSUPPORT_ENABLE_ADMIN_TABS: false,
      ASSISTSUPPORT_ENABLE_NETWORK_INGEST: false,
    },
  );
}

export function getEnabledRevampFlags(flags: RevampFlags): RevampFlagId[] {
  // "Revamp enabled" is used for preview-mode messaging. Keep this narrowly scoped
  // to actual revamp / routing toggles, not general feature policy flags.
  const revampScoped: RevampFlagId[] = (Object.keys(flags) as RevampFlagId[]).filter((flagId) => {
    return flagId.startsWith('ASSISTSUPPORT_REVAMP_') || flagId === 'ASSISTSUPPORT_LLM_ROUTER_V2';
  });
  return revampScoped.filter((flagId) => flags[flagId]);
}
