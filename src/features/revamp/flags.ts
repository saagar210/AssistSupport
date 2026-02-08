export type RevampFlagId =
  | 'ASSISTSUPPORT_REVAMP_APP_SHELL'
  | 'ASSISTSUPPORT_REVAMP_INBOX'
  | 'ASSISTSUPPORT_REVAMP_WORKSPACE'
  | 'ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2'
  | 'ASSISTSUPPORT_LLM_ROUTER_V2';

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

  return (Object.values(REVAMP_FLAG_DEFINITIONS) as RevampFlagDefinition[]).reduce<RevampFlags>(
    (acc, definition) => {
      const envValue = parseBooleanLike(envValues[definition.envKey]);

      let storageValue: boolean | null = null;
      try {
        storageValue = parseBooleanLike(resolvedStorage?.getItem(definition.storageKey));
      } catch {
        storageValue = null;
      }

      // Storage override wins for local testing/rehearsal to avoid rebuilds.
      acc[definition.id] = storageValue ?? envValue ?? definition.defaultValue;
      return acc;
    },
    {
      ASSISTSUPPORT_REVAMP_APP_SHELL: false,
      ASSISTSUPPORT_REVAMP_INBOX: false,
      ASSISTSUPPORT_REVAMP_WORKSPACE: false,
      ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2: false,
      ASSISTSUPPORT_LLM_ROUTER_V2: false,
    },
  );
}

export function getEnabledRevampFlags(flags: RevampFlags): RevampFlagId[] {
  return (Object.keys(flags) as RevampFlagId[]).filter((flagId) => flags[flagId]);
}
