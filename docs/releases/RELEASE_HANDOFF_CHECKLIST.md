# Release Handoff Checklist

## Build and Artifacts

- [ ] Build ID and commit SHA recorded.
- [ ] Artifact paths recorded.
- [ ] Checksums recorded for release artifacts.

## Security and Compliance

- [ ] Dependency advisories reviewed and linked.
- [ ] Security audit commands and outputs attached.
- [ ] Secrets handling validated (no credentials committed).

## Runtime Operations

- [ ] Search API production env variables documented.
- [ ] Redis backend reachability confirmed.
- [ ] Rollback instructions validated.

## Ownership and Escalation

- [ ] On-call owner assigned for first 7 days after release.
- [ ] Dependency alert owner assigned.
- [ ] Incident escalation channel documented.
