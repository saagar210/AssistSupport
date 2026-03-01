#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root/src-tauri"

# Temporary advisory waiver set for transitive crates in current desktop/runtime
# dependencies while upstream remediation paths are tracked.
#
# Owner: Platform Engineering
# Review date: 2026-03-01
# Umbrella tracking issue: https://github.com/saagar210/AssistSupport/issues/11
#
# GTK3/Tauri Linux runtime chain (issue #12):
# - RUSTSEC-2024-0411 gdkwayland-sys
# - RUSTSEC-2024-0412 gdk
# - RUSTSEC-2024-0413 atk
# - RUSTSEC-2024-0415 gtk
# - RUSTSEC-2024-0416 atk-sys
# - RUSTSEC-2024-0418 gdk-sys
# - RUSTSEC-2024-0419 gtk3-macros
# - RUSTSEC-2024-0420 gtk-sys
# - RUSTSEC-2024-0429 glib
# - RUSTSEC-2024-0370 proc-macro-error
#
# tauri-utils/urlpattern chain (issue #13):
# - RUSTSEC-2025-0057 fxhash
# - RUSTSEC-2025-0075 unic-char-range
# - RUSTSEC-2025-0080 unic-common
# - RUSTSEC-2025-0081 unic-char-property
# - RUSTSEC-2025-0098 unic-ucd-version
# - RUSTSEC-2025-0100 unic-ucd-ident
#
# Lance/DataFusion chain (issue #14):
# - RUSTSEC-2024-0436 paste
#
# Tantivy/Lance chain (issue #15):
# - RUSTSEC-2026-0002 lru
# Deny unsound/unmaintained advisories but do not hard-fail on yanked crate warnings,
# which can fluctuate transitively outside this repo's direct control.
cargo audit --deny unsound --deny unmaintained \
  --ignore RUSTSEC-2024-0411 \
  --ignore RUSTSEC-2024-0412 \
  --ignore RUSTSEC-2024-0413 \
  --ignore RUSTSEC-2024-0415 \
  --ignore RUSTSEC-2024-0416 \
  --ignore RUSTSEC-2024-0418 \
  --ignore RUSTSEC-2024-0419 \
  --ignore RUSTSEC-2024-0420 \
  --ignore RUSTSEC-2024-0429 \
  --ignore RUSTSEC-2024-0370 \
  --ignore RUSTSEC-2024-0436 \
  --ignore RUSTSEC-2025-0057 \
  --ignore RUSTSEC-2025-0075 \
  --ignore RUSTSEC-2025-0080 \
  --ignore RUSTSEC-2025-0081 \
  --ignore RUSTSEC-2025-0098 \
  --ignore RUSTSEC-2025-0100 \
  --ignore RUSTSEC-2026-0002
