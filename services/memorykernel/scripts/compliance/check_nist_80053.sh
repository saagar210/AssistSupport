#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
# shellcheck source=common.sh
source "$script_dir/common.sh"

memorykernel_root=$(parse_memorykernel_root "${1:-}")

run_step "threat model present" require_file "$memorykernel_root/docs/security/threat-model.md"
run_step "trust controls present" require_file "$memorykernel_root/docs/security/trust-controls.md"
run_step "append-only requirement documented" require_grep "MKR-002" "$memorykernel_root/docs/spec/requirements.md"
run_step "deterministic query requirement documented" require_grep "MKR-012" "$memorykernel_root/docs/spec/requirements.md"
run_step "contract parity requirement documented" require_grep "MKR-057" "$memorykernel_root/docs/spec/requirements.md"
run_step "audit/replay surface documented" require_file "$memorykernel_root/components/multi-agent-center/docs/trace-schema.md"
