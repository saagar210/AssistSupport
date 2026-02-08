# Service SLO Policy (Normative)

## Scope

This policy governs benchmark-based service performance guardrails for MemoryKernel release and CI gates.

## Canonical Policy File

- `contracts/integration/v1/service-slo-policy.json`

## Required Benchmark Profile

- Command family: `cargo run -p memory-kernel-cli -- outcome benchmark run`
- Workloads: `100`, `500`, `2000`
- Repetitions per workload: `3`
- Required output mode: `--json`

## Latency Guardrails (p95, milliseconds)

- `append_p95_max`: `8`
- `replay_p95_max`: `250`
- `gate_p95_max`: `8`

## Enforcement

- CI and release workflows MUST use the same guardrail values as the canonical policy file.
- Policy drift between `service-slo-policy.json` and workflow gate commands is a release blocker.
