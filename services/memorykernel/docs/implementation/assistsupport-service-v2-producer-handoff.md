# AssistSupport Producer Handoff (`service.v2`)

## Purpose

This note records the producer-side contract guarantees that AssistSupport consumes for the
service-first integration path.

## Pinned Baseline

- Release tag: `v0.3.0`
- Commit: `b9e1b397558dfba1fa8a4948fcf723ed4b505e1c`
- Service contract: `service.v2`
- API envelope contract: `api.v1`
- Integration baseline: `integration/v1`

## Stable Error Envelope Policy (`service.v2`)

For non-2xx responses, MemoryKernel guarantees:

- Present: `service_contract_version`
- Present: `error.code`
- Present: `error.message`
- Optional: `error.details`
- Present: `legacy_error` (transition-safe mirror of `error.message`)
- Absent: `api_contract_version`

`legacy_error` remains mandatory for the full `service.v2` lifecycle and is only eligible for
removal in a `service.v3` contract bump with explicit migration guidance.

## Consumer Expectations (AssistSupport)

- Keep `error.code` as primary machine-readable branch key.
- Keep `legacy_error` parsing enabled for transition safety during `service.v2`.
- Keep deterministic fallback behavior for offline, timeout, malformed payload, version mismatch,
  and non-2xx responses.
- Keep pin + compatibility matrix synchronized and validated in CI.

## Producer Gates That Protect This Contract

- `./scripts/verify_service_contract_alignment.sh`
- `cargo test --workspace --all-targets --all-features` (includes `TSVC-009`)
- CI workflow `Service contract alignment` step
- Release workflow `Service contract alignment` step

