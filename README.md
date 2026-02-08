# MultiAgentCenter

Controlled execution layer for stateless agents with permissioned memory access and replayable audit traces.

Memory context retrieval supports both deterministic policy queries and deterministic recall queries
through `task.context_queries[].mode` (`policy` or `recall`).

## Quick Start

```bash
cargo run -p multi-agent-center-cli -- run --workflow examples/workflow.mock.yaml --trace-db /tmp/mac-trace.sqlite --non-interactive
cargo run -p multi-agent-center-cli -- replay --trace-db /tmp/mac-trace.sqlite --run-id <RUN_ID>
```
