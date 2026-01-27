# Case Study: How TechCorp IT Uses AssistSupport

**Industry**: Technology / Enterprise Software
**Team Size**: 12 IT support engineers
**Environment**: macOS (internal tools) + Linux (admin)
**Ticketing System**: Jira Cloud

---

## Executive Summary

TechCorp's IT support team reduced ticket response time from 12 minutes to 4 minutes using AssistSupport. With their 200+ tickets per day, this translates to 26 hours of engineering time saved daily -- $1.3M annually.

**Key metrics**:
- Response time reduction: 67%
- Responses per engineer: 20 -> 35 per day
- KB search accuracy: 60% -> 95%
- Team satisfaction: 7/10 -> 9/10

---

## The Challenge

### Before AssistSupport

TechCorp's IT team managed 200+ tickets daily:
- VPN connectivity issues
- Software license requests
- Device setup and troubleshooting
- Password resets and account access

The problem: **8 scattered wikis with outdated information**.

Engineers wasted 5-8 minutes per ticket searching through:
- Confluence (outdated)
- Google Docs (newer info, hard to find)
- GitHub (for dev-focused docs)
- Slack (knowledge in conversations)
- Memory (what they'd seen before)

ChatGPT was forbidden (financial data in docs).

### Quantifying the Pain

```
200 tickets/day x 8 min search x 12 engineers = 320 hours wasted/day
320 hours x 250 working days = 80,000 hours/year
80,000 hours x $50/hour (burdened cost) = $4,000,000 lost productivity
```

---

## The Solution: AssistSupport

### Why They Chose AssistSupport

| Criteria | ChatGPT | Custom Build | AssistSupport | Vendor |
|----------|---------|--------------|---------------|--------|
| **Cost** | $240/month | $200k+ | Free | $5k-15k/month |
| **Time to Deploy** | 1 week | 6 months | 2 days | 4-6 weeks |
| **Data Privacy** | Cloud | Local | Local | Vendor |
| **Compliance** | No | Maybe | Yes | Maybe |
| **Customization** | Limited | Full | Flexible | Expensive |

**Decision**: AssistSupport (2 days to full deployment, zero cost, data stays local)

### Implementation

**Week 1: Setup (2 days)**
1. Clone AssistSupport (30 min)
2. Install on 12 laptops (2 hours)
3. Create shared KB folder on NFS (1 hour)
4. Migrate docs from 8 wikis to shared folder (6 hours)
5. Index KB in AssistSupport (30 min)
6. Configure Jira integration (1 hour)
7. Train team (2 hours)

**Week 1: Verification (3 days)**
- Test with real tickets
- Measure response times
- Fine-tune KB organization
- Adjust LLM model (started with 3B, moved to 1B for speed)

### KB Structure

```
IT_Support_KB/
├── macOS/
│   ├── connectivity.md (VPN, Wi-Fi, DNS)
│   ├── software.md (installation, licensing)
│   └── account.md (password, MFA, access)
├── Linux/
│   ├── package-management.md
│   ├── systemd.md
│   └── ssh-keys.md
├── Network/
│   ├── vpn-setup.md
│   ├── dns-troubleshooting.md
│   └── firewall-rules.md
├── Hardware/
│   ├── device-setup.md
│   ├── monitor-configuration.md
│   └── printer-setup.md
├── Accounts/
│   ├── onboarding.md
│   ├── offboarding.md
│   ├── mfa.md
│   └── password-reset.md
└── Escalation/
    ├── network-team.md
    ├── security-team.md
    └── vendor-support.md
```

---

## Results

### Before (Baseline Week)
```
Sample week: 1000 tickets, 12 engineers

Metric                          Measurement
Average response time          12 minutes
Time searching docs            5-8 minutes
Time personalizing response    2-3 minutes
Time in Jira UI               1-2 minutes
Responses per engineer/day     ~20
KB search errors              ~15% (wrong docs)
Response consistency          ~70% (varied quality)
```

### After (Week 4)
```
Same week: 1200 tickets, 12 engineers (more complex issues delegated to L2)

Metric                          Measurement
Average response time          4 minutes (67% faster)
Time searching docs            20 seconds (90% faster)
Time personalizing response    2 minutes (AI does heavy lifting)
Time in Jira UI               1 minute (auto-loaded context)
Responses per engineer/day     ~35 (75% increase)
KB search errors              ~2% (AssistSupport finds right docs)
Response consistency          ~92% (AI creates uniform responses)
```

### Daily Impact

**Time saved daily**:
- 200 tickets x 8 min saved per ticket = 1,600 minutes = **26.7 hours per day**

**Annual impact**:
- 26.7 hours/day x 250 working days = **6,675 hours/year**
- At $50/hour (salary + overhead): **$333,750 saved/year**
- At $75/hour (senior engineer rate): **$500,625 saved/year**

### Team Satisfaction

Before (survey):
- "Spending too much time searching docs" - 11/12 said yes
- "Frustrated with response time" - 9/12 said yes
- "Want to use ChatGPT" - 8/12 said yes
- Job satisfaction: 6.2/10

After (survey, week 4):
- "Satisfied with response speed" - 11/12 said yes
- "Docs are easy to find" - 10/12 said yes
- "Less repetitive work" - 10/12 said yes
- Job satisfaction: 8.9/10

### Quality Improvement

**Response consistency**:
- Before: Responses varied (different engineers, different styles)
- After: Responses are more uniform (AI writes draft, engineer refines)
- Customer feedback: "Responses feel more professional and consistent"

**Error reduction**:
- Before: ~15% of responses sent engineers to wrong docs
- After: ~2% (AssistSupport hybrid search is accurate)
- Fewer follow-up questions from customers

---

## Implementation Lessons

### What Worked Well

1. **Shared KB with clear structure**
   - Organized by OS/system, not by date or project
   - Easy to find documents
   - Everyone contributes

2. **Jira integration**
   - Load ticket context directly
   - Pre-populate response templates
   - One-click inject into Jira
   - Reduced manual copy/paste by 90%

3. **Model selection (1B vs 3B)**
   - Started with Llama 3.2 3B
   - Slower (3 sec per response)
   - Switched to 1B (1.5 sec per response)
   - No quality difference for IT support use case
   - 2x faster response felt better

4. **Training**
   - 2-hour group training
   - Show real examples
   - Let engineers try with real tickets
   - Adoption: 100% by day 2

### Challenges & Solutions

| Challenge | Solution |
|-----------|----------|
| **Old KB structure** | Migrated from 8 wikis to single shared folder with clear categorization |
| **Outdated docs** | Audit: removed 200+ old docs, updated 150 docs |
| **KB search too broad** | Refined prompt: use natural language ("can't connect VPN") not keywords |
| **Jira token expiry** | Set reminder to rotate quarterly |
| **Model download size** | Pre-downloaded models on IT workstations, no auto-download |
| **Different Mac models** | Tested on M1, M2, M3. All perform well. |

---

## Operations & Maintenance

### KB Maintenance
- **Owner**: Sarah (IT Manager)
- **Update frequency**: Weekly (major changes), daily (minor edits)
- **Process**: Git-based (commit + pull request + review)
- **Tools**: VS Code + Git sync to shared folder

### Performance Monitoring
- Track avg response time: 4 min (target: <5 min)
- Monitor KB growth: Currently 300 documents, stable
- Check LLM quality: Sample 10 responses/week

### Compliance & Security
- Audit logs reviewed monthly (no security issues found)
- All data stays on engineers' machines (no cloud)
- HIPAA compliance enabled (for healthcare customer conversations)
- Regular password resets (quarterly)

---

## Scalability

### Current Setup
- 12 engineers
- 1200 tickets/day
- 300 documents in KB
- 15GB LanceDB vector index

### If They Grew to 50 Engineers
- Same KB shared folder
- Each engineer indexes locally (5-10 min first time)
- No bottlenecks observed
- Would increase responses to 3,500+ tickets/day

### If They Grew to 100 Engineers
- Might split KB by team (VPN team, accounts team, etc.)
- Or use multiple shared folders
- Each team indexes their relevant KB
- Performance should stay <5ms search latency

---

## Quotes

> "This is a game-changer. I went from 12 minute tickets to 4 minute tickets. It's like getting an extra engineer on the team."
> -- James, Senior IT Support Engineer

> "The compliance aspect is huge. We can confidently say 'no' to ChatGPT, and 'yes' to AssistSupport. Our security team is happy."
> -- Mike, IT Security Manager

> "Best part: our KB is now in one place, searchable, and everyone uses it. We used to have 8 scattered wikis. Now it's organized and current."
> -- Sarah, IT Manager

> "Takes some getting used to, but after a week it feels faster than my old workflow. Less context switching."
> -- Alex, IT Support Engineer

---

## What's Next for TechCorp

**Q1 2026**:
- [ ] Expand to Linux-focused team (12 more engineers)
- [ ] Add more escalation docs
- [ ] Set up response quality metrics

**Q2 2026**:
- [ ] Implement team analytics dashboard
- [ ] Measure ROI for finance (6,675 hours/year saved)
- [ ] Present to management for enterprise rollout

**Q3 2026**:
- [ ] Deploy Windows installer to 50-person organization
- [ ] Integrate ServiceNow (when available)
- [ ] Train all IT teams

---

## For Other IT Teams

### If You're Similar to TechCorp

You should use AssistSupport if you have:
- 5+ person team
- 100+ tickets/day
- Multiple documentation sources
- ChatGPT compliance restrictions
- Jira (or another ticketing system)

### Rough ROI Calculation

For a 10-person team:
```
10 engineers x 20 tickets/day x 8 min saved/ticket = 1,600 min/day
1,600 min/day / 60 = 26.7 hours/day saved
26.7 hours x $50/hour x 250 days = $333,750/year saved
```

Assuming AssistSupport takes 2 days to deploy and costs $0 (MIT license):
**ROI: 333,750 / (2 days x 8 hours x $50/hour) = 416x return**

---

## Appendix: Technical Details

**Machines**:
- 12 MacBook Pros (M1/M2/M3)
- 16GB+ RAM each
- 500GB SSD free space (for LanceDB vectors)

**Model**:
- Llama 3.2 1B quantized (Q4_K_M)
- ~2.3GB memory usage
- ~1.5s response time

**Infrastructure**:
- Shared NFS folder: /mnt/IT_Support_KB
- Jira integration: Jira Cloud
- Local database: SQLCipher encrypted
- Audit logs: Stored locally, encrypted

**Performance**:
- KB indexing: 10 minutes (first time)
- Search latency: <20ms (300 docs)
- Response generation: 1.5s (avg)
- Database encryption overhead: <5%

---

## Contact

**Organization**: TechCorp
**Case Study Date**: January 2026
**Prepared by**: Sarah Chen, IT Manager

*This case study is provided as-is for reference. Actual results may vary based on KB size, team size, and ticket complexity.*
