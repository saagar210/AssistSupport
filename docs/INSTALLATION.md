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
4. Launch the **Onboarding Wizard**

You'll be prompted to allow Keychain access - click "Allow" to proceed.

### Onboarding Wizard

The onboarding wizard guides you through initial setup:

1. **Welcome**: Overview of features and privacy guarantees
2. **Model Download**: Select and download an AI model (Llama 3.2 or Qwen 2.5)
3. **Knowledge Base**: Point to a folder containing your documentation
4. **Complete**: Verify setup and start using the app

You can skip any step and configure settings later. The wizard only appears on first run.

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
- Use the built-in diagnostics panel (Settings > System Health) to run repair tools

**Model not loading**
- Check if you have enough RAM (model size + 2GB headroom)
- Verify the GGUF file isn't corrupted (Settings > Models > Verify)
- Try a smaller model (Llama 3.2 3B uses less memory)

**Search returning poor results**
- Ensure documents are indexed (Knowledge > check document count)
- Try rebuilding the vector store (Settings > System Health > Rebuild)
- Adjust search weights (Sources > Settings icon)
