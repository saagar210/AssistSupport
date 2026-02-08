# Backup and Export Trust Controls (Normative)

## Snapshot Signing

- Export supports `--signing-key-file` for `manifest.json` signatures.
- Signature file: `manifest.sig`
- Algorithm identifier: `hmac-sha256`
- Import verification:
  - Signed snapshots require `--verify-key-file`.
  - Verification failure MUST abort import.

## Snapshot Encryption

- Export supports `--encrypt-key-file` for snapshot payload encryption.
- Encryption metadata file: `manifest.security.json`
- Encryption algorithm identifier: `xchacha20poly1305`
- Import decryption:
  - Encrypted snapshots require `--decrypt-key-file`.
  - Decryption failure MUST abort import.

## Unsigned Import Policy

- Default behavior rejects unsigned snapshots.
- Operator override is explicit: `--allow-unsigned`.

## Operational Guidance

- Use distinct keys for signing and encryption.
- Store keys outside the repository and rotate on compromise.
- Treat snapshot directories as untrusted until verification succeeds.
