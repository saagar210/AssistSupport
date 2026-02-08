# Threat Model (Normative)

## Scope

- Local `SQLite` persistence
- Snapshot export/import flows
- Local service/CLI request surfaces

## Trust Boundaries

- Snapshot files (`manifest.json`, NDJSON artifacts, optional security sidecars) are untrusted input on import.
- CLI/service request payloads are untrusted input.
- Key files used for signing/encryption are trusted only if provided by the operator.

## Abuse Cases

1. Snapshot tampering:
   - Attacker modifies `manifest.json` or NDJSON content before import.
   - Control: signature verification and digest checks MUST fail import (`MKR-051`).
2. Downgrade to unsigned snapshot:
   - Attacker supplies unsigned snapshot to bypass verification.
   - Control: unsigned imports require explicit `--allow-unsigned` override.
3. Encrypted snapshot misuse:
   - Attacker supplies encrypted snapshot without key or with wrong key.
   - Control: import MUST fail unless a valid decrypt key is provided.
4. Malformed payload insertion:
   - Attacker injects invalid IDs/enums/timestamps.
   - Control: strict parsing and schema/domain validation reject invalid writes.

## Residual Risks

- Local key-file handling is operator-managed and outside process isolation controls.
- Pass-through local service usage assumes host-level trust and access control.
