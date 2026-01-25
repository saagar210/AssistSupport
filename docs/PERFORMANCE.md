# AssistSupport Performance Guide

**Version**: 1.0
**Last Updated**: 2026-01-25

---

## Performance Budgets

### Startup

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cold launch to UI ready | < 2 seconds | Time from app launch to DraftTab visible |
| First paint | < 500ms | Time to loading spinner visible |
| Engine initialization | Background | LLM/Embedding init does not block UI |

### Search & Navigation

| Metric | Target | Measurement |
|--------|--------|-------------|
| KB search response | < 300ms | Time from query to results displayed |
| Namespace loading | < 500ms | Time to load namespace list with counts |
| Tab switch | < 100ms | Time to render new tab content |

### List Rendering

| Metric | Target | Measurement |
|--------|--------|-------------|
| Document list (1000 items) | Smooth 60fps scroll | No frame drops during scroll |
| Chunk list (5000 items) | Smooth 60fps scroll | Virtual list maintains performance |
| Draft history (500 items) | < 200ms initial render | Time to first paint |

### Memory

| Metric | Target | Measurement |
|--------|--------|-------------|
| Base memory | < 150MB | With DB loaded, no model |
| With model loaded | < 6GB | Depends on model size |
| Peak during generation | < 8GB | During active text generation |

---

## Optimization Strategies Implemented

### 1. Database Query Optimization

**Problem**: N+1 query pattern when loading namespaces with counts

**Solution**: Added `list_namespaces_with_counts()` command that uses a single SQL query with JOINs instead of multiple round-trips.

```sql
-- Before: 1 + (2 × N) queries
SELECT * FROM namespaces;
-- Then for each namespace:
SELECT COUNT(*) FROM kb_documents WHERE namespace_id = ?;
SELECT COUNT(*) FROM ingest_sources WHERE namespace_id = ?;

-- After: 1 query
SELECT n.*,
       COALESCE(d.doc_count, 0) as document_count,
       COALESCE(s.source_count, 0) as source_count
FROM namespaces n
LEFT JOIN (SELECT namespace_id, COUNT(*) as doc_count FROM kb_documents GROUP BY namespace_id) d
  ON d.namespace_id = n.id
LEFT JOIN (SELECT namespace_id, COUNT(*) as source_count FROM ingest_sources GROUP BY namespace_id) s
  ON s.namespace_id = n.id;
```

### 2. Result Caching

**Problem**: Repeated queries for same data

**Solution**: Module-level cache with TTL for namespace data

```typescript
// 30-second TTL cache for namespace list
let namespacesCache: { data: NamespaceWithCounts[]; timestamp: number } | null = null;
const CACHE_TTL = 30000;
```

Cache is invalidated on mutations (create, delete, update operations).

### 3. Non-Blocking Initialization

**Problem**: LLM and embedding engine initialization blocked app startup

**Solution**: Moved non-critical initialization to background after UI renders

```typescript
// Critical path (blocks UI):
initialize_app → check_fts5_enabled → get_vector_consent

// Background (after UI ready):
init_llm_engine + init_embedding_engine (parallel, with 5s timeout)
```

### 4. Component Memoization

**Problem**: Unnecessary re-renders of child components

**Solution**: Applied `useMemo` and `useCallback` to hooks returning stable references

```typescript
const actions = useMemo(() => ({
  loadNamespaces,
  selectNamespace,
  // ...
}), [/* deps */]);
```

---

## Monitoring & Profiling

### Development Profiling

Use React DevTools Profiler to identify unnecessary re-renders:

```bash
# In Chrome DevTools with React DevTools extension
# - Components tab → Profiler → Start profiling
# - Perform action → Stop profiling
# - Look for red bars (slow renders)
```

### Database Profiling

Enable SQLite query logging in development:

```rust
// In debug builds, log slow queries (>100ms)
#[cfg(debug_assertions)]
if elapsed.as_millis() > 100 {
    eprintln!("Slow query ({}ms): {}", elapsed.as_millis(), sql);
}
```

### Performance Regression Tests

```bash
# Run performance-related tests
cargo test --release -- --nocapture performance
```

---

## Future Optimization Opportunities

### Short-term (Phase 4 completion)

1. **Virtual list for large data sets** - Add `react-window` for document/chunk lists
2. **Pagination for database lists** - Add offset/limit to all `list_*` commands
3. **Search result caching** - Cache FTS results for identical queries (5min TTL)

### Medium-term

1. **Event-based model status** - Replace polling with Tauri events
2. **Incremental indexing** - Only re-index changed files
3. **WebWorker for heavy computation** - Offload text processing

### Long-term

1. **SQLite connection pooling** - For concurrent queries
2. **Lazy chunk loading** - Load chunks on scroll
3. **Background embedding generation** - Generate embeddings while idle

---

## Performance Checklist

### Before Release

- [ ] Cold launch under 2 seconds on baseline hardware
- [ ] No memory leaks (stable over 1 hour of use)
- [ ] Search returns in < 300ms for 10,000 chunk corpus
- [ ] Smooth scrolling with 5,000+ items in lists
- [ ] No frame drops during text generation

### Per-Commit

- [ ] Run `pnpm test` - all tests pass
- [ ] Run `cargo test --release` - all tests pass
- [ ] Check bundle size hasn't increased > 5%
- [ ] Verify no new console warnings in dev mode

---

## Baseline Hardware

Performance targets are measured against:

- MacBook Pro M1/M2/M3/M4 (8GB minimum)
- SSD storage
- macOS 13+

Lower-spec machines may experience degraded performance.
