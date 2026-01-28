# Week 3 Deploy Checklist

## Pre-Deployment

- [ ] Build fresh release binary: `cargo build --release`
- [ ] Run test suite: `cargo test` (expect 426+ passed)
- [ ] Manual smoke test: Run app, test "Can I get a flash drive?"
- [ ] Verify response: Should deny clearly with policy + alternative
- [ ] Check no warnings/errors in build log

## Deployment Package

- [ ] Binary location: `target/release/assistsupport`
- [ ] Create ZIP: `assistsupport-v0.6.0.zip`
- [ ] Include: Quick start guide link (PILOT_QUICK_START.md)
- [ ] Include: 20 validation queries link (TEAM_PILOT_VALIDATION.md)

## Team Communication

- [ ] Message ready: Send to #assistsupport
- [ ] Participant list: 5-10 team members identified
- [ ] Feedback channel: Slack thread or in-app feedback form ready
- [ ] Deadline: Friday 5 PM for feedback

## Go/No-Go

- [ ] Build successful?
- [ ] All tests passing?
- [ ] Smoke test passes?

**Status**: ________

**Deployed by**: ________

**Date/Time**: ________
