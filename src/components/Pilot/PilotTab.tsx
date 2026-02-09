import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { PilotDashboard } from './PilotDashboard';
import { PilotQueryTester } from './PilotQueryTester';
import './Pilot.css';

interface PilotLoggingPolicy {
  enabled: boolean;
  retention_days: number;
  max_rows: number;
}

export function PilotTab() {
  const [refreshKey, setRefreshKey] = useState(0);
  const [policy, setPolicy] = useState<PilotLoggingPolicy | null>(null);
  const pilotLoggingEnabled = policy?.enabled ?? false;

  useEffect(() => {
    invoke<PilotLoggingPolicy>('get_pilot_logging_policy')
      .then(setPolicy)
      .catch(() => setPolicy({ enabled: false, retention_days: 14, max_rows: 500 }));
  }, []);

  const handleQueryLogged = useCallback(() => {
    setRefreshKey(k => k + 1);
  }, []);

  return (
    <div className="pilot-tab-scroll">
      <PilotQueryTester
        pilotLoggingEnabled={pilotLoggingEnabled}
        policy={policy}
        onQueryLogged={handleQueryLogged}
      />
      <PilotDashboard
        key={refreshKey}
        pilotLoggingEnabled={pilotLoggingEnabled}
        policy={policy}
      />
    </div>
  );
}
