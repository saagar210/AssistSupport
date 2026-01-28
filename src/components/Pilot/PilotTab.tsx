import { useState, useCallback } from 'react';
import { PilotDashboard } from './PilotDashboard';
import { PilotQueryTester } from './PilotQueryTester';
import './Pilot.css';

export function PilotTab() {
  const [refreshKey, setRefreshKey] = useState(0);

  const handleQueryLogged = useCallback(() => {
    setRefreshKey(k => k + 1);
  }, []);

  return (
    <div className="pilot-tab-scroll">
      <PilotQueryTester onQueryLogged={handleQueryLogged} />
      <PilotDashboard key={refreshKey} />
    </div>
  );
}
