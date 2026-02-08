export function countWords(text: string): number {
  return text.trim().split(/\s+/).filter(Boolean).length;
}

function levenshteinDistance(a: string, b: string): number {
  if (a === b) {
    return 0;
  }

  const rows = a.length + 1;
  const cols = b.length + 1;
  const matrix: number[][] = Array.from({ length: rows }, () => Array(cols).fill(0));

  for (let i = 0; i < rows; i += 1) {
    matrix[i][0] = i;
  }
  for (let j = 0; j < cols; j += 1) {
    matrix[0][j] = j;
  }

  for (let i = 1; i < rows; i += 1) {
    for (let j = 1; j < cols; j += 1) {
      const substitutionCost = a[i - 1] === b[j - 1] ? 0 : 1;
      matrix[i][j] = Math.min(
        matrix[i - 1][j] + 1,
        matrix[i][j - 1] + 1,
        matrix[i - 1][j - 1] + substitutionCost,
      );
    }
  }

  return matrix[rows - 1][cols - 1];
}

export function calculateEditRatio(original: string, current: string): number {
  const maxLength = Math.max(original.length, current.length);
  if (maxLength === 0) {
    return 0;
  }

  // Avoid quadratic work for long responses where a coarse score is sufficient.
  if (maxLength > 1200) {
    const originalTokens = original.trim().toLowerCase().split(/\s+/).filter(Boolean);
    const currentTokens = current.trim().toLowerCase().split(/\s+/).filter(Boolean);
    if (originalTokens.length === 0 && currentTokens.length === 0) {
      return 0;
    }

    const tokenCounts = new Map<string, number>();
    for (const token of originalTokens) {
      tokenCounts.set(token, (tokenCounts.get(token) ?? 0) + 1);
    }

    let common = 0;
    for (const token of currentTokens) {
      const remaining = tokenCounts.get(token) ?? 0;
      if (remaining > 0) {
        common += 1;
        tokenCounts.set(token, remaining - 1);
      }
    }

    const baseline = Math.max(originalTokens.length, currentTokens.length);
    if (baseline === 0) {
      return 0;
    }

    return 1 - common / baseline;
  }

  const distance = levenshteinDistance(original, current);
  return distance / maxLength;
}
