import { useState, useCallback } from 'react';
import { PilotDashboard } from './PilotDashboard';
import { PilotQueryTester } from './PilotQueryTester';

export function PilotTab() {
  const [refreshKey, setRefreshKey] = useState(0);

  const handleQueryLogged = useCallback(() => {
    setRefreshKey(k => k + 1);
  }, []);

  return (
    <div style={{ maxWidth: 900, padding: '1.5rem' }}>
      <PilotQueryTester onQueryLogged={handleQueryLogged} />
      <PilotDashboard key={refreshKey} />
    </div>
  );
}
