# Vector Store Rebuild Runbook

## When to use this runbook

Use this when vector search is quarantined, semantic KB search is marked as rebuild-required, or document deletion/search results suggest stale vector data.

## Important operating notes

- Vector embeddings stay local, but the current vector store is not encrypted at rest when enabled.
- SQLite chunk metadata is the source of truth.
- A rebuild is safe because the vector table can be regenerated from trusted chunk records.

## Typical triggers

- Upgrade from a pre-fix vector schema.
- Recovery screen or diagnostics report `rebuild required`.
- Namespace filtering or delete behavior was previously inconsistent.
- You intentionally cleared or replaced the KB dataset.

## Step 1: Confirm rebuild is actually required

Use the diagnostics surface and look for:

- vector store not ready,
- legacy rows detected,
- missing metadata columns,
- or stale delete state.

## Step 2: Prepare for the rebuild

- Confirm the KB folder still points at the intended content.
- Ensure the embedding model is present and loadable.
- Avoid running concurrent ingest jobs during the rebuild.

## Step 3: Run the rebuild

- Start the rebuild from the diagnostics or KB maintenance action.
- Let the app recreate vector rows from the current SQLite chunk metadata.
- Do not manually copy old LanceDB files back into place after rebuild.

## Step 4: Re-verify trust boundaries

After rebuild, confirm:

- KB search returns expected results for at least two different namespaces.
- Deleting a KB document removes it from subsequent semantic search.
- Clearing KB data leaves vector search unavailable until fresh content is indexed again.

## If rebuild fails

- Keep the vector feature quarantined.
- Capture the rebuild error and current KB stats.
- Re-check whether the embedding model loaded successfully.
- If SQLite chunk data looks correct but rebuild still fails, treat the vector store as disposable and investigate the rebuild path instead of reusing old vector files.

## Completion criteria

- Diagnostics no longer report rebuild required.
- Semantic search works for current content only.
- No deleted document appears in vector-backed results.
