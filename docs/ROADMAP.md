# AssistSupport Roadmap

**Current Version**: v0.5.0
**Target Audience**: IT Support Engineers (macOS first, Windows later)
**Status**: Production Ready, Actively Developed

---

## Release History

### v0.5.0 (Current — January 2026)
**Goal**: Polish the user experience with a professional, modern UI

- [x] ChatGPT-inspired UI redesign (dark-first, green accent)
- [x] System font stack for instant rendering
- [x] Smooth animations with reduced-motion support
- [x] Polished sidebar, lift-on-hover buttons, glow effects
- [x] WCAG AA accessibility maintained

### v0.4.x (January 2026)
**Goal**: Build productivity features and analytics

- [x] Analytics dashboard with response ratings and article drill-down
- [x] Response alternatives (side-by-side comparison)
- [x] Template recycling (save top responses for reuse)
- [x] Jira post + transition workflow
- [x] KB health and staleness monitoring
- [x] Fast startup with background model loading (2-3 seconds)
- [x] Session tokens (24h auto-unlock)
- [x] CLI with real search and indexing
- [x] Batch processing, auto-suggest, draft versioning

### v0.3.x (January 2026)
**Goal**: Security hardening, compliance, IT-focused positioning

- [x] Security audit and hardening (AES-256, SSRF protection, path validation)
- [x] Compliance validation (HIPAA, GDPR, FISMA, SOC2, ISO 27001, PCI DSS, NIST)
- [x] IT Support Guide and Quick Start documentation
- [x] Jira integration (fetch tickets, inject responses)
- [x] Comprehensive test suite

### v0.3.0 (January 2026)
**Goal**: Core functionality

- [x] LLM-driven response generation with streaming
- [x] Hybrid search (FTS5 + vector)
- [x] Multi-format document indexing (Markdown, PDF, DOCX, XLSX, code, images)
- [x] Web, YouTube, and GitHub content ingestion
- [x] SQLCipher encryption with dual key storage
- [x] Audit logging

---

## Upcoming

### v0.6.0 (Q2 2026)
**Goal**: Draft management and KB authoring

- [ ] Draft management improvements (save, resume, full history)
- [ ] KB management UI (create and edit articles in-app)
- [ ] Advanced analytics (ROI metrics, team benchmarking)
- [ ] Enhanced Jira integration (auto-categorization, custom templates per ticket type)
- [ ] Export responses (PDF, Word)

### v0.7.0 (Q3 2026)
**Goal**: Cross-platform and enterprise features

- [ ] Windows support (MSI installer, Windows Credential Manager)
- [ ] ServiceNow integration
- [ ] Advanced search (semantic filters, saved searches, custom facets)
- [ ] Team deployment guide v2 (MDM, shared KB, policy docs)
- [ ] Performance optimization for large KBs (10,000+ docs)

### v1.0 (Q4 2026)
**Goal**: Stable enterprise release

- [ ] Enterprise security audit
- [ ] Linux support (MVP)
- [ ] Deployment automation (MDM, silent installer, auto-updates)
- [ ] Advanced compliance (SOC2 audit-ready, HIPAA BAA-ready)
- [ ] Comprehensive documentation and team onboarding guides

---

## Beyond v1.0 (2027+)

- Custom LLM models per team
- API for third-party integrations
- Plugin system
- Multi-language support
- Mobile companion (iPad)
- Enterprise support plans

---

## Not Planned (And Why)

### Cloud Sync
Contradicts the privacy-first value proposition. Alternative: team shared KB folder (already supported).

### Multi-tenant SaaS
Local-first architecture is the differentiator. Alternative: on-premise deployment.

### Mobile App
IT support engineers work from desks. Lower priority than Windows/Linux support.

---

## How to Contribute

See the [Contributing section in README](../README.md#contributing) for details.

### Areas Needing Help
- Windows testing and porting
- Documentation and case studies
- Performance benchmarking
- Security testing

Open an issue on GitHub with the `roadmap` label to discuss feature priorities.
