# Quick Start for IT Support Engineers

**Goal**: Generate your first ticket response in 5 minutes.

## 1. Download & Install

```bash
# Clone repo
git clone https://github.com/saagar210/AssistSupport.git
cd AssistSupport

# Install system dependencies (macOS)
brew install protobuf pkgconf cmake leptonica tesseract

# Install frontend dependencies
pnpm install

# Launch in dev mode
pnpm tauri dev
```

The app opens with an onboarding wizard.

## 2. Choose Key Storage

**Option A: Keychain Mode** (Recommended)
- Master key stored in macOS Keychain
- Automatically unlocked when logged in
- Click "Use Keychain" in wizard

**Option B: Passphrase Mode**
- Master key protected by your passphrase
- Portable to other machines
- Click "Use Passphrase" in wizard, enter password

## 3. Select KB Folder

Choose a folder with your documentation:
- Local folder on your machine
- OR shared network drive (e.g., `\\server\IT-Support-KB`)
- OR create an empty folder and add docs later

Click "Browse", select folder, click "Re-index".

Supported formats: Markdown, PDF, DOCX, XLSX, code files.

## 4. Generate Your First Response

1. Go to the "Draft" tab
2. Paste a support question:
   ```
   "User can't login to VPN after password change on Windows 11"
   ```
3. Click "Generate Response"
4. AI searches your KB and drafts a response
5. Review and edit as needed
6. Copy to your ticket system (Cmd+Shift+C)

## Next Steps

- **Add Jira Integration**: Settings > Integrations > Jira ([details](IT_SUPPORT_GUIDE.md#jira-integration-built-in))
- **Populate KB**: Add your IT docs to the folder you selected
- **Invite Team**: Have each engineer clone the repo and point to a shared KB folder ([details](IT_SUPPORT_GUIDE.md#setup-for-your-team))
- **Read Full Guide**: [IT Support Guide](IT_SUPPORT_GUIDE.md) for workflows, team setup, and compliance info

## Compliance

Everything stays on your machine. No exceptions.

- HIPAA: Technical safeguards implemented
- GDPR: Privacy by design, local processing
- FISMA: NIST SP 800-53 controls implemented
- SOC 2: Security controls documented

See the [Compliance Report](compliance/COMPLIANCE_REPORT.md) for full details.
