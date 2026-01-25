# AssistSupport Installation Guide

## macOS Installation (Unsigned DMG)

AssistSupport is distributed as an unsigned DMG for internal use. macOS will show a security warning when you first open the app.

### Installation Steps

1. **Download** `AssistSupport_x.x.x_aarch64.dmg`

2. **Mount** the DMG by double-clicking it

3. **Drag** AssistSupport.app to the Applications folder

4. **First Launch** - one of these methods:

   **Option A: Right-click to open (easiest)**
   - Right-click (or Control-click) on AssistSupport.app
   - Select "Open" from the context menu
   - Click "Open" in the dialog

   **Option B: Remove quarantine via Terminal**
   ```bash
   xattr -d com.apple.quarantine /Applications/AssistSupport.app
   ```

   **Option C: Allow in System Settings**
   - Try to open the app normally (it will be blocked)
   - Go to System Settings > Privacy & Security
   - Scroll down to find the message about AssistSupport
   - Click "Open Anyway"

### System Requirements

- macOS 10.15 (Catalina) or later
- Apple Silicon (M1/M2/M3/M4) or Intel processor
- 200MB free disk space

### First Run

On first launch, AssistSupport will:
1. Generate a new master encryption key
2. Store it securely in your macOS Keychain
3. Create an encrypted database

You'll be prompted to allow Keychain access - click "Allow" to proceed.

### Uninstallation

1. Quit AssistSupport if running
2. Delete `/Applications/AssistSupport.app`
3. Optional: Delete app data:
   ```bash
   rm -rf ~/Library/Application\ Support/com.d.assistsupport
   ```
4. Optional: Remove Keychain entry:
   - Open Keychain Access
   - Search for "com.d.assistsupport"
   - Delete the entry

### Troubleshooting

**"App is damaged and can't be opened"**
```bash
xattr -cr /Applications/AssistSupport.app
```

**Keychain access denied**
- Open Keychain Access
- Right-click the AssistSupport entry
- Select "Get Info" > "Access Control"
- Add AssistSupport to allowed applications

**Database errors**
- Ensure ~/Library/Application Support/com.d.assistsupport exists and is writable
- Check Console.app for detailed error messages
