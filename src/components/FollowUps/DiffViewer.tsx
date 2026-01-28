import { useMemo } from 'react';
import { Button } from '../shared/Button';
import './DiffViewer.css';

interface DiffViewerProps {
  textA: string;
  textB: string;
  labelA?: string;
  labelB?: string;
  onClose: () => void;
}

type DiffLine = {
  type: 'add' | 'remove' | 'same';
  content: string;
};

// LCS-based diff algorithm
function computeDiff(textA: string, textB: string): DiffLine[] {
  const linesA = textA.split('\n');
  const linesB = textB.split('\n');

  // Build LCS table
  const m = linesA.length;
  const n = linesB.length;
  const dp: number[][] = Array.from({ length: m + 1 }, () => Array(n + 1).fill(0));

  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (linesA[i - 1] === linesB[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
      }
    }
  }

  // Backtrack to get diff
  let i = m, j = n;
  const stack: DiffLine[] = [];

  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && linesA[i - 1] === linesB[j - 1]) {
      stack.push({ type: 'same', content: linesA[i - 1] });
      i--; j--;
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      stack.push({ type: 'add', content: linesB[j - 1] });
      j--;
    } else {
      stack.push({ type: 'remove', content: linesA[i - 1] });
      i--;
    }
  }

  return stack.reverse();
}

export function DiffViewer({ textA, textB, labelA = 'Version A', labelB = 'Version B', onClose }: DiffViewerProps) {
  const diffLines = useMemo(() => computeDiff(textA, textB), [textA, textB]);

  const stats = useMemo(() => {
    let additions = 0;
    let deletions = 0;
    for (const line of diffLines) {
      if (line.type === 'add') additions++;
      else if (line.type === 'remove') deletions++;
    }
    return { additions, deletions };
  }, [diffLines]);

  // Compute line numbers for each side
  let lineNumA = 0;
  let lineNumB = 0;

  return (
    <div className="diff-overlay" onClick={onClose}>
      <div className="diff-modal" onClick={(e) => e.stopPropagation()}>
        <div className="diff-header">
          <div className="diff-header-labels">
            <span className="diff-label diff-label-remove">{labelA}</span>
            <span className="diff-label-separator">vs</span>
            <span className="diff-label diff-label-add">{labelB}</span>
          </div>
          <div className="diff-header-stats">
            <span className="diff-stat diff-stat-add">+{stats.additions}</span>
            <span className="diff-stat diff-stat-remove">-{stats.deletions}</span>
          </div>
          <Button variant="ghost" size="small" onClick={onClose}>
            Close
          </Button>
        </div>

        <div className="diff-content">
          {diffLines.map((line, index) => {
            let displayLineA = '';
            let displayLineB = '';

            if (line.type === 'same') {
              lineNumA++;
              lineNumB++;
              displayLineA = String(lineNumA);
              displayLineB = String(lineNumB);
            } else if (line.type === 'remove') {
              lineNumA++;
              displayLineA = String(lineNumA);
            } else {
              lineNumB++;
              displayLineB = String(lineNumB);
            }

            return (
              <div key={index} className={`diff-line ${line.type}`}>
                <span className="diff-line-number diff-line-number-a">{displayLineA}</span>
                <span className="diff-line-number diff-line-number-b">{displayLineB}</span>
                <span className="diff-line-marker">
                  {line.type === 'add' ? '+' : line.type === 'remove' ? '-' : ' '}
                </span>
                <span className="diff-line-content">{line.content || '\u00A0'}</span>
              </div>
            );
          })}
          {diffLines.length === 0 && (
            <div className="diff-empty">No differences found.</div>
          )}
        </div>
      </div>
    </div>
  );
}
