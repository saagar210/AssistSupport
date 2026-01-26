# AssistSupport Testing Strategy

## Automated Tests

### Backend (cargo test)
- Run full backend test suite
- Verify all tests pass
- Note any failures for investigation

### Frontend (pnpm test)
- Run full frontend test suite
- Verify all tests pass
- Note any failures for investigation

## Manual Tests (require app running)

### Test 1: App Launch & Initial Setup
- Run `pnpm tauri dev`
- Verify app launches without errors
- Test onboarding wizard
- Select default LLM (Llama 3.2 1B)

### Test 2: Knowledge Base Indexing
- Create test KB directory with sample docs
- Point KB folder to test directory
- Click "Re-index"
- Verify document count increases

### Test 3: Hybrid Search
- Search for "VPN": Should find relevant docs
- Search for "password reset": Should find relevant docs
- Verify results show relevance scores

### Test 4: Response Generation
- Enter query: "User can't connect to VPN"
- Verify KB search populates relevant docs
- Click Generate
- Check response streams in real-time
- Verify response is saveable as draft

### Test 5: Jira Integration
- Go to Settings > Integrations
- Test Jira connection
- If available: Load a test ticket
- If unavailable: Document that Jira requires credentials

### Test 6: Encryption & Key Storage
- Test Keychain mode (macOS default)
- Test Passphrase mode
- Verify wrong passphrase rejected
- Check audit log for key events

### Test 7: Offline Functionality
- Verify KB search still works offline
- Verify response generation still works offline
- Verify error messages are clear for web features

### Test 8: Path Security
- Attempt to index .ssh directory (should be blocked)
- Attempt to index .gnupg directory (should be blocked)
- Attempt to index outside $HOME (should be blocked)

### Test 9: Audit Logging
- Check audit.log exists in app data directory
- Generate a few actions
- Verify no secrets are logged

### Test 10: Health Diagnostics
- Go to Settings > Diagnostics
- Check component status

## Success Criteria
- All automated tests pass
- App launches without errors
- Core workflows functional
- Security controls verified
