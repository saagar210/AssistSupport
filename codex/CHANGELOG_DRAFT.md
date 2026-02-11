# Changelog Draft

## Theme: Queue Metadata Resilience

### What changed
- Hardened `safeParseQueueMeta` to normalize persisted metadata entries before use, including:
  - strict `state` fallback to `open`
  - strict `priority` fallback to `normal`
  - owner trimming with fallback to `unassigned`
  - timestamp validity checks with safe empty fallback
  - non-object entry filtering
- Added regression coverage for malformed-but-parseable queue metadata payloads.

### Why it changed
- Persisted localStorage payloads can become malformed via manual edits/legacy states; previously parsed objects were trusted without shape validation, risking inconsistent queue calculations.

### User-visible behavior changes
- Queue views/summaries now degrade more safely when local queue metadata is malformed, instead of propagating invalid state/priority values.
- No behavior change for valid metadata payloads.
