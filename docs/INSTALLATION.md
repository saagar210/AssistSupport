# Installation Guide

## System Requirements

### Hardware
- **CPU**: Apple Silicon (M1/M2/M3/M4) or Intel x86_64
- **RAM**: 8 GB minimum, 16 GB+ recommended for larger models
- **Storage**: 2 GB for application, 5-20 GB per LLM model

### Software
- **macOS**: 13.0 (Ventura) or later
- **Node.js**: 20.x or later
- **Rust**: 1.75 or later
- **pnpm**: 8.x or later (or npm)

### Optional Dependencies
- **yt-dlp**: Required for YouTube transcript ingestion
  ```bash
  brew install yt-dlp
  ```

## Installation from Source

### 1. Clone Repository
```bash
git clone https://github.com/your-org/assistsupport.git
cd assistsupport
```

### 2. Install Dependencies
```bash
# Install Node.js dependencies
pnpm install

# Rust dependencies are handled by Cargo automatically
```

### 3. Build and Run

#### Development Mode
```bash
pnpm tauri dev
```

#### Production Build
```bash
pnpm tauri build
```

The built application will be in `src-tauri/target/release/bundle/macos/`.

## First Run Setup

### 1. Key Storage Selection
On first launch, the onboarding wizard guides you through:
- **Keychain Mode** (recommended): Master key stored in macOS Keychain
- **Passphrase Mode**: Master key protected by your passphrase

### 2. Model Selection
Choose an LLM model:
- **Llama 3.2 3B** (recommended): Best balance of quality and speed
- **Phi 3 Mini 4K**: Strong reasoning, higher RAM usage
- **Llama 3.2 1B**: Fastest, lowest RAM usage
- **Custom GGUF**: Load your own model file

### 3. Knowledge Base Setup
Configure your knowledge base folder:
- Must be within your home directory
- Supports markdown, PDF, DOCX, XLSX
- Subfolders are indexed recursively

## Configuration Files

### Application Data Location
```
~/Library/Application Support/com.d.assistsupport/
├── data/
│   └── assistsupport.db     # Encrypted SQLite database
├── vectors/
│   └── lance/               # Vector store data
├── audit.log                # Security audit log
├── mode.txt                 # Key storage mode
├── tokens.json              # Encrypted API tokens
└── wrapped.key              # Passphrase-wrapped key (if using passphrase mode)
```

### Model Storage
```
~/Library/Application Support/com.d.assistsupport/models/
├── Llama-3.2-1B-Instruct-Q4_K_M.gguf
├── Llama-3.2-3B-Instruct-Q4_K_M.gguf
├── Phi-3.1-mini-4k-instruct-Q4_K_M.gguf
└── nomic-embed-text-v1.5.Q5_K_M.gguf
```

## Upgrading

### From Previous Versions

1. **Backup your data** (Settings > Export Backup)
2. Replace the application
3. Launch the new version
4. Data migration runs automatically

### Key Migration
If upgrading from a version without Keychain support:
- Legacy plaintext keys are automatically migrated to Keychain
- No action required on your part
- Migration logged in audit.log

## Troubleshooting

### "Cannot open database"
- Delete and recreate: `rm -rf ~/Library/Application\ Support/com.d.assistsupport/data`
- Re-index your knowledge base

### "Model failed to load"
- Verify GGUF file integrity
- Check available RAM (Activity Monitor)
- Try a smaller quantization (Q4 instead of Q8)

### "Keychain access denied"
- Grant keychain access when prompted
- Or switch to passphrase mode in Settings

### Build Errors
```bash
# Clean and rebuild
cd src-tauri && cargo clean
pnpm tauri build
```

## Uninstallation

### Remove Application
1. Quit AssistSupport
2. Delete from Applications folder
3. Remove application data:
   ```bash
   rm -rf ~/Library/Application\ Support/com.d.assistsupport
   ```

### Remove from Keychain
```bash
security delete-generic-password -s "com.d.assistsupport.master-key" 2>/dev/null
```
