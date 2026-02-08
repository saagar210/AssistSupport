#!/usr/bin/env bash
set -euo pipefail

PACK_VERSION="${1:-v1}"
LOCAL_ROOT="${OUTCOME_CONTRACT_PACK_ROOT:-contracts/integration}"
if [[ -n "${MEMORYKERNEL_CONTRACT_PACK_ROOT:-}" ]]; then
  CANONICAL_ROOT="${MEMORYKERNEL_CONTRACT_PACK_ROOT}"
elif [[ -d "../../../contracts/integration" ]]; then
  CANONICAL_ROOT="../../../contracts/integration"
else
  CANONICAL_ROOT="../MemoryKernel/contracts/integration"
fi

LOCAL_PACK="${LOCAL_ROOT}/${PACK_VERSION}"
CANONICAL_PACK="${CANONICAL_ROOT}/${PACK_VERSION}"

if [[ ! -d "${LOCAL_PACK}" ]]; then
  echo "error: missing local contract pack directory: ${LOCAL_PACK}" >&2
  exit 1
fi

if [[ ! -d "${CANONICAL_PACK}" ]]; then
  echo "error: missing MemoryKernel canonical pack directory: ${CANONICAL_PACK}" >&2
  exit 1
fi

DIFF_FILE="$(mktemp)"
trap 'rm -f "${DIFF_FILE}"' EXIT

if ! diff -ru --exclude ".DS_Store" "${CANONICAL_PACK}" "${LOCAL_PACK}" >"${DIFF_FILE}"; then
  echo "error: integration contract pack drift detected for ${PACK_VERSION}" >&2
  echo "v1 contracts are frozen; drift requires an explicit version bump (for example, v2)." >&2
  echo "diff (canonical -> local):" >&2
  cat "${DIFF_FILE}" >&2
  exit 1
fi

echo "contract parity OK: ${PACK_VERSION}"
echo "local=${LOCAL_PACK}"
echo "canonical=${CANONICAL_PACK}"
