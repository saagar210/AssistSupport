#!/usr/bin/env bash
set -euo pipefail

root="public"
limit_bytes=${ASSET_MAX_BYTES:-350000}
fail=0

if [[ ! -d "$root" ]]; then
  echo "No public assets directory found; skipping."
  exit 0
fi

while IFS= read -r file; do
  size="$(wc -c <"$file")"
  if ((size > limit_bytes)); then
    echo "Asset too large (>${limit_bytes}B): $file"
    fail=1
  fi
done < <(find "$root" -type f \( -name "*.png" -o -name "*.jpg" -o -name "*.jpeg" -o -name "*.webp" -o -name "*.avif" -o -name "*.svg" \))

exit "$fail"

