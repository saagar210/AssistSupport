# Phase 3 Rust Advisory Map

Date: 2026-02-22  
Source commands:

- `pnpm run test:security:audit:rust`
- `cd src-tauri && cargo audit --json`
- `cd src-tauri && cargo tree --target all -i <crate>`

## Baseline and Delta

- Baseline waiver count entering Phase 3: **20** IDs (Week 1-2 script baseline).
- Active denied-warning advisories after Phase 3 remediation: **18** IDs.
- Net reduction: **2 IDs removed** (`RUSTSEC-2024-0414`, `RUSTSEC-2024-0417`) via Tauri feature pruning (`x11` removed from enabled features).
- Remaining IDs are tracked under umbrella issue: https://github.com/saagar210/AssistSupport/issues/11.

## Advisory Table

| RUSTSEC ID        | Package            | Dependency path (high-level)                     | Mitigation candidate                           | Owner                | Tracking issue                                              | Status             |
| ----------------- | ------------------ | ------------------------------------------------ | ---------------------------------------------- | -------------------- | ----------------------------------------------------------- | ------------------ |
| RUSTSEC-2024-0411 | gdkwayland-sys     | tauri -> tauri-runtime-wry/wry -> gtk/webkit2gtk | Upstream Tauri runtime upgrade                 | Platform Engineering | [#12](https://github.com/saagar210/AssistSupport/issues/12) | Blocked (upstream) |
| RUSTSEC-2024-0412 | gdk                | tauri runtime Linux GTK stack                    | Upstream Tauri runtime upgrade                 | Platform Engineering | [#12](https://github.com/saagar210/AssistSupport/issues/12) | Blocked (upstream) |
| RUSTSEC-2024-0413 | atk                | tauri runtime Linux GTK stack                    | Upstream Tauri runtime upgrade                 | Platform Engineering | [#12](https://github.com/saagar210/AssistSupport/issues/12) | Blocked (upstream) |
| RUSTSEC-2024-0415 | gtk                | tauri runtime Linux GTK stack                    | Upstream Tauri runtime upgrade                 | Platform Engineering | [#12](https://github.com/saagar210/AssistSupport/issues/12) | Blocked (upstream) |
| RUSTSEC-2024-0416 | atk-sys            | tauri runtime Linux GTK stack                    | Upstream Tauri runtime upgrade                 | Platform Engineering | [#12](https://github.com/saagar210/AssistSupport/issues/12) | Blocked (upstream) |
| RUSTSEC-2024-0418 | gdk-sys            | tauri runtime Linux GTK stack                    | Upstream Tauri runtime upgrade                 | Platform Engineering | [#12](https://github.com/saagar210/AssistSupport/issues/12) | Blocked (upstream) |
| RUSTSEC-2024-0419 | gtk3-macros        | tauri runtime Linux GTK stack                    | Upstream Tauri runtime upgrade                 | Platform Engineering | [#12](https://github.com/saagar210/AssistSupport/issues/12) | Blocked (upstream) |
| RUSTSEC-2024-0420 | gtk-sys            | tauri runtime Linux GTK stack                    | Upstream Tauri runtime upgrade                 | Platform Engineering | [#12](https://github.com/saagar210/AssistSupport/issues/12) | Blocked (upstream) |
| RUSTSEC-2024-0429 | glib               | tauri runtime Linux GTK stack                    | Upstream Tauri runtime upgrade                 | Platform Engineering | [#12](https://github.com/saagar210/AssistSupport/issues/12) | Blocked (upstream) |
| RUSTSEC-2024-0370 | proc-macro-error   | gtk/glib macro chain                             | Upstream GTK stack retirement                  | Platform Engineering | [#12](https://github.com/saagar210/AssistSupport/issues/12) | Blocked (upstream) |
| RUSTSEC-2025-0057 | fxhash             | tauri-utils -> kuchikiki/selectors               | tauri-utils upstream replacement               | Platform Engineering | [#13](https://github.com/saagar210/AssistSupport/issues/13) | Blocked (upstream) |
| RUSTSEC-2025-0075 | unic-char-range    | tauri-utils -> urlpattern -> unic\*              | tauri-utils/urlpattern upstream replacement    | Platform Engineering | [#13](https://github.com/saagar210/AssistSupport/issues/13) | Blocked (upstream) |
| RUSTSEC-2025-0080 | unic-common        | tauri-utils -> urlpattern -> unic\*              | tauri-utils/urlpattern upstream replacement    | Platform Engineering | [#13](https://github.com/saagar210/AssistSupport/issues/13) | Blocked (upstream) |
| RUSTSEC-2025-0081 | unic-char-property | tauri-utils -> urlpattern -> unic\*              | tauri-utils/urlpattern upstream replacement    | Platform Engineering | [#13](https://github.com/saagar210/AssistSupport/issues/13) | Blocked (upstream) |
| RUSTSEC-2025-0098 | unic-ucd-version   | tauri-utils -> urlpattern -> unic\*              | tauri-utils/urlpattern upstream replacement    | Platform Engineering | [#13](https://github.com/saagar210/AssistSupport/issues/13) | Blocked (upstream) |
| RUSTSEC-2025-0100 | unic-ucd-ident     | tauri-utils -> urlpattern -> unic\*              | tauri-utils/urlpattern upstream replacement    | Platform Engineering | [#13](https://github.com/saagar210/AssistSupport/issues/13) | Blocked (upstream) |
| RUSTSEC-2024-0436 | paste              | lancedb/lance/datafusion chain                   | Upstream lancedb/lance update                  | Platform Engineering | [#14](https://github.com/saagar210/AssistSupport/issues/14) | Blocked (upstream) |
| RUSTSEC-2026-0002 | lru                | tantivy -> lance-index/lance/lancedb             | Upstream tantivy/lance update or safe override | Platform Engineering | [#15](https://github.com/saagar210/AssistSupport/issues/15) | Blocked (upstream) |

## Phase 3 Actions Completed

1. Enabled feature-pruned Tauri config to remove X11-only dependency path.
2. Reduced advisory set by removing `RUSTSEC-2024-0414` and `RUSTSEC-2024-0417` from active graph.
3. Replaced `Unknown` waiver metadata with real issue-backed ownership in `scripts/security/run-cargo-audit.sh`.
4. Created and linked mitigation issue tree (#11–#15).
