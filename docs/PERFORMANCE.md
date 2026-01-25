# Performance Guide

## Baseline Metrics

These benchmarks were measured on Apple M4 Pro (48GB RAM):

### Encryption Performance
| Operation | 1KB | 64KB | 1MB |
|-----------|-----|------|-----|
| Encrypt | ~15 μs | ~200 μs | ~2.5 ms |
| Decrypt | ~12 μs | ~180 μs | ~2.2 ms |

Throughput: ~400 MB/s for AES-256-GCM

### Key Derivation (Argon2id)
- Key wrap: ~500 ms
- Key unwrap: ~500 ms

Note: This is intentionally slow for security (brute-force resistance).

### Database Operations
| Operation | Latency |
|-----------|---------|
| Open + Initialize | ~50 ms |
| Integrity Check | ~1 ms |
| Read Setting | ~0.1 ms |
| Write Setting | ~0.5 ms |

### FTS Search
| Query Type | 100 docs | 1,000 docs | 10,000 docs |
|------------|----------|------------|-------------|
| Simple | ~1 ms | ~5 ms | ~20 ms |
| Multi-word | ~2 ms | ~8 ms | ~30 ms |
| Phrase | ~2 ms | ~10 ms | ~40 ms |

### LLM Generation
Highly dependent on model and hardware:

| Model | Tokens/sec (M4 Pro) |
|-------|---------------------|
| Qwen 2.5 7B Q4 | ~30-40 |
| Llama 3.2 3B Q4 | ~50-70 |
| Phi-4 14B Q4 | ~15-25 |

## Performance Tuning

### RAM Management

#### Available RAM Calculation
AssistSupport checks available memory before loading models:
```
Available = Physical RAM - (System + Other Apps)
Recommended for 7B model: 8 GB free
Recommended for 14B model: 16 GB free
```

#### Reduce Memory Usage
1. Use smaller quantizations (Q4 vs Q8)
2. Reduce context window (2048 vs 4096)
3. Close other memory-intensive apps

### Context Window

#### Trade-offs
| Context Size | RAM Usage | Generation Speed |
|--------------|-----------|------------------|
| 2048 | Lower | Faster |
| 4096 | Medium | Standard |
| 8192 | Higher | Slower |

#### Recommendation
Start with 4096, reduce if experiencing memory pressure.

### Vector Search

#### Embedding Model
- **nomic-embed-text-v1.5**: 768-dim, ~300MB VRAM
- Batch size: 32 chunks per embedding call
- Index: LanceDB with IVF-PQ

#### Tuning Vector Search
```javascript
// Adjust hybrid search weights
search_kb_with_options({
  query: "...",
  fts_weight: 0.6,      // FTS contribution
  vector_weight: 0.4,   // Vector contribution
  min_score: 0.3        // Filter low-relevance results
})
```

### Database Optimization

#### Scheduled Maintenance
The diagnostics system includes automatic maintenance:
- **Fragmentation check**: If > 10%, VACUUM is recommended
- **Page count tracking**: Monitor growth over time

#### Manual Maintenance
```sql
-- Run from debug console
VACUUM;
ANALYZE;
```

#### Index Usage
FTS5 indices are maintained automatically. For large KBs:
- Consider namespace partitioning
- Remove stale sources periodically

### Indexing Performance

#### Chunk Size
Default: 500-1500 characters per chunk
- Smaller chunks: Better semantic precision, more vectors
- Larger chunks: Fewer vectors, less precise matching

#### Parallel Indexing
File indexing is single-threaded per file but multiple files can be queued. For large initial imports:
1. Index in batches of ~100 files
2. Allow embedding generation to complete between batches
3. Monitor memory usage

## Bottleneck Identification

### Common Bottlenecks

| Symptom | Likely Cause | Solution |
|---------|--------------|----------|
| Slow generation | Insufficient RAM | Smaller model/quantization |
| Slow search | Large vector index | Namespace filtering |
| Slow indexing | Many PDFs | Disable OCR for typed PDFs |
| High memory | Context window | Reduce context size |

### Monitoring Tools

#### Built-in Diagnostics
```typescript
// Get resource metrics
invoke('get_resource_metrics_cmd')
// Returns: { memory_used, memory_total, cpu_usage }

// Get database stats
invoke('get_database_stats_cmd')
// Returns: { file_size_bytes, document_count, chunk_count }
```

#### System Tools
- **Activity Monitor**: RAM and CPU per process
- **Instruments**: Detailed profiling (Xcode)
- **Console.app**: Application logs

## Model Selection Guide

### By Hardware

| RAM | Recommended Model |
|-----|-------------------|
| 8 GB | Llama 3.2 3B Q4 |
| 16 GB | Qwen 2.5 7B Q4 |
| 32 GB+ | Phi-4 14B Q4 or larger |

### By Use Case

| Use Case | Model | Why |
|----------|-------|-----|
| Quick responses | Llama 3.2 3B | Fast generation |
| Quality responses | Qwen 2.5 7B | Best balance |
| Complex analysis | Phi-4 14B | Higher capability |

## Running Benchmarks

### Backend Benchmarks
```bash
cd src-tauri
cargo bench
```

This runs:
- Encryption/decryption throughput
- Key generation
- Database operations
- FTS search latency

### Frontend Benchmarks
Use React DevTools Profiler to identify:
- Expensive re-renders
- Component mount times
- Effect timing

## Optimizations Applied

### Backend
- Non-blocking LLM initialization
- Connection pooling for database
- Lazy embedding model loading
- Result caching for repeated searches

### Frontend
- Component memoization (useMemo, useCallback)
- Virtual scrolling for large lists
- Debounced search input
- Optimistic UI updates
