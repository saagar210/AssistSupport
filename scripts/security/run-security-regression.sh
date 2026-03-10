#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

mkdir -p "$repo_root/dist"
if [[ ! -f "$repo_root/dist/index.html" ]]; then
  printf '<html></html>\n' > "$repo_root/dist/index.html"
fi

cd "$repo_root/src-tauri"
cargo test security -- --nocapture
cargo test ssrf_dns_rebinding -- --nocapture
cargo test filter_injection -- --nocapture
cargo test path_validation -- --nocapture
