# Using AssistSupport as an IT Support Engineer

This guide shows how to use AssistSupport in real IT support workflows.

---

## Common Workflows

### Workflow 1: VPN/Network Connectivity Issue

**Ticket Received:**
```
User: "Can't connect to VPN from home on Windows 11"
```

**Using AssistSupport:**
1. Copy ticket text into AssistSupport search
2. Search: "VPN Windows 11 connectivity"
3. AssistSupport finds:
   - Your VPN troubleshooting guide
   - Windows 11 network config docs
   - Recent ticket history on VPN issues
4. AI generates draft:
   ```
   "1. Verify Cisco AnyConnect version (latest is 5.1.x)
   2. Check Windows 11 networking settings
   3. Run: ipconfig /all (send output)
   4. If still failing: Escalate to Network team"
   ```
5. You edit: Add your ticket reference, adjust wording
6. Copy to Jira: done

---

### Workflow 2: Software Installation Request

**Ticket Received:**
```
User: "Need Excel license for my new computer"
```

**Using AssistSupport:**
1. Search: "Excel license software installation"
2. AssistSupport finds:
   - License allocation policy
   - Installation steps
   - Approval process
   - Cost center info
3. AI generates:
   ```
   "Excel licenses are managed through IT Procurement.
   1. Submit request here: [link]
   2. Cost: $70/license
   3. Approval: 1-2 business days
   4. After approval: I'll install remotely"
   ```
4. You personalize: Add timeline, reference installation docs
5. Reply to ticket

---

### Workflow 3: Password Reset

**Ticket Received:**
```
User: "Forgot my email password"
```

**Using AssistSupport:**
1. Search: "password reset email"
2. AssistSupport finds:
   - Password reset procedure
   - MFA recovery steps
   - Account unlock process
3. AI generates:
   ```
   "1. Go to [company portal]
   2. Click 'Forgot Password'
   3. Verify your identity (SMS code)
   4. Set new password (12+ chars, special char required)
   5. If MFA enabled, use backup codes in your vault"
   ```
4. Verify it matches your policy
5. Send response

---

### Workflow 4: Printer Setup

**Ticket Received:**
```
User: "Can't print to the office printer from my laptop"
```

**Using AssistSupport:**
1. Search: "printer setup drivers Mac Windows"
2. AssistSupport finds:
   - Printer driver docs
   - Network printer setup guide
   - Firewall/network requirements
3. AI generates troubleshooting steps
4. Personalize with specific model number
5. Send

---

### Workflow 5: Email Client Configuration

**Ticket Received:**
```
User: "Outlook isn't syncing on my new MacBook Pro"
```

**Using AssistSupport:**
1. Search: "Outlook Mac configuration IMAP sync"
2. Find: Your Outlook setup guide, IMAP settings, common issues
3. AI generates:
   ```
   "1. Ensure you're on latest Outlook (v16.60+)
   2. Add account: Mail > Settings > Internet Accounts
   3. IMAP server: mail.company.com (port 993)
   4. If 2FA enabled: Use app-specific password
   5. Check: System Preferences > Internet Accounts"
   ```
4. Send

---

## Setup for Your Team

### Option 1: Individual Setup (Each Engineer)

**Best for:** Small teams (1-5 engineers), quick start

```bash
# Each engineer does this:
git clone https://github.com/saagar210/AssistSupport.git
cd AssistSupport
pnpm install
pnpm tauri dev

# Point KB folder to shared drive:
# Settings > Knowledge Base > Select Folder
# Choose: \\shared-drive\IT_KB or /mnt/shared/IT_KB
```

**Advantages:**
- No infrastructure required
- Each engineer can customize
- Works immediately

**Disadvantages:**
- Each person has a local KB copy
- No sync between engineers
- Manual KB updates

---

### Option 2: Team Shared KB (Recommended)

**Best for:** Medium teams (5-50 engineers), shared responsibility

**Setup:**
1. Create shared drive location: `\\shared-drive\IT_KnowledgeBase` (or `/mnt/shared/kb`)
2. Populate with team docs using this structure:
   ```
   IT_KnowledgeBase/
   ├── Windows/
   │   ├── connectivity/
   │   ├── accounts/
   │   └── software/
   ├── Mac/
   │   ├── connectivity/
   │   └── setup/
   ├── Network/
   │   ├── VPN/
   │   ├── Printing/
   │   └── Email/
   ├── Accounts/
   │   ├── password-resets.md
   │   └── mfa-setup.md
   └── Procedures/
       ├── onboarding.md
       └── offboarding.md
   ```
3. Each engineer points AssistSupport to the shared folder
4. Designate a KB owner to maintain and update docs

**Advantages:**
- Single source of truth
- All engineers have current docs
- Search across entire KB
- Consistency in responses

**Disadvantages:**
- Requires shared drive setup
- KB owner must maintain docs

---

### Option 3: Enterprise Deployment

**Best for:** Large teams (50+ engineers), strict compliance

**Setup:**
1. Use Azure Sync, OneDrive, or similar to sync shared KB to each machine
2. Configure read-only mode to prevent accidental edits
3. Enable audit logging for compliance
4. Document all procedures
5. Create escalation process for KB updates

---

## Integration with Your Ticketing System

### Jira Integration (Built-in)

**Setup:**
1. Settings > Integrations > Jira
2. Enter:
   - Jira Instance URL (e.g., `https://company.atlassian.net`)
   - Your email
   - API token (from Jira account settings)
3. Click "Test Connection"

**Usage:**
1. Open AssistSupport
2. Paste ticket key (e.g., `ITS-1234`)
3. Click "Load Ticket"
4. AssistSupport:
   - Fetches ticket title and description
   - Shows in response generation
   - Auto-adds reference in draft
5. Generate response with full context

**Example:**
```
Ticket: ITS-5678 - "Can't login to Exchange"
AssistSupport loads context:
  - Reporter: Jane Smith
  - Description: "Getting 401 error on Outlook"
  - Status: Open
Response generation includes: "Hi Jane, regarding your login issue..."
```

---

### Manual Integration (All Systems)

**For ServiceNow, Zendesk, Freshdesk, or any system:**

1. Copy ticket text
2. Paste into AssistSupport search
3. Generate response
4. Copy response
5. Paste into ticket reply
6. Send

---

## Privacy & Compliance

### Your Data Stays Local

**All processing happens on your machine:**
- KB indexed locally (not sent to any cloud)
- Responses generated locally (not sent to any cloud)
- No telemetry
- No analytics
- No data sharing

**Encryption:**
- Database encrypted (SQLCipher AES-256)
- Credentials encrypted (AES-256-GCM)
- Master key in macOS Keychain or passphrase-protected

**Audit Trail:**
- All actions logged (searches, generations, key changes)
- Logs stored locally
- Available for compliance review

---

### Compliance Status

**Current (v0.3.1):**
- **HIPAA**: Technical safeguards implemented
- **GDPR**: Privacy by design, local processing
- **FISMA**: NIST SP 800-53 controls implemented
- **SOC2**: Security controls documented
- **ISO 27001**: Encryption and access control
- **PCI DSS**: Application-level security

See the [Compliance Report](compliance/COMPLIANCE_REPORT.md) for full details.

---

## Troubleshooting

### KB Search Returns No Results
- Check KB folder is set (Settings > Knowledge Base)
- Verify documents exist in folder
- Try broader search terms
- Check document format (markdown, PDF, DOCX supported)

### Response Generation Is Slow
- Close other apps (frees RAM for LLM)
- Use smaller model (Llama 3.2 1B faster than 3B)
- Reduce context window size (Settings > Model)

### Can't Connect to Jira
- Verify Jira URL is correct (including `https://`)
- Check API token is valid
- Ensure firewall allows connection
- Try manual copy/paste workflow in the meantime

### Offline Mode Issues
- Verify KB was indexed while online
- Check KB folder is accessible
- Try restarting the app

---

## Tips & Tricks

### Create Response Templates
Store common escalations in your KB:
```markdown
# Escalation Scripts

## Network Team Escalation
"This appears to be a network issue. I'm escalating to the Network team.
Expected response: 24-48 hours. I'll follow up with you."

## Hardware Replacement
"Your device needs repair. I've created RMA ticket [#].
Estimated replacement: 5-7 business days."
```

Then search for these when needed.

---

### Build a Searchable FAQ
Keep your most common issues organized:
```
IT_KnowledgeBase/
├── FAQ/
│   ├── TOP_20_TICKETS.md     (most common issues)
│   ├── QUICK_ANSWERS.md      (1-2 sentence replies)
│   └── RECENT_ISSUES.md      (this month's problems)
```

---

### Track Response Time Improvements
- Week 1: Measure current average response time
- Start using AssistSupport
- Week 4: Measure new average response time

---

## Next Steps

1. **Read:** [INSTALLATION.md](INSTALLATION.md) for detailed setup
2. **Test:** Run app and try a search with your real KB
3. **Iterate:** Improve KB based on gaps you discover
4. **Deploy:** Roll out to team following Option 1, 2, or 3 above

---

## Questions?

See the [README](../README.md) for an overview, [SECURITY.md](SECURITY.md) for compliance details, or create an issue on GitHub.
