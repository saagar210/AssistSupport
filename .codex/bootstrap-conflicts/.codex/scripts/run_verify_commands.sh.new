#!/usr/bin/env bash
set -euo pipefail

COMMANDS_FILE="${1:-.codex/verify.commands}"

if [[ ! -f "$COMMANDS_FILE" ]]; then
  echo "Missing $COMMANDS_FILE"
  exit 2
fi

while IFS= read -r cmd || [[ -n "$cmd" ]]; do
  [[ -z "${cmd//[[:space:]]/}" ]] && continue
  [[ "$cmd" =~ ^[[:space:]]*# ]] && continue

  echo ">> $cmd"
  eval "$cmd"
done < "$COMMANDS_FILE"
