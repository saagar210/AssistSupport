# Contract Fixtures

This directory stores canonical request/response fixtures for high-risk integration boundaries:

- `contracts/tauri/v1/*` for Rust Tauri command payloads consumed by the frontend.
- `contracts/search-api/v1/*` for Search API HTTP envelope shapes consumed by the Tauri layer.
- `contracts/tauri/v1/command-lifecycle.json` for command lifecycle/deprecation policy metadata.

These fixtures are treated as compatibility contracts during migration work:

- Rust fixture gate: `pnpm run test:contracts:rust`
- Search API fixture gate: included in `search-api/tests/test_contract_fixtures.py`
- JSON fixture sanity gate: `pnpm run check:contract-gate`
- Command lifecycle policy gate: `pnpm run check:command-lifecycle`

When payload shape changes are intentional, update fixtures and tests in the same change.
