#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
# shellcheck source=common.sh
source "$script_dir/common.sh"

memorykernel_root=$(parse_memorykernel_root "${1:-}")

run_step "versioning policy documented" require_file "$memorykernel_root/docs/spec/versioning.md"
run_step "version bump rules enforced" require_grep "version bump" "$memorykernel_root/docs/spec/versioning.md"
run_step "release gate documented" require_file "$memorykernel_root/docs/implementation/trilogy-release-gate.md"
run_step "closeout playbook documented" require_file "$memorykernel_root/docs/implementation/trilogy-closeout-playbook.md"
run_step "change governance references integration v1 lock" require_grep "integration/v1" "$memorykernel_root/docs/spec/integration-contract.md"
