#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
# shellcheck source=common.sh
source "$script_dir/common.sh"

memorykernel_root=$(parse_memorykernel_root "${1:-}")

run_step "encryption controls documented" require_grep "Encryption" "$memorykernel_root/docs/security/trust-controls.md"
run_step "signature verification controls documented" require_grep "signature" "$memorykernel_root/docs/security/trust-controls.md"
run_step "provenance requirements documented" require_grep "source_uri" "$memorykernel_root/docs/spec/domain.md"
run_step "context package includes trust semantics" require_grep "truth_status" "$memorykernel_root/docs/spec/context-package.md"
run_step "audit/replay substrate present" require_file "$memorykernel_root/components/multi-agent-center/docs/trace-schema.md"
