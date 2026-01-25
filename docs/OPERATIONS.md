# Operations Guide

## Daily Workflow

### Starting Your Session
1. Launch AssistSupport
2. Verify model loaded (green indicator in header)
3. Check KB status (indexed document count)

### Creating Support Responses

1. **Enter Context**
   - Paste customer query in input field
   - Optionally add Jira ticket ID for context

2. **Review KB Matches**
   - Check "Sources" tab for relevant documents
   - Verify search results are current

3. **Generate Response**
   - Press `Cmd+Enter` or click Generate
   - Response streams in real-time

4. **Refine and Export**
   - Edit generated text as needed
   - `Cmd+Shift+C` to copy
   - `Cmd+E` to export

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+K` | Open command palette |
| `Cmd+Enter` | Generate response |
| `Cmd+S` | Save draft |
| `Cmd+Shift+C` | Copy response |
| `Cmd+E` | Export response |
| `Cmd+1-6` | Switch tabs |
| `Cmd+N` | New draft |
| `Cmd+/` | Focus search |
| `Escape` | Cancel generation |

## Knowledge Base Management

### Adding Content

#### Local Files
1. Go to Settings > Knowledge Base
2. Set KB folder (must be in home directory)
3. Click "Re-index" to scan for new files

Supported formats:
- Markdown (.md)
- PDF (.pdf)
- Word documents (.docx)
- Excel spreadsheets (.xlsx)
- Code files (various)

#### Web Content
1. Go to "Ingest" tab
2. Enter URL
3. Select namespace
4. Click "Ingest"

#### YouTube Transcripts
1. Requires `yt-dlp` installed
2. Paste YouTube URL in Ingest tab
3. Transcripts are extracted and indexed

#### GitHub Repositories
1. Supports public repos or local clones
2. Indexes README, docs/, and code comments

### Organizing with Namespaces

Namespaces separate knowledge:
- `default` - General knowledge
- `product-x` - Product-specific docs
- `internal` - Internal procedures

To create namespace:
1. Settings > Namespaces
2. Click "Add Namespace"
3. Enter ID and description

To move documents:
1. Delete from current namespace
2. Re-ingest with new namespace selected

### Keeping Content Fresh

#### Automatic Watching
Enable file watcher in Settings:
- Monitors KB folder for changes
- Re-indexes modified files automatically
- Respects debounce (5 second delay)

#### Manual Refresh
- Re-index single file: Delete and re-add
- Full re-index: Settings > Re-index KB

#### Stale Content Detection
Check "Sources" tab for:
- Last indexed date
- Health status
- Error messages

## Draft Management

### Saving Drafts

Drafts are auto-saved every 30 seconds.

Manual save: `Cmd+S` or click Save button.

Draft metadata:
- Title (auto-generated or manual)
- Tags (comma-separated)
- Associated ticket ID
- Version history

### Finding Drafts

1. Go to "Drafts" tab
2. Search by keyword or ticket ID
3. Filter by date range or tags

### Version History

Each save creates a version:
1. Open draft
2. Click "History" button
3. View and restore previous versions

### Finalizing Drafts

Mark draft as complete:
1. Open draft
2. Click "Finalize"
3. Draft moves to "Completed" section

## Model Management

### Switching Models

1. Settings > Model
2. Select from downloaded models
3. Click "Load"
4. Wait for model initialization (~5-15 seconds)

### Downloading New Models

1. Settings > Model > Available Models
2. Click download icon
3. Monitor progress in Downloads tab
4. Model ready when download completes

### Custom Models

Load any GGUF model:
1. Settings > Model > Custom Model
2. Browse to .gguf file
3. Click "Load"

Note: Custom models show as "Unverified" - hash not in allowlist.

### Context Window

Adjust context size:
1. Settings > Model > Context Window
2. Drag slider or enter value
3. Lower = faster, less context
4. Higher = slower, more context

## Integrations

### Jira Integration

#### Setup
1. Settings > Integrations > Jira
2. Enter instance URL (e.g., https://company.atlassian.net)
3. Enter email and API token
4. Test connection

#### Usage
- Enter ticket ID (e.g., PROJ-123) in draft
- Ticket context auto-loaded
- Responses include ticket reference

### HuggingFace Integration

Required for gated model downloads:
1. Settings > Integrations > HuggingFace
2. Enter API token from hf.co/settings/tokens
3. Accept model licenses on HuggingFace

## Backup and Restore

### Creating Backups

1. Settings > Backup > Export
2. Choose what to include:
   - Drafts and templates
   - Custom variables
   - Settings
   - KB metadata (not files)
3. Optionally set password
4. Choose save location

### Restoring Backups

1. Settings > Backup > Import
2. Select backup file
3. Enter password if encrypted
4. Preview changes
5. Confirm import

### What's Included

| Data | Backed Up | Notes |
|------|-----------|-------|
| Drafts | Yes | Full content and metadata |
| Templates | Yes | All custom templates |
| Custom Variables | Yes | All variables |
| Settings | Yes | App preferences |
| KB Documents | Metadata only | Re-index from source files |
| Models | No | Re-download after restore |

## Troubleshooting

### Health Check

1. Settings > Diagnostics
2. View component health
3. Check for issues

Health statuses:
- 🟢 Healthy - Working normally
- 🟡 Degraded - Working with issues
- 🔴 Error - Not functioning

### Common Issues

#### "Model not loading"
1. Check available RAM
2. Try smaller model/quantization
3. Restart application

#### "Search returns no results"
1. Verify KB folder is set
2. Check document count > 0
3. Run re-index

#### "Generation very slow"
1. Reduce context window
2. Use smaller model
3. Close other apps

### Logs

View application logs:
```bash
# Application logs (Console.app)
log show --predicate 'process == "AssistSupport"' --last 1h

# Audit log
cat ~/Library/Application\ Support/com.d.assistsupport/audit.log
```

### Reset to Defaults

Full reset (caution - deletes all data):
```bash
rm -rf ~/Library/Application\ Support/com.d.assistsupport
```

Then restart the application.

## Maintenance Schedule

### Daily
- None required (auto-managed)

### Weekly
- Review stale sources
- Clean up old drafts
- Check disk space

### Monthly
- Update application
- Review audit log
- Verify backups

### Quarterly
- Full backup export
- Review namespace organization
- Evaluate model performance
