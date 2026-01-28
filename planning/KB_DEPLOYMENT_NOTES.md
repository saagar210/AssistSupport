# KB Deployment Notes

## Source

- **Origin**: Confluence HTML space export (Box, Inc. corporate IT)
- **Export size**: 664 HTML pages, 2.0 GB (with attachments)
- **Pages audited**: 663
- **Pages kept**: 481 (skipped Windows-only, duplicates, project pages)
- **Date converted**: January 27, 2026

## Conversion Process

1. User exported Confluence space as HTML to `~/Downloads/IT/`
2. Audited 663 pages, identified 481 to keep
3. Converted HTML to clean Markdown via 9 parallel agents:
   - Stripped Confluence chrome (navigation, sidebar, breadcrumbs, metadata panels)
   - Extracted titles from `<h1>` or `<title>` tags
   - Converted body content to clean Markdown (tables, code blocks, lists, links)
   - Applied kebab-case filenames
4. Validated all 484 converted files:
   - Fixed 1 unclosed code block (`eus-automations-tools.md`)
   - Fixed HTML entities in 1 file (`ticket-etiquette-closure.md`)
   - Removed 16 empty stub files (<50 bytes)
   - Resolved 3 cross-category duplicate filenames
5. Mapped 87 cross-reference relationships across categories
6. Added "Related Articles" sections to 15 hub articles (64 links total)
7. Created KB documentation (README, MAINTENANCE, DEPLOYMENT)

## Final KB Location

```
~/Documents/IT-KnowledgeBase/
```

## Article Counts

| Category | Articles |
|----------|----------|
| Accounts-Access | 7 |
| Admin-Runbooks | 230 |
| Applications | 33 |
| Communication-Collab | 32 |
| Disaster-Recovery | 18 |
| Email | 13 |
| Escalation | 9 |
| Hardware-Peripherals | 13 |
| macOS | 16 |
| Network-VPN | 10 |
| Onboarding-Offboarding | 12 |
| Provisioning | 38 |
| Security-Auth | 35 |
| **Total** | **466** |

## Statistics

- **Total words**: ~462,000
- **Total size**: 2.98 MB
- **Average article length**: ~955 words
- **Largest category**: Admin-Runbooks (230 articles, ~1.6 MB)
- **Smallest category**: Accounts-Access (7 articles, ~79 KB)

### Top Tools Referenced

| Tool | Article mentions |
|------|-----------------|
| Box | 352 |
| Okta | 189 |
| Confluence | 187 |
| Slack | 129 |
| Jira | 115 |
| Zoom | 88 |
| Duo | 86 |
| Kandji | 48 |

## AssistSupport Configuration

1. Open AssistSupport > **Settings** (Cmd+,)
2. Navigate to **Knowledge Base**
3. Set KB path to: `~/Documents/IT-KnowledgeBase/`
4. Click **Index**

### Expected Indexing Performance

| Metric | Expected |
|--------|----------|
| Files to index | 466 .md files |
| Content size | 2.98 MB |
| First-run indexing | 5-10 minutes |
| FTS5 index size | ~5-10 MB |
| LanceDB vector index size | ~50-100 MB |
| Re-index (subsequent) | 1-2 minutes |

### Verification Queries

After indexing, test these searches:

| Query | Expected Category |
|-------|------------------|
| "VPN connection error" | Network-VPN |
| "password reset" | Security-Auth |
| "new hire onboarding" | Onboarding-Offboarding |
| "Okta admin" | Admin-Runbooks |
| "escalate ticket" | Escalation |
| "YubiKey setup" | Security-Auth |
| "laptop provisioning" | Provisioning |
| "Slack integration" | Communication-Collab |
| "disaster recovery" | Disaster-Recovery |
| "Kandji" | macOS |

## KB Documentation

The KB includes three documentation files:

- `README.md` - Structure guide, key articles, writing guidelines, statistics
- `MAINTENANCE.md` - Weekly/monthly/quarterly maintenance checklists
- `DEPLOYMENT.md` - AssistSupport setup steps, verification, troubleshooting

## Maintenance

When updating the KB:
1. Add/edit/remove markdown files in the appropriate category folder
2. Re-index in AssistSupport (Settings > Knowledge Base > Re-index)
3. Verify search results reflect changes

See `~/Documents/IT-KnowledgeBase/MAINTENANCE.md` for the full maintenance schedule.
