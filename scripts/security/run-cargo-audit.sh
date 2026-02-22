#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root/src-tauri"

# Temporary advisory waiver set for transitive crates in the current Tauri stack.
# Owner: Platform
# Expires: 2026-03-01
# Mitigation issue: Unknown (must be created in Week 3 hardening)
cargo audit --deny warnings \
  --ignore RUSTSEC-2024-0411 \
  --ignore RUSTSEC-2024-0412 \
  --ignore RUSTSEC-2024-0413 \
  --ignore RUSTSEC-2024-0414 \
  --ignore RUSTSEC-2024-0415 \
  --ignore RUSTSEC-2024-0416 \
  --ignore RUSTSEC-2024-0417 \
  --ignore RUSTSEC-2024-0418 \
  --ignore RUSTSEC-2024-0419 \
  --ignore RUSTSEC-2024-0420 \
  --ignore RUSTSEC-2024-0429 \
  --ignore RUSTSEC-2024-0436 \
  --ignore RUSTSEC-2024-0370 \
  --ignore RUSTSEC-2025-0057 \
  --ignore RUSTSEC-2025-0075 \
  --ignore RUSTSEC-2025-0080 \
  --ignore RUSTSEC-2025-0081 \
  --ignore RUSTSEC-2025-0098 \
  --ignore RUSTSEC-2025-0100 \
  --ignore RUSTSEC-2026-0002
