import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { DecisionTree, TreeStructure } from '../types';

export function useDecisionTrees() {
  const [trees, setTrees] = useState<DecisionTree[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadTrees = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<DecisionTree[]>('list_decision_trees');
      setTrees(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  const getTree = useCallback(async (treeId: string): Promise<TreeStructure | null> => {
    try {
      const tree = await invoke<DecisionTree>('get_decision_tree', { treeId });
      return JSON.parse(tree.tree_json) as TreeStructure;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return null;
    }
  }, []);

  return { trees, loading, error, loadTrees, getTree };
}
