import type { RevampFlags } from '../revamp';
import type { TabId } from './types';

export function isTabEnabled(tab: TabId, flags: RevampFlags): boolean {
  // Default posture: keep the operator UI focused and offline-first.
  // Admin and network ingestion surfaces are opt-in via flags.
  if (tab === 'ingest') {
    return Boolean(flags.ASSISTSUPPORT_ENABLE_NETWORK_INGEST);
  }

  if (tab === 'analytics' || tab === 'pilot' || tab === 'search') {
    return Boolean(flags.ASSISTSUPPORT_ENABLE_ADMIN_TABS);
  }

  return true;
}

