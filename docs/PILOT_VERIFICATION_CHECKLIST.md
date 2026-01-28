# Pre-Pilot Verification Checklist

Complete all checks before deploying v0.6.0 to the team.

## Build & Tests

- [ ] `cargo build --release` completes without errors
- [ ] `cargo test` returns 426+ passed, 0 failed
- [ ] App starts with `./target/release/assistsupport` or `pnpm tauri dev`
- [ ] No compilation warnings affecting functionality

## Database

- [ ] Existing KB data loads correctly
- [ ] 27 articles can be ingested via `ingest_kb_from_disk()`
- [ ] Source tracking (file, namespace, run) entries created
- [ ] FTS5 search returns results for known terms

## Core Features

- [ ] Flash drive policy correctly detected and returned
- [ ] Policy denial includes explanation and alternatives
- [ ] Procedure queries return PROCEDURES/ articles first
- [ ] No exceptions given for "emergency" or "just this once" requests
- [ ] Response latency < 2 seconds for typical queries

## UI/UX

- [ ] Search input responsive to typing
- [ ] Results display clearly with source citations
- [ ] No broken links or formatting issues
- [ ] Error messages are helpful (not raw stack traces)

## Documentation

- [ ] TEAM_PILOT_VALIDATION.md prepared (20 queries)
- [ ] PILOT_QUICK_START.md ready for distribution
- [ ] Known issues documented
- [ ] CHANGELOG.md up to date (v0.6.0 entry)

## Deployment Readiness

- [ ] Distribution method decided (GitHub release / Slack / manual build)
- [ ] Team members identified (minimum 5 participants)
- [ ] Feedback channel set up (Slack #assistsupport or similar)
- [ ] Timeline communicated to participants
- [ ] Feedback deadline set

## Go/No-Go

**Status**: [ ] GO (Deploy) / [ ] NO-GO (Fix issues first)

**Date**: _______________

**Verified by**: _______________

**Notes**: _______________
