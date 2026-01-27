# AssistSupport Roadmap

**Current Version**: v0.3.2
**Target Audience**: IT Support Engineers (macOS first, Windows later)
**Status**: Production Ready, Actively Developed

---

## Release Strategy

### v0.3.x (Current - Q1 2026: Foundational)
**Goal**: Establish Jira as the primary integration, prove ROI for IT teams

### v0.4.x (Q2 2026: Scaling)
**Goal**: Add team analytics, measure impact, expand deployment

### v0.5.x (Q3 2026: Enterprise)
**Goal**: Polish product, add advanced features, Windows support

### v1.0 (Q4 2026: Stable)
**Goal**: Enterprise-ready, fully documented, team deployment guides

---

## Q1 2026 (Jan-Mar): Jira Mastery

### Completed
- [x] Repositioning for IT support engineers
- [x] IT Support Guide with workflows
- [x] Quick Start guide (5-minute setup)
- [x] Compliance validation (HIPAA/GDPR/FISMA/SOC2)
- [x] Security audit and hardening
- [x] 436 passing tests (backend + frontend)

### In Progress
- [ ] **Jira Integration Enhancement**
  - Status: Planning
  - Goal: Seamless ticket context loading
  - Features:
    - Auto-load ticket title, description, assignee, status
    - Pre-populate response with ticket metadata
    - One-click inject response back to Jira
  - Impact: 70% faster ticket responses

- [ ] **Testing Infrastructure**
  - Status: Planning
  - Goal: Automated tests IT teams can run to verify setup
  - Components:
    - Integration test suite (KB indexing, search, generation)
    - Performance benchmarks (search latency, generation speed)
    - Security validation (encryption, key storage, audit logging)
    - Jira connectivity test
  - Impact: Confidence in production deployment

- [ ] **GitHub Enterprise Discovery**
  - Status: Planning
  - Goal: Make repo discoverable to IT teams
  - Components:
    - GitHub Topics (ai-support, offline-first, hipaa, gdpr)
    - README GitHub Discussions
    - Contribution guidelines
  - Impact: Organic reach to target market

### Upcoming (Q1)
- [ ] **Case Study Template**
  - Goal: Document how teams use AssistSupport
  - Include: Time savings, response quality, team feedback
  - Format: Markdown template in docs/CASE_STUDIES/

---

## Q2 2026 (Apr-Jun): Scale & Measure

### High Priority
- [ ] **Team Analytics Dashboard**
  - Goal: Track responses generated, time saved, KB usage
  - Metrics:
    - Responses generated per engineer
    - Average time per response (before/after)
    - KB search frequency
    - Top search queries
    - LLM generation time percentiles
  - Storage: Local SQLite table (encrypted, audit logged)
  - Impact: Prove ROI to management

- [ ] **Enhanced Jira Integration**
  - Custom response templates per ticket type
  - Auto-categorization (incident, request, question)
  - Response quality scoring (user feedback)

### Medium Priority
- [ ] **Windows Support (Planning Only)**
  - Goal: Plan Windows deployment
  - Components: Installer (.msi), path handling, credential storage (Windows Credential Manager)
  - Decision: In-app installer vs external .msi

- [ ] **Advanced Templating**
  - Custom variables beyond {{company_name}}
  - Template versioning
  - Team template library

### Low Priority
- [ ] **Performance Optimization**
  - Benchmark real KB sizes (100 to 10,000 docs)
  - LLM context optimization
  - Vector search speed tuning

---

## Q3 2026 (Jul-Sep): Enterprise & Expand

### High Priority
- [ ] **Windows Installer & Support**
  - Goal: Deploy to Windows IT teams
  - Components:
    - MSI installer
    - Windows Credential Manager integration
    - Windows 10/11 compatibility testing
    - Installer self-update mechanism
  - Impact: Expand market to 50%+ of IT teams

- [ ] **Team Deployment Guide v2**
  - Goal: Formalize team rollout process
  - Include: Shared KB setup, policy docs, compliance checklist
  - Target: Rollout to 20+ person teams

### Medium Priority
- [ ] **Linux Support (MVP)**
  - Goal: Basic Linux support for DevOps teams
  - Note: Lower priority than Windows

- [ ] **Advanced Search Features**
  - Semantic filters (search by topic, severity, etc.)
  - Custom facets
  - Search history and saved searches

### Deferred to v1.0+
- [ ] **Mobile app** (iOS/Android) - Future consideration
- [ ] **Cloud sync** - Contradicts privacy value prop, skip for now
- [ ] **Multi-user sync** - Plan for enterprise tier (later)

---

## Q4 2026 (Oct-Dec): Stabilize & Document

### High Priority
- [ ] **v1.0 Stable Release**
  - Polish all features
  - Comprehensive documentation
  - Enterprise security audit
  - Performance optimization

- [ ] **Deployment Automation**
  - Device Management (MDM) integration
  - Silent installer
  - Automatic updates

### Medium Priority
- [ ] **Advanced Compliance**
  - SOC2 audit
  - HIPAA Business Associate Agreement (BAA) ready
  - Export compliance controls for audits

---

## Beyond v1.0 (2027+)

### Future Directions
- Custom LLM models per team
- Advanced analytics (team benchmarking, security insights)
- API for third-party integrations
- Enterprise support plans

---

## Jira Integration Focus

### Why Jira?
- Industry standard for IT teams
- 80%+ of enterprise IT departments use Jira
- Strong API support
- Direct competitive advantage vs ChatGPT

### Jira Roadmap
1. **v0.3.x** (Current)
   - Load ticket by ID
   - Fetch title and description
   - Inject response manually
   - Auto-load all ticket fields
   - One-click response injection
   - Template variables ({{ticket_id}}, {{reporter}})
   - Connection pooling

2. **v0.4.0** (Q2 2026)
   - Smart ticket categorization
   - Response templates by type
   - Quality scoring (helpful/not helpful)
   - Bulk response generation (for similar tickets)

3. **v0.5.0+** (Q3+)
   - Custom fields mapping
   - Jira Cloud federation
   - Cross-Jira instance support

---

## Success Metrics

### Q1 Goals
- 50+ GitHub stars (targeting IT community)
- 5+ case studies (from beta testers)
- 90%+ test pass rate (automated tests)
- <100ms search latency (on realistic KB)
- <3s response generation (on macOS M-series)

### Q2 Goals
- 20+ IT teams actively using
- 50% average response time reduction
- 5+ enterprise pilots
- Windows beta available

### Q3+ Goals
- Windows parity with macOS
- 100+ IT teams deployed
- Annual users grow 10x
- Vendor partnership discussions

---

## Not Planned (And Why)

### ServiceNow Integration
- **Why deferred**: Jira covers 80% of target market better
- **Reason**: ServiceNow is heavier, more enterprise-focused
- **Timeline**: After Jira mastery (v0.5+)

### Cloud Sync
- **Why**: Contradicts privacy value prop
- **Alternative**: Team shared KB folder (already supported)

### Mobile App
- **Why**: IT support engineers work from desks
- **Alternative**: Responsive web (future, optional)

### Multi-tenant SaaS
- **Why**: Local-first architecture is the differentiator
- **Alternative**: On-premise deployment (v1.0+)

---

## Past Releases

### v0.3.2 (2026-01-27)
- Real download progress in OnboardingWizard
- Quick start guide and compliance badges
- Documentation repositioning for IT support engineers

### v0.3.1 (2026-01-25)
- Compliance validation (NIST, ISO 27001, SOC2, HIPAA, PCI DSS, GDPR, FISMA)
- Security hardening (path validation, SSRF protection, encryption)
- Comprehensive test suite (436 tests)
- Documentation (6 guides)

### v0.3.0 (2026-01-20)
- LLM-driven response generation
- Hybrid search (FTS + vector)
- Multi-format document indexing
- Jira integration
- Audit logging

---

## How to Contribute

See [CONTRIBUTING.md](../CONTRIBUTING.md) for details.

### Areas Needing Help
- Windows testing and porting
- Documentation and case studies
- Performance benchmarking
- Security testing

---

## Questions?

Open an issue on GitHub with the `roadmap` label, or start a discussion.
