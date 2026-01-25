import { useState } from 'react';
import type { TreeStructure } from '../../types';
import './TreeRunner.css';

interface TreeRunnerProps {
  tree: TreeStructure;
  onComplete: (path: string[]) => void;
  onReset: () => void;
}

export function TreeRunner({ tree, onComplete, onReset }: TreeRunnerProps) {
  const [currentNodeId, setCurrentNodeId] = useState(tree.root_node_id);
  const [path, setPath] = useState<string[]>([tree.root_node_id]);

  const currentNode = tree.nodes[currentNodeId];

  function handleOption(nextNodeId: string | null) {
    if (!nextNodeId) {
      onComplete(path);
      return;
    }
    setCurrentNodeId(nextNodeId);
    setPath(prev => [...prev, nextNodeId]);
  }

  function handleBack() {
    if (path.length > 1) {
      const newPath = path.slice(0, -1);
      setPath(newPath);
      setCurrentNodeId(newPath[newPath.length - 1]);
    }
  }

  function handleRestart() {
    setCurrentNodeId(tree.root_node_id);
    setPath([tree.root_node_id]);
  }

  if (!currentNode) {
    return (
      <div className="tree-runner tree-error">
        <p>Error: Node not found</p>
        <button onClick={handleRestart}>Restart</button>
      </div>
    );
  }

  return (
    <div className="tree-runner">
      <div className="tree-breadcrumb">
        {path.map((nodeId, i) => {
          const node = tree.nodes[nodeId];
          return (
            <span key={nodeId} className="breadcrumb-item">
              {i > 0 && <span className="breadcrumb-separator">→</span>}
              <span className="breadcrumb-text">{node?.title || nodeId}</span>
            </span>
          );
        })}
      </div>

      <div className={`tree-node tree-node-${currentNode.type}`}>
        <h4 className="node-title">{currentNode.title}</h4>

        {currentNode.content && (
          <div className="node-content">
            {currentNode.content.split('\n').map((line, i) => (
              <p key={i}>{line}</p>
            ))}
          </div>
        )}

        {currentNode.options && currentNode.options.length > 0 && (
          <div className="node-options">
            {currentNode.options.map((opt, i) => (
              <button
                key={i}
                className="node-option"
                onClick={() => handleOption(opt.next_node_id)}
              >
                {opt.label}
              </button>
            ))}
          </div>
        )}

        {currentNode.type === 'terminal' && (
          <div className="terminal-actions">
            <button className="tree-complete" onClick={() => onComplete(path)}>
              Done
            </button>
            <button className="tree-restart" onClick={handleRestart}>
              Start Over
            </button>
          </div>
        )}
      </div>

      <div className="tree-controls">
        {path.length > 1 && (
          <button className="tree-back" onClick={handleBack}>
            ← Back
          </button>
        )}
        <button className="tree-close" onClick={onReset}>
          Close Tree
        </button>
      </div>
    </div>
  );
}
