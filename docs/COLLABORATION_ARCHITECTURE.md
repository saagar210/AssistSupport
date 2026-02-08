# Phase 23: Collaboration Architecture

**Status:** Design Only -- Not Implemented
**Author:** VaultMind Engineering
**Created:** 2026-02-08
**Last Updated:** 2026-02-08
**Schema Version Baseline:** v5

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Design Principles](#2-design-principles)
3. [Architecture Overview](#3-architecture-overview)
4. [Peer Discovery (mDNS)](#4-peer-discovery-mdns)
5. [Transport Layer](#5-transport-layer)
6. [Authentication and Trust](#6-authentication-and-trust)
7. [Sync Protocol](#7-sync-protocol)
8. [Conflict Resolution](#8-conflict-resolution)
9. [Privacy Model](#9-privacy-model)
10. [Database Schema Extensions](#10-database-schema-extensions)
11. [Security Considerations](#11-security-considerations)
12. [Crate Dependencies](#12-crate-dependencies)
13. [Implementation Phases](#13-implementation-phases)
14. [Appendix: Wire Protocol Specification](#14-appendix-wire-protocol-specification)

---

## 1. Executive Summary

VaultMind is a local-first, privacy-focused AI knowledge management application. Every byte of user data lives on the user's device. This Phase 23 design extends VaultMind with optional peer-to-peer collaboration on the local network while preserving that core guarantee: **no data leaves the device without explicit user action.**

The collaboration system enables two or more VaultMind instances on the same LAN to:

- Discover each other automatically via mDNS (Bonjour/Zeroconf).
- Establish mutually authenticated, TLS-encrypted connections.
- Sync explicitly shared collections -- metadata via CRDTs, document content via full re-ingestion on the receiving side.
- Operate entirely offline with zero cloud dependencies.

This document is design-only. It specifies the architecture, protocols, schema extensions, and security model. No implementation code is included.

---

## 2. Design Principles

| Principle | Implication |
|---|---|
| **Local-first** | All data processing happens on-device. Sync is additive -- receiving a document means re-parsing and re-embedding it locally with the receiver's own Ollama models. |
| **Explicit sharing** | Collections are private by default. A user must explicitly mark a collection as shared and approve each peer. |
| **Zero trust network** | Even on a home LAN, all traffic is TLS 1.3 encrypted. Peers must be mutually authenticated before any data exchange. |
| **Offline-capable** | Sync is opportunistic. Peers sync when they see each other and resume gracefully after disconnection. |
| **No central coordinator** | There is no server, relay, or cloud service. Every peer is equal. |
| **Privacy by design** | Embeddings are never transferred. The receiver re-embeds using their own model configuration. This prevents model fingerprinting and ensures local model independence. |

---

## 3. Architecture Overview

```
+----------------------------------------------------------------------+
|                          VaultMind Instance A                        |
|                                                                      |
|  +------------+    +---------------+    +-------------------------+  |
|  | Collection |    | Sync Engine   |    | Peer Manager            |  |
|  | Manager    |--->| (CRDT merge + |--->| (discovery, auth,       |  |
|  |            |    |  doc transfer)|    |  connection lifecycle)   |  |
|  +------------+    +-------+-------+    +------------+------------+  |
|                            |                         |               |
|                    +-------v-------+         +-------v-------+       |
|                    | SQLite DB     |         | TLS Transport |       |
|                    | (local state) |         | (rustls)      |       |
|                    +---------------+         +-------+-------+       |
+----------------------------------------------|-------+---------------+
                                               |  LAN (mDNS + TLS)
+----------------------------------------------|-------+---------------+
|                          VaultMind Instance B |       |               |
|                                        +------v-------v-------+      |
|                                        | Peer Manager         |      |
|                                        | + TLS Transport      |      |
|                                        +-----------+----------+      |
|                                                    |                 |
|                                        +-----------v----------+      |
|                                        | Sync Engine          |      |
|                                        | (CRDT merge +        |      |
|                                        |  doc re-ingestion)   |      |
|                                        +----------------------+      |
+----------------------------------------------------------------------+
```

### Component Responsibilities

| Component | Role |
|---|---|
| **Peer Manager** | Runs mDNS advertisement/browsing. Manages the lifecycle of peer connections (discovered, authenticating, trusted, connected, disconnected). Maintains the `peers` table. |
| **TLS Transport** | Wraps all peer communication in TLS 1.3 via `rustls`. Handles connection setup, certificate exchange, and channel multiplexing. |
| **Sync Engine** | Orchestrates the sync protocol: CRDT metadata exchange, document manifest comparison, content transfer, and sync log bookkeeping. |
| **Collection Manager** | Existing component. Extended to expose sharing controls -- marking collections as shared and assigning a stable `sync_id`. |

---

## 4. Peer Discovery (mDNS)

### Protocol

VaultMind uses mDNS (RFC 6762) with DNS-SD (RFC 6763) for zero-configuration peer discovery on the local network. The `mdns-sd` crate provides cross-platform support (macOS Bonjour, Linux Avahi, Windows mDNS).

### Service Registration

Each VaultMind instance registers a service:

```
Service Type:  _vaultmind._tcp.local.
Service Name:  <display_name>._vaultmind._tcp.local.
Port:          <dynamic, default 9847>
TXT Records:
    version=1
    peer_id=<uuid>
    fingerprint=<first 8 chars of public key SHA-256>
```

### Discovery Flow

```
  Instance A                        mDNS (LAN)                    Instance B
  ----------                        ----------                    ----------
      |                                  |                             |
      |-- Register service ------------->|                             |
      |                                  |<----------- Register -------|
      |                                  |                             |
      |-- Browse _vaultmind._tcp ------->|                             |
      |<-- Service found (Instance B) ---|                             |
      |                                  |--- Service found (A) ------>|
      |                                  |                             |
      |  [Add to peers table as          |    [Add to peers table as   |
      |   untrusted, show in UI]         |     untrusted, show in UI]  |
      |                                  |                             |
```

### Peer Lifecycle States

```
DISCOVERED --> AUTHENTICATING --> TRUSTED --> CONNECTED
     |               |                           |
     |               v                           v
     +--------> REJECTED                   DISCONNECTED
                                                |
                                                v
                                           CONNECTED (auto-reconnect)
```

- **DISCOVERED:** Seen on mDNS, not yet authenticated. Visible in UI as "Available peer."
- **AUTHENTICATING:** User initiated pairing. Waiting for code/QR exchange.
- **TRUSTED:** Authentication succeeded. `peers.trusted = 1`. Will auto-connect on future sightings.
- **CONNECTED:** Active TLS session. Sync can occur.
- **DISCONNECTED:** Trusted peer not currently reachable. Will auto-reconnect when seen again.
- **REJECTED:** User explicitly declined. Hidden from UI unless reset.

### Stale Peer Cleanup

Peers not seen for 30 days are flagged in the UI. Peers not seen for 90 days are suggested for removal. No automatic deletion -- user action required.

---

## 5. Transport Layer

### Primary: TLS 1.3 over TCP

All peer communication uses TLS 1.3 (via `rustls`) over persistent TCP connections.

| Property | Value |
|---|---|
| TLS Version | 1.3 only (no fallback to 1.2) |
| Cipher Suites | TLS_AES_256_GCM_SHA384, TLS_CHACHA20_POLY1305_SHA256 |
| Certificate Type | Self-signed Ed25519 (generated on first launch) |
| Mutual Authentication | Both peers present certificates |
| Session Resumption | TLS 1.3 PSK for reconnection efficiency |

### Connection Setup

```
  Instance A                                           Instance B
  ----------                                           ----------
      |                                                     |
      |--- TCP SYN (port from mDNS TXT) ------------------>|
      |<-- TCP SYN-ACK ------------------------------------|
      |                                                     |
      |--- TLS 1.3 ClientHello (with Ed25519 cert) ------->|
      |<-- TLS 1.3 ServerHello (with Ed25519 cert) --------|
      |                                                     |
      |  [Both sides verify peer fingerprint against        |
      |   trusted peers table]                              |
      |                                                     |
      |--- VaultMind Handshake (protocol version, caps) --->|
      |<-- VaultMind Handshake ACK -------------------------|
      |                                                     |
      |  [Connection established, sync can begin]           |
      |                                                     |
```

### Alternative: QUIC via `quinn` (Future)

QUIC (HTTP/3 transport) provides advantages for unreliable networks:
- Built-in connection migration (device switches Wi-Fi networks).
- Multiplexed streams without head-of-line blocking.
- 0-RTT reconnection for trusted peers.

QUIC is not in the initial implementation scope but the protocol is designed to be transport-agnostic. The `quinn` crate is listed as a future dependency.

### Keep-Alive and Reconnection

- TCP keep-alive every 30 seconds while connected.
- On disconnect, exponential backoff reconnection: 1s, 2s, 4s, 8s, 16s, 30s max.
- Reconnection only attempted for trusted peers seen on mDNS in the last 5 minutes.

---

## 6. Authentication and Trust

### Trust Establishment

VaultMind uses a pre-shared key (PSK) model. Two trust establishment methods are supported:

#### Method 1: QR Code Scan

1. Instance A generates a pairing payload: `{ peer_id, public_key_fingerprint, one_time_code }`.
2. Payload is encoded as a QR code displayed on screen.
3. Instance B scans the QR code (via webcam or screenshot).
4. Instance B sends a pairing request to Instance A containing its own `peer_id` and `public_key_fingerprint`, encrypted with the `one_time_code`.
5. Instance A verifies the code and both sides mark each other as trusted.

#### Method 2: 6-Digit Code Entry

1. Instance A displays a 6-digit numeric code (TOTP-style, valid for 60 seconds).
2. User reads the code and enters it on Instance B.
3. Instance B sends a pairing request containing the code and its identity.
4. Instance A verifies the code and both sides exchange public key fingerprints.
5. Both mark each other as trusted.

### Pairing Sequence Diagram

```
  Instance A (Initiator)                        Instance B (Joiner)
  ----------------------                        -------------------
      |                                              |
      |  [User clicks "Pair New Device"]             |
      |  [Displays QR + 6-digit code: 847293]        |
      |                                              |
      |                    [User enters 847293 on B] |
      |                                              |
      |<--- PairingRequest { code: 847293,           |
      |       peer_id: B, fingerprint: B_fp } -------|
      |                                              |
      |  [Verify code matches]                       |
      |  [Show on A: "Device 'B_name' wants to       |
      |   pair. Accept?"]                            |
      |                                              |
      |  [User clicks Accept]                        |
      |                                              |
      |--- PairingAccepted { peer_id: A,             |
      |      fingerprint: A_fp } ------------------->|
      |                                              |
      |  [Both store peer as trusted]                |
      |  [Both show "Paired successfully"]           |
      |                                              |
```

### Trust Verification

After initial pairing, peers verify each other on every TLS connection by comparing the presented certificate's fingerprint against the stored `peers.public_key` value. A mismatch triggers an immediate disconnection and a user-visible warning.

### Trust Revocation

A user can untrust a peer at any time. This:
1. Sets `peers.trusted = 0`.
2. Immediately terminates any active connection.
3. Removes all sharing permissions for that peer.
4. Does NOT delete previously synced data (that data is now local to the receiver).

---

## 7. Sync Protocol

### Overview

The sync protocol has two layers:

1. **Metadata Sync (CRDT-based):** Collection names, descriptions, document metadata (filename, title, author, page count, word count). Uses operation-based CRDTs for convergent, conflict-free updates.
2. **Content Sync (Full Transfer):** Document files are transferred in their entirety. The receiver re-parses, re-chunks, and re-embeds using its own pipeline. This ensures local model independence.

### Sync Initiation

Sync occurs:
- Immediately upon connection establishment (initial sync).
- Incrementally when either peer detects a change to a shared collection (change notification).
- On user request ("Sync Now" button).

### Initial Sync Handshake

```
  Instance A                                        Instance B
  ----------                                        ----------
      |                                                  |
      |--- SyncInit { shared_collections: [             |
      |      { sync_id: "abc", version_vector: {...} }, |
      |      { sync_id: "def", version_vector: {...} }  |
      |    ] } ----------------------------------------->|
      |                                                  |
      |  [B compares against its own shared collections  |
      |   and computes the delta]                        |
      |                                                  |
      |<-- SyncManifest { collections: [                 |
      |      { sync_id: "abc",                           |
      |        metadata_ops: [...],                      |
      |        new_documents: ["doc1", "doc3"],           |
      |        removed_documents: ["doc2"] }             |
      |    ] } ------------------------------------------|
      |                                                  |
      |  [A applies CRDT ops for metadata]               |
      |  [A requests content for new documents]          |
      |                                                  |
      |--- ContentRequest { documents: ["doc1","doc3"] } |
      |---------------------------------------------->   |
      |                                                  |
      |<-- ContentStream { doc_id: "doc1",               |
      |      filename: "paper.pdf",                      |
      |      chunks: [...raw file bytes...] }            |
      |<-- ContentStream { doc_id: "doc3", ... }         |
      |                                                  |
      |  [A saves files locally, runs full ingestion     |
      |   pipeline: parse -> chunk -> embed]             |
      |                                                  |
      |--- SyncComplete { status: "ok",                  |
      |      synced: 2, failed: 0 } ------------------->|
      |                                                  |
```

### CRDT Strategy for Metadata

Each shared collection maintains an **operation log** using Grow-Only Sets (G-Sets) and Last-Writer-Wins Registers (LWW-Registers):

| Field | CRDT Type | Rationale |
|---|---|---|
| Collection name | LWW-Register | Single value, last edit wins |
| Collection description | LWW-Register | Single value, last edit wins |
| Document set membership | Observed-Remove Set (OR-Set) | Documents can be added and removed concurrently |
| Document metadata (title, author, etc.) | LWW-Register per field | Each metadata field is independently mergeable |

Each operation carries a **Hybrid Logical Clock (HLC)** timestamp combining wall-clock time and a logical counter. This provides causal ordering without requiring synchronized clocks.

```
HLC Timestamp: {
    wall_time: u64,    // milliseconds since epoch
    counter: u32,      // logical counter for same-millisecond events
    node_id: [u8; 8],  // first 8 bytes of peer_id for tie-breaking
}
```

### Incremental Sync

After the initial handshake, peers maintain a **version vector** per shared collection. Changes are streamed incrementally:

```
  Instance A                              Instance B
  ----------                              ----------
      |                                        |
      |  [User renames doc in shared           |
      |   collection]                          |
      |                                        |
      |--- IncrementalOp {                     |
      |      collection_sync_id: "abc",        |
      |      op: MetadataUpdate {              |
      |        doc_sync_id: "doc1",            |
      |        field: "title",                 |
      |        value: "New Title",             |
      |        hlc: { wall: ..., ctr: 1,       |
      |               node: A }                |
      |      }                                 |
      |    } ------------------------------>   |
      |                                        |
      |<-- IncrementalAck { hlc: ... } --------|
      |                                        |
```

### Content Transfer Protocol

Document content is transferred as raw file bytes (the original PDF, DOCX, etc.). The transfer uses chunked streaming to handle large files:

1. Sender computes SHA-256 hash of the file.
2. Sender streams the file in 64KB chunks over the TLS connection.
3. Receiver reassembles and verifies the hash.
4. Receiver runs the full ingestion pipeline: parse, chunk, embed.
5. Receiver creates local database records with local IDs but references the shared `sync_id` for future deduplication.

Embeddings, graph edges, and FTS indexes are **never transferred** -- they are always generated locally. This is a core privacy guarantee: the receiver's AI processing is fully independent.

---

## 8. Conflict Resolution

### Metadata Conflicts

**Strategy: Last-Writer-Wins (LWW) with HLC ordering.**

When two peers concurrently modify the same metadata field, the operation with the higher HLC timestamp wins. If HLC timestamps are identical (same wall time and counter), the `node_id` component provides deterministic tie-breaking.

This is acceptable for metadata because:
- Collection names and descriptions are low-contention fields.
- Document metadata changes are infrequent and typically driven by a single user.

### Content Conflicts

**Strategy: No merge -- content is always re-ingested fresh.**

If both peers modify the same document (identified by `sync_id`):
1. Both versions are kept as separate documents on the receiver.
2. The receiver sees both in the UI with a "conflict" indicator.
3. The user manually resolves by choosing one version, or keeping both.

This avoids the unsolvable problem of merging binary document formats (PDFs, DOCX) and aligns with the local-first principle: the user is always in control.

### Document Deletion

Deletions use OR-Set semantics. A delete operation propagates as a tombstone with an HLC timestamp. A concurrent add-and-delete resolves in favor of the add (add-wins semantics), which is the safer default for a knowledge management tool -- it is better to have a duplicate than to lose data.

---

## 9. Privacy Model

### Core Guarantees

1. **Private by default.** All collections are private (`shared = 0`) until the user explicitly enables sharing.
2. **Per-collection granularity.** Sharing is configured at the collection level, not the document or instance level.
3. **Explicit peer approval.** Even for shared collections, data only syncs to trusted peers.
4. **No ambient sharing.** Discovering a peer on mDNS does not expose any data. Trust must be established first.
5. **Local AI processing.** Embeddings, graph edges, and FTS indexes are generated locally. A peer cannot infer the receiver's model configuration.

### Sharing Permissions Matrix

| Sharing State | Metadata Visible to Peer | Documents Transferable | Sync Active |
|---|---|---|---|
| `shared = 0` (default) | No | No | No |
| `shared = 1`, no trusted peers | No (no one to share with) | No | No |
| `shared = 1`, trusted peer connected | Yes (collection name, doc list) | Yes (on request) | Yes |
| `shared = 1`, peer untrusted | Immediately stops | No | Terminated |

### Data Flow Audit

Every sync operation is recorded in the `sync_log` table:
- **Direction:** `outbound` (sent to peer) or `inbound` (received from peer).
- **Entity type:** `collection_metadata`, `document_metadata`, `document_content`.
- **Status:** `success`, `failed`, `rejected`.

The existing `audit_log` table (migration v2) is also extended to log sync events with action types: `sync.initiated`, `sync.completed`, `sync.failed`, `peer.trusted`, `peer.untrusted`, `collection.shared`, `collection.unshared`.

### What Is Never Shared

| Data Type | Shared? | Rationale |
|---|---|---|
| Embeddings (chunk_embeddings) | Never | Generated locally with user's model |
| Graph edges | Never | Derived from local processing |
| FTS indexes | Never | Built locally from chunks |
| Conversations and messages | Never | Private to the user's interactions |
| Citations | Never | Tied to local conversations |
| Search history | Never | Private usage patterns |
| Settings | Never | Local configuration |
| Entities and entity relationships | Never | Derived from local NER processing |

---

## 10. Database Schema Extensions

The following schema changes are required for collaboration support. These will be implemented as migration v6 when Phase 23 moves to implementation.

**Current schema version:** v5

### Migration v6: Collaboration Tables

```sql
-- Extend collections with sharing metadata
ALTER TABLE collections ADD COLUMN shared INTEGER DEFAULT 0;
ALTER TABLE collections ADD COLUMN sync_id TEXT;
-- sync_id is a stable UUID that identifies this collection across peers.
-- It is generated when sharing is first enabled and never changes.

-- Peers discovered and/or trusted on the local network
CREATE TABLE peers (
    id TEXT PRIMARY KEY,                  -- UUID, matches the peer's self-reported peer_id
    display_name TEXT NOT NULL,           -- Human-readable name (e.g., "Dan's MacBook")
    hostname TEXT NOT NULL,               -- mDNS hostname
    port INTEGER NOT NULL,               -- TCP port from mDNS TXT record
    public_key TEXT NOT NULL,             -- PEM-encoded Ed25519 public key
    fingerprint TEXT NOT NULL,            -- SHA-256 of public key, hex-encoded
    last_seen_at TEXT,                    -- ISO 8601 timestamp of last mDNS sighting
    trusted INTEGER DEFAULT 0,           -- 0 = discovered only, 1 = trusted/paired
    trusted_at TEXT,                      -- When trust was established
    status TEXT DEFAULT 'discovered',     -- discovered | authenticating | trusted | rejected
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_peers_trusted ON peers(trusted);
CREATE INDEX idx_peers_status ON peers(status);

-- Log of all sync operations for auditability
CREATE TABLE sync_log (
    id TEXT PRIMARY KEY,                  -- UUID
    peer_id TEXT NOT NULL REFERENCES peers(id) ON DELETE CASCADE,
    collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    direction TEXT NOT NULL,              -- 'inbound' or 'outbound'
    entity_type TEXT NOT NULL,            -- 'collection_metadata', 'document_metadata', 'document_content'
    entity_id TEXT,                       -- The specific document or collection sync_id
    status TEXT NOT NULL,                 -- 'success', 'failed', 'rejected', 'in_progress'
    bytes_transferred INTEGER DEFAULT 0, -- For content transfers
    error_message TEXT,                   -- Populated on failure
    created_at TEXT NOT NULL
);

CREATE INDEX idx_sync_log_peer ON sync_log(peer_id);
CREATE INDEX idx_sync_log_collection ON sync_log(collection_id);
CREATE INDEX idx_sync_log_created ON sync_log(created_at);

-- CRDT operation log for metadata convergence
CREATE TABLE crdt_operations (
    id TEXT PRIMARY KEY,                  -- UUID
    collection_sync_id TEXT NOT NULL,     -- References collections.sync_id
    entity_type TEXT NOT NULL,            -- 'collection', 'document'
    entity_sync_id TEXT,                  -- Specific entity's sync_id
    field_name TEXT NOT NULL,             -- 'name', 'description', 'title', etc.
    field_value TEXT,                     -- JSON-encoded value
    hlc_wall_time INTEGER NOT NULL,      -- HLC wall clock component (ms since epoch)
    hlc_counter INTEGER NOT NULL,        -- HLC logical counter
    hlc_node_id TEXT NOT NULL,           -- Originating peer_id (first 16 hex chars)
    applied INTEGER DEFAULT 0,           -- Whether this op has been applied locally
    received_from TEXT,                   -- peer_id that sent this op (NULL if local)
    created_at TEXT NOT NULL
);

CREATE INDEX idx_crdt_ops_collection ON crdt_operations(collection_sync_id);
CREATE INDEX idx_crdt_ops_entity ON crdt_operations(entity_sync_id);
CREATE INDEX idx_crdt_ops_hlc ON crdt_operations(hlc_wall_time, hlc_counter);

-- Per-collection per-peer version vectors for incremental sync
CREATE TABLE version_vectors (
    collection_sync_id TEXT NOT NULL,
    peer_id TEXT NOT NULL,
    last_hlc_wall_time INTEGER NOT NULL,
    last_hlc_counter INTEGER NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (collection_sync_id, peer_id)
);

-- Extend documents with sync identity
ALTER TABLE documents ADD COLUMN sync_id TEXT;
-- sync_id identifies this document across peers for deduplication.
-- Generated from file_hash + collection_sync_id for deterministic matching.
```

### Relationship to Existing Schema

```
+-------------------+          +-------------------+
| collections       |          | peers             |
|-------------------|          |-------------------|
| id (PK)           |          | id (PK)           |
| name              |          | display_name       |
| description       |          | hostname           |
| shared (NEW)      |          | port               |
| sync_id (NEW)     |          | public_key         |
| created_at        |          | fingerprint        |
| updated_at        |          | trusted            |
+--------+----------+          | status             |
         |                     +--------+-----------+
         |                              |
         |    +-------------------------+
         |    |
         v    v
+-------------------+          +-------------------+
| sync_log          |          | crdt_operations   |
|-------------------|          |-------------------|
| id (PK)           |          | id (PK)           |
| peer_id (FK)      |          | collection_sync_id|
| collection_id (FK)|          | entity_type        |
| direction         |          | field_name         |
| entity_type       |          | field_value        |
| entity_id         |          | hlc_wall_time      |
| status            |          | hlc_counter        |
| bytes_transferred |          | hlc_node_id        |
| created_at        |          | applied            |
+-------------------+          +-------------------+

+-------------------+
| version_vectors   |
|-------------------|
| collection_sync_id|
| peer_id            |
| last_hlc_wall_time |
| last_hlc_counter   |
+-------------------+
```

---

## 11. Security Considerations

### Threat Model

| Threat | Mitigation |
|---|---|
| **Eavesdropping on LAN** | All traffic TLS 1.3 encrypted. No plaintext data on the wire. |
| **Man-in-the-middle** | Mutual TLS authentication. Peers verify certificate fingerprints against stored values. |
| **Rogue mDNS advertisement** | mDNS discovery alone grants zero access. Trust must be established out-of-band (QR/code). |
| **Replay attacks** | HLC timestamps and TLS session state prevent replay of sync operations. |
| **Data exfiltration via malicious peer** | Only explicitly shared collections are visible. Peer trust can be revoked instantly. |
| **Compromised peer device** | Trust revocation cuts access. Previously synced data is already local to the receiver and cannot be "un-synced." |
| **Denial of service (LAN flooding)** | Rate limiting on incoming connections. Max 8 concurrent peers. Connection throttling per IP. |

### Cryptographic Details

| Component | Algorithm | Key Size |
|---|---|---|
| Peer identity | Ed25519 | 256-bit |
| TLS cipher | AES-256-GCM or ChaCha20-Poly1305 | 256-bit |
| Key fingerprint | SHA-256 of public key | 256-bit (displayed as first 8 hex chars for human verification) |
| File integrity | SHA-256 | 256-bit |
| Pairing code | Cryptographically random | 6 decimal digits (approx. 20 bits of entropy, acceptable for 60-second window) |

### Key Management

- **Key generation:** Ed25519 keypair generated on first application launch. Stored in the OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service) via `keyring` crate.
- **Key rotation:** Every 30 days, the instance generates a new keypair and announces the updated fingerprint to connected trusted peers via a `KeyRotation` message. Peers have a 7-day grace period to accept the new key.
- **Key backup:** Not supported. If a device is lost, the user pairs the new device fresh. This is intentional -- there is no recovery mechanism that could be exploited.

### Key Rotation Sequence

```
  Instance A (rotating)                    Instance B (trusted peer)
  ---------------------                    -----------------------
      |                                          |
      |  [Generate new Ed25519 keypair]          |
      |  [Store new key, keep old key active]    |
      |                                          |
      |--- KeyRotation {                         |
      |      old_fingerprint: "a1b2c3d4",        |
      |      new_fingerprint: "e5f6g7h8",        |
      |      new_public_key: "...",              |
      |      signed_with: old_key                |
      |    } ---------------------------------->|
      |                                          |
      |  [B verifies signature with old key]     |
      |  [B updates stored public_key for A]     |
      |                                          |
      |<-- KeyRotationAck -----------------------|
      |                                          |
      |  [After all peers ACK or 7 days:         |
      |   retire old key]                        |
      |                                          |
```

### Audit Logging

All collaboration events are logged to the existing `audit_log` table with the following action types:

| Action | Details |
|---|---|
| `peer.discovered` | `{ peer_id, display_name, hostname }` |
| `peer.trust_initiated` | `{ peer_id, method: "qr" | "code" }` |
| `peer.trusted` | `{ peer_id, display_name }` |
| `peer.untrusted` | `{ peer_id, reason }` |
| `peer.connected` | `{ peer_id, tls_version, cipher }` |
| `peer.disconnected` | `{ peer_id, reason }` |
| `peer.key_rotated` | `{ peer_id, old_fingerprint, new_fingerprint }` |
| `collection.shared` | `{ collection_id, sync_id }` |
| `collection.unshared` | `{ collection_id, sync_id }` |
| `sync.started` | `{ peer_id, collection_sync_id, direction }` |
| `sync.completed` | `{ peer_id, docs_synced, bytes_transferred }` |
| `sync.failed` | `{ peer_id, error }` |
| `sync.conflict` | `{ document_sync_id, resolution }` |

---

## 12. Crate Dependencies

The following crates are required for implementation. They are listed here for evaluation and security audit purposes. None are added to `Cargo.toml` until implementation begins.

| Crate | Version | Purpose | License |
|---|---|---|---|
| `mdns-sd` | ^0.11 | mDNS service discovery and advertisement (cross-platform) | MIT/Apache-2.0 |
| `rustls` | ^0.23 | TLS 1.3 implementation (no OpenSSL dependency) | MIT/Apache-2.0 |
| `rustls-pemfile` | ^2 | PEM file parsing for certificates | MIT/Apache-2.0 |
| `rcgen` | ^0.13 | Self-signed certificate generation (Ed25519) | MIT/Apache-2.0 |
| `ring` | ^0.17 | Cryptographic primitives (Ed25519, SHA-256, random) | ISC-style |
| `quinn` | ^0.11 | QUIC transport (future, alternative to raw TCP) | MIT/Apache-2.0 |
| `keyring` | ^3 | OS keychain access for key storage | MIT/Apache-2.0 |
| `qrcode` | ^0.14 | QR code generation for pairing | MIT/Apache-2.0 |
| `image` | ^0.25 | QR code rendering to image | MIT/Apache-2.0 |

### Dependency Rationale

- **`rustls` over `native-tls`:** Pure Rust, no system OpenSSL dependency, auditable, and consistent behavior across macOS/Windows/Linux.
- **`ring` over `openssl`:** Same rationale. Minimal, audited crypto library.
- **`mdns-sd` over `zeroconf`:** More actively maintained, better cross-platform support, async-friendly API.
- **`rcgen`:** Eliminates the need for external tooling (like `openssl` CLI) to generate self-signed certificates.

---

## 13. Implementation Phases

This roadmap describes the future implementation sequence. Each phase is independently shippable and provides user value.

### Phase A: Peer Discovery + Trust Establishment

**Scope:**
- mDNS service registration and browsing.
- Peer table UI (list of discovered devices).
- QR code and 6-digit code pairing flow.
- Ed25519 key generation and OS keychain storage.
- `peers` table migration.

**User Value:** Users can see other VaultMind instances on their network and pair with them.

**Estimated Effort:** 2-3 weeks.

### Phase B: Collection Sharing UI

**Scope:**
- "Share" toggle on collection settings.
- `sync_id` generation for shared collections.
- Per-collection peer permission management.
- `collections.shared` and `collections.sync_id` migration.

**User Value:** Users can designate which collections they want to share with paired devices.

**Estimated Effort:** 1-2 weeks.

### Phase C: Metadata Sync (CRDTs)

**Scope:**
- CRDT operation log implementation (G-Set, LWW-Register, OR-Set).
- Hybrid Logical Clock implementation.
- Version vector tracking.
- Metadata sync protocol (collection names, document metadata).
- `crdt_operations` and `version_vectors` table migrations.

**User Value:** Shared collection metadata stays in sync across devices. Renaming a collection or adding a document on one device is reflected on the other.

**Estimated Effort:** 3-4 weeks.

### Phase D: Document Content Sync

**Scope:**
- Chunked file transfer over TLS.
- SHA-256 integrity verification.
- Receiver-side re-ingestion pipeline (parse, chunk, embed).
- `sync_log` table migration.
- Progress UI for large transfers.
- Conflict detection and resolution UI.

**User Value:** Full document sync -- add a PDF on one device, it appears (fully indexed and searchable) on the paired device.

**Estimated Effort:** 3-4 weeks.

### Phase E: Real-Time Sync

**Scope:**
- Change notification streaming (push-based, not poll-based).
- Incremental operation delivery.
- Connection health monitoring and auto-reconnection.
- Key rotation protocol.
- Bandwidth management (throttling for large syncs).

**User Value:** Near-instantaneous sync. Changes appear on paired devices within seconds.

**Estimated Effort:** 2-3 weeks.

### Total Estimated Effort: 11-16 weeks

---

## 14. Appendix: Wire Protocol Specification

### Message Framing

All messages are length-prefixed on the TLS stream:

```
+----------+----------+---------+
| Length    | Type     | Payload |
| (4 bytes) | (1 byte) | (var)   |
| u32 BE   |          | msgpack |
+----------+----------+---------+
```

- **Length:** Total byte length of Type + Payload (big-endian u32).
- **Type:** Message type identifier (see table below).
- **Payload:** MessagePack-encoded message body.

### Message Types

| Type ID | Name | Direction | Description |
|---|---|---|---|
| `0x01` | `Handshake` | Both | Protocol version and capabilities exchange |
| `0x02` | `HandshakeAck` | Both | Handshake confirmation |
| `0x10` | `PairingRequest` | Joiner -> Initiator | Contains pairing code and identity |
| `0x11` | `PairingAccepted` | Initiator -> Joiner | Trust confirmed |
| `0x12` | `PairingRejected` | Initiator -> Joiner | Trust denied |
| `0x20` | `SyncInit` | Both | Start sync with collection list and version vectors |
| `0x21` | `SyncManifest` | Both | Delta manifest (what needs syncing) |
| `0x22` | `IncrementalOp` | Both | Single CRDT operation |
| `0x23` | `IncrementalAck` | Both | Acknowledge receipt of CRDT op |
| `0x30` | `ContentRequest` | Receiver | Request specific document files |
| `0x31` | `ContentChunk` | Sender | 64KB chunk of document content |
| `0x32` | `ContentComplete` | Sender | Transfer complete with SHA-256 hash |
| `0x33` | `ContentAck` | Receiver | Integrity verified, ingestion started |
| `0x40` | `KeyRotation` | Rotator | Announce new public key, signed with old key |
| `0x41` | `KeyRotationAck` | Peer | Acknowledge key update |
| `0xF0` | `Ping` | Both | Keep-alive |
| `0xF1` | `Pong` | Both | Keep-alive response |
| `0xFF` | `Disconnect` | Both | Graceful shutdown |

### Handshake Payload

```json
{
    "protocol_version": 1,
    "peer_id": "uuid-string",
    "display_name": "Dan's MacBook",
    "capabilities": ["metadata_sync", "content_sync"],
    "shared_collections_count": 3
}
```

### Error Handling

If a peer sends an unrecognized message type or a malformed payload:
1. The receiver sends a `Disconnect` message with error details.
2. The connection is closed.
3. The event is logged to `audit_log`.
4. Reconnection follows the standard backoff schedule.

---

## Revision History

| Date | Version | Change |
|---|---|---|
| 2026-02-08 | 1.0 | Initial design document |
