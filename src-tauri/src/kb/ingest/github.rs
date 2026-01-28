//! GitHub repository ingestion module for AssistSupport
//! Indexes local Git repositories and supports private repos via token

use super::{
    CancellationToken, IngestError, IngestPhase, IngestProgress, IngestResult, IngestedDocument,
    ProgressCallback,
};
use crate::db::{Database, IngestRunCompletion, IngestSource};
use crate::kb::indexer::{KbIndexer, ParsedDocument, Section};
use crate::security::create_secure_dir;
use base64::engine::general_purpose;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;
use walkdir::WalkDir;

/// GitHub ingestion configuration
#[derive(Debug, Clone)]
pub struct GitHubIngestConfig {
    /// Allowed root directories for local repos
    pub allowed_roots: Vec<PathBuf>,
    /// Maximum file size to index
    pub max_file_size: usize,
    /// Maximum total repository size
    pub max_repo_size: usize,
    /// Maximum size for a single cloned repository
    pub max_clone_size: u64,
    /// Maximum total cache size for cloned repositories
    pub max_cache_size: u64,
    /// Cache root for cloned repositories
    pub cache_root: PathBuf,
    /// File extensions to index
    pub allowed_extensions: Vec<String>,
    /// Directories to skip
    pub skip_dirs: Vec<String>,
    /// Files to skip
    pub skip_files: Vec<String>,
}

impl Default for GitHubIngestConfig {
    fn default() -> Self {
        let app_dir = dirs::data_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("AssistSupport");
        Self {
            allowed_roots: vec![dirs::home_dir().unwrap_or_default()],
            max_file_size: 1024 * 1024,              // 1MB per file
            max_repo_size: 100 * 1024 * 1024,        // 100MB total
            max_clone_size: 1024 * 1024 * 1024,      // 1GB per repo clone
            max_cache_size: 10 * 1024 * 1024 * 1024, // 10GB total cache
            cache_root: app_dir.join("github-cache"),
            allowed_extensions: vec![
                // Documentation
                "md".into(),
                "mdx".into(),
                "txt".into(),
                "rst".into(),
                "adoc".into(),
                // Code - common languages
                "py".into(),
                "js".into(),
                "ts".into(),
                "jsx".into(),
                "tsx".into(),
                "rs".into(),
                "go".into(),
                "java".into(),
                "kt".into(),
                "swift".into(),
                "c".into(),
                "cpp".into(),
                "h".into(),
                "hpp".into(),
                "cs".into(),
                "rb".into(),
                "php".into(),
                "pl".into(),
                // Config
                "json".into(),
                "yaml".into(),
                "yml".into(),
                "toml".into(),
                "xml".into(),
                "ini".into(),
                "cfg".into(),
                // Web
                "html".into(),
                "css".into(),
                "scss".into(),
                "less".into(),
                // Shell
                "sh".into(),
                "bash".into(),
                "zsh".into(),
                "fish".into(),
            ],
            skip_dirs: vec![
                ".git".into(),
                "node_modules".into(),
                "target".into(),
                "build".into(),
                "dist".into(),
                "__pycache__".into(),
                ".venv".into(),
                "venv".into(),
                ".tox".into(),
                ".pytest_cache".into(),
                ".mypy_cache".into(),
                "vendor".into(),
                ".cargo".into(),
                "Pods".into(),
            ],
            skip_files: vec![
                "package-lock.json".into(),
                "yarn.lock".into(),
                "Cargo.lock".into(),
                "poetry.lock".into(),
                "Gemfile.lock".into(),
                ".DS_Store".into(),
                "Thumbs.db".into(),
            ],
        }
    }
}

/// Repository file to index
#[derive(Debug)]
struct RepoFile {
    path: PathBuf,
    relative_path: String,
    #[allow(dead_code)]
    size: u64,
}

#[derive(Debug)]
struct CacheEntry {
    path: PathBuf,
    size_bytes: u64,
    last_used: SystemTime,
}

/// Parsed GitHub-style HTTPS repository URL
#[derive(Debug, Clone)]
pub struct GitHubRemoteRepo {
    pub host: String,
    pub host_port: String,
    pub owner: String,
    pub repo: String,
}

impl GitHubRemoteRepo {
    pub fn clone_url(&self) -> String {
        format!(
            "https://{}/{}/{}.git",
            self.host_port, self.owner, self.repo
        )
    }

    pub fn display_url(&self) -> String {
        format!("https://{}/{}/{}", self.host_port, self.owner, self.repo)
    }

    pub fn cache_key(&self) -> String {
        format!(
            "{}:{}/{}",
            self.host_port.to_lowercase(),
            self.owner.to_lowercase(),
            self.repo.to_lowercase()
        )
    }
}

fn is_valid_slug(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

fn sanitize_host_for_path(host_port: &str) -> String {
    host_port
        .chars()
        .map(|c| if c == ':' { '_' } else { c })
        .collect()
}

/// Parse an HTTPS repo URL into host/owner/repo components
pub fn parse_https_repo_url(url: &str) -> IngestResult<GitHubRemoteRepo> {
    let parsed =
        Url::parse(url).map_err(|e| IngestError::InvalidSource(format!("Invalid URL: {}", e)))?;

    if parsed.scheme() != "https" {
        return Err(IngestError::InvalidSource(
            "Repository URL must use HTTPS".to_string(),
        ));
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(IngestError::InvalidSource(
            "Repository URL must not include credentials".to_string(),
        ));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| IngestError::InvalidSource("Repository URL missing host".to_string()))?;
    if !host
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return Err(IngestError::InvalidSource(
            "Repository host contains invalid characters".to_string(),
        ));
    }
    let host_port = if let Some(port) = parsed.port() {
        format!("{}:{}", host, port)
    } else {
        host.to_string()
    };

    let mut path_parts = parsed
        .path()
        .trim_matches('/')
        .split('/')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>();

    if path_parts.len() < 2 {
        return Err(IngestError::InvalidSource(
            "Repository URL must be in the form https://host/owner/repo".to_string(),
        ));
    }

    let owner = path_parts.remove(0);
    let mut repo = path_parts.remove(0).to_string();
    if repo.ends_with(".git") {
        repo = repo.trim_end_matches(".git").to_string();
    }

    if !path_parts.is_empty() {
        return Err(IngestError::InvalidSource(
            "Repository URL must not include extra path segments".to_string(),
        ));
    }

    if !is_valid_slug(owner) || !is_valid_slug(&repo) {
        return Err(IngestError::InvalidSource(
            "Repository owner and name must use only letters, numbers, '.', '-', or '_'"
                .to_string(),
        ));
    }

    Ok(GitHubRemoteRepo {
        host: host.to_string(),
        host_port,
        owner: owner.to_string(),
        repo,
    })
}

/// GitHub repository ingester
pub struct GitHubIngester {
    config: GitHubIngestConfig,
}

impl GitHubIngester {
    /// Create a new GitHub ingester
    pub fn new(config: GitHubIngestConfig) -> Self {
        Self { config }
    }

    /// Validate that a path is within allowed roots
    fn validate_repo_path(&self, path: &Path) -> IngestResult<PathBuf> {
        // Canonicalize the path
        let canonical = path
            .canonicalize()
            .map_err(|e| IngestError::InvalidSource(format!("Invalid path: {}", e)))?;

        // Check it's within one of the allowed roots
        for root in &self.config.allowed_roots {
            if let Ok(root_canonical) = root.canonicalize() {
                if canonical.starts_with(&root_canonical) {
                    return Ok(canonical);
                }
            }
        }

        Err(IngestError::InvalidSource(format!(
            "Path {} is not within allowed roots",
            path.display()
        )))
    }

    /// Check if a path is a Git repository
    fn is_git_repo(path: &Path) -> bool {
        path.join(".git").exists()
    }

    /// Get repository name from path
    fn repo_name(path: &Path) -> String {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Check if a directory should be skipped
    fn should_skip_dir(&self, name: &str) -> bool {
        self.config.skip_dirs.iter().any(|d| d == name)
    }

    /// Check if a file should be skipped
    fn should_skip_file(&self, name: &str) -> bool {
        self.config.skip_files.iter().any(|f| f == name)
    }

    /// Check if a file extension is allowed
    fn is_allowed_extension(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| self.config.allowed_extensions.iter().any(|a| a == ext))
            .unwrap_or(false)
    }

    /// Discover files in a repository
    fn discover_files(&self, repo_path: &Path) -> IngestResult<Vec<RepoFile>> {
        let mut files = Vec::new();
        let mut total_size: u64 = 0;

        for entry in WalkDir::new(repo_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Skip hidden directories (except .github)
                if let Some(name) = e.file_name().to_str() {
                    if name.starts_with('.') && name != ".github" {
                        return false;
                    }
                    if e.file_type().is_dir() && self.should_skip_dir(name) {
                        return false;
                    }
                }
                true
            })
        {
            let entry = entry.map_err(|e| IngestError::Io(e.into()))?;

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip certain files
            if self.should_skip_file(file_name) {
                continue;
            }

            // Check extension
            if !self.is_allowed_extension(path) {
                continue;
            }

            // Check file size
            let metadata = entry.metadata().map_err(|e| IngestError::Io(e.into()))?;
            let size = metadata.len();

            if size > self.config.max_file_size as u64 {
                tracing::debug!("Skipping large file: {} ({} bytes)", path.display(), size);
                continue;
            }

            total_size += size;
            if total_size > self.config.max_repo_size as u64 {
                return Err(IngestError::ContentTooLarge {
                    size: total_size as usize,
                    max: self.config.max_repo_size,
                });
            }

            let relative_path = path
                .strip_prefix(repo_path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());

            files.push(RepoFile {
                path: path.to_path_buf(),
                relative_path,
                size,
            });
        }

        Ok(files)
    }

    /// Read and parse a file
    fn read_file(&self, file: &RepoFile) -> IngestResult<String> {
        let content = std::fs::read(&file.path).map_err(IngestError::Io)?;

        // Try to decode as UTF-8, fall back to lossy conversion
        Ok(String::from_utf8(content)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).to_string()))
    }

    fn ensure_cache_root(&self) -> IngestResult<()> {
        create_secure_dir(&self.config.cache_root).map_err(IngestError::Io)
    }

    fn cache_dir_for(&self, remote: &GitHubRemoteRepo) -> PathBuf {
        let key = remote.cache_key();
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hex::encode(hasher.finalize());
        let short_hash = &hash[..12];

        let host_dir = sanitize_host_for_path(&remote.host_port);
        let repo_dir = format!("{}-{}", remote.repo, short_hash);

        self.config
            .cache_root
            .join(host_dir)
            .join(&remote.owner)
            .join(repo_dir)
    }

    fn write_cache_marker(path: &Path) -> IngestResult<()> {
        let marker = path.join(".last_used");
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        std::fs::write(marker, ts.to_string()).map_err(IngestError::Io)
    }

    fn read_cache_marker(path: &Path) -> Option<SystemTime> {
        let marker = path.join(".last_used");
        if let Ok(contents) = std::fs::read_to_string(marker) {
            if let Ok(secs) = contents.trim().parse::<u64>() {
                return UNIX_EPOCH.checked_add(std::time::Duration::from_secs(secs));
            }
        }
        path.metadata().and_then(|m| m.modified()).ok()
    }

    fn dir_size(path: &Path) -> IngestResult<u64> {
        let mut total = 0u64;
        for entry in WalkDir::new(path).follow_links(false) {
            let entry = entry.map_err(|e| IngestError::Io(e.into()))?;
            if entry.file_type().is_file() {
                total = total.saturating_add(
                    entry
                        .metadata()
                        .map_err(|e| IngestError::Io(e.into()))?
                        .len(),
                );
            }
        }
        Ok(total)
    }

    fn list_cache_entries(&self) -> IngestResult<Vec<CacheEntry>> {
        let mut entries = Vec::new();
        if !self.config.cache_root.exists() {
            return Ok(entries);
        }

        for entry in WalkDir::new(&self.config.cache_root)
            .min_depth(3)
            .max_depth(6)
            .follow_links(false)
        {
            let entry = entry.map_err(|e| IngestError::Io(e.into()))?;
            if !entry.file_type().is_dir() {
                continue;
            }
            let path = entry.path();
            if !path.join(".git").exists() {
                continue;
            }
            let size = Self::dir_size(path)?;
            let last_used = Self::read_cache_marker(path).unwrap_or_else(SystemTime::now);
            entries.push(CacheEntry {
                path: path.to_path_buf(),
                size_bytes: size,
                last_used,
            });
        }

        Ok(entries)
    }

    fn enforce_cache_limit(&self) -> IngestResult<()> {
        let mut entries = self.list_cache_entries()?;
        let mut total: u64 = entries.iter().map(|e| e.size_bytes).sum();

        if total <= self.config.max_cache_size {
            return Ok(());
        }

        entries.sort_by_key(|e| e.last_used);

        for entry in entries {
            if total <= self.config.max_cache_size {
                break;
            }
            if entry.path.starts_with(&self.config.cache_root) {
                let _ = std::fs::remove_dir_all(&entry.path);
                total = total.saturating_sub(entry.size_bytes);
            }
        }

        Ok(())
    }

    fn build_auth_header(token: &str) -> String {
        let auth = format!("x-access-token:{}", token);
        general_purpose::STANDARD.encode(auth.as_bytes())
    }

    fn run_git(
        &self,
        args: &[&str],
        token: Option<&str>,
        workdir: Option<&Path>,
    ) -> IngestResult<()> {
        let mut command = Command::new("git");

        if let Some(token) = token {
            if !token.trim().is_empty() {
                let header = Self::build_auth_header(token.trim());
                command
                    .arg("-c")
                    .arg(format!("http.extraHeader=Authorization: Basic {}", header));
            }
        }

        if let Some(dir) = workdir {
            command.arg("-C").arg(dir);
        }

        command.args(args);
        command.env("GIT_TERMINAL_PROMPT", "0");

        let output = command.output().map_err(IngestError::Io)?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let err_lower = stderr.to_lowercase();

        if err_lower.contains("authentication")
            || err_lower.contains("403")
            || err_lower.contains("401")
        {
            return Err(IngestError::AuthRequired(
                "Authentication failed for GitHub repository".to_string(),
            ));
        }

        Err(IngestError::InvalidSource(if stderr.is_empty() {
            "Git command failed".to_string()
        } else {
            stderr
        }))
    }

    fn clone_or_update_repo(
        &self,
        remote: &GitHubRemoteRepo,
        token: Option<&str>,
        cancel_token: &CancellationToken,
    ) -> IngestResult<PathBuf> {
        if cancel_token.is_cancelled() {
            return Err(IngestError::Cancelled);
        }

        self.ensure_cache_root()?;
        let repo_dir = self.cache_dir_for(remote);

        if repo_dir.exists() {
            if !Self::is_git_repo(&repo_dir) {
                let _ = std::fs::remove_dir_all(&repo_dir);
            } else {
                self.run_git(&["fetch", "--depth", "1", "origin"], token, Some(&repo_dir))?;
                self.run_git(&["reset", "--hard", "FETCH_HEAD"], None, Some(&repo_dir))?;
                Self::write_cache_marker(&repo_dir)?;
                self.enforce_cache_limit()?;
                return Ok(repo_dir);
            }
        }

        if let Some(parent) = repo_dir.parent() {
            create_secure_dir(parent).map_err(IngestError::Io)?;
        }

        let url = remote.clone_url();
        let dest = repo_dir.to_string_lossy().to_string();
        self.run_git(
            &["clone", "--depth", "1", "--single-branch", &url, &dest],
            token,
            None,
        )?;

        Self::write_cache_marker(&repo_dir)?;

        let size = Self::dir_size(&repo_dir)?;
        if size > self.config.max_clone_size {
            let _ = std::fs::remove_dir_all(&repo_dir);
            return Err(IngestError::ContentTooLarge {
                size: size as usize,
                max: self.config.max_clone_size as usize,
            });
        }

        self.enforce_cache_limit()?;

        Ok(repo_dir)
    }

    #[allow(clippy::too_many_arguments)]
    fn ingest_repo_internal(
        &self,
        db: &Database,
        repo_path: &Path,
        namespace_id: &str,
        cancel_token: &CancellationToken,
        progress: Option<&ProgressCallback>,
        source_uri: String,
        repo_name: String,
        metadata: serde_json::Value,
    ) -> IngestResult<Vec<IngestedDocument>> {
        if cancel_token.is_cancelled() {
            return Err(IngestError::Cancelled);
        }

        // Report progress
        if let Some(progress) = progress {
            progress(IngestProgress {
                phase: IngestPhase::Fetching,
                current: 0,
                total: None,
                message: format!("Discovering files in {}", repo_name),
            });
        }

        // Discover files
        let files = self.discover_files(repo_path)?;
        let total_files = files.len();

        if total_files == 0 {
            return Err(IngestError::NotFound(format!(
                "No indexable files found in {}",
                repo_name
            )));
        }

        let now = chrono::Utc::now().to_rfc3339();
        let mut metadata = metadata;
        if let Some(obj) = metadata.as_object_mut() {
            obj.insert("file_count".to_string(), serde_json::json!(total_files));
        }

        // Create or update source
        let source = match db.find_ingest_source("github", &source_uri, namespace_id)? {
            Some(mut existing) => {
                existing.title = Some(repo_name.clone());
                existing.last_ingested_at = Some(now.clone());
                existing.status = "active".to_string();
                existing.updated_at = now.clone();
                db.save_ingest_source(&existing)?;
                existing
            }
            None => {
                let source = IngestSource {
                    id: uuid::Uuid::new_v4().to_string(),
                    source_type: "github".to_string(),
                    source_uri: source_uri.clone(),
                    namespace_id: namespace_id.to_string(),
                    title: Some(repo_name.clone()),
                    etag: None,
                    last_modified: None,
                    content_hash: None,
                    last_ingested_at: Some(now.clone()),
                    status: "active".to_string(),
                    error_message: None,
                    metadata_json: Some(metadata.to_string()),
                    created_at: now.clone(),
                    updated_at: now.clone(),
                };
                db.save_ingest_source(&source)?;
                source
            }
        };

        // Create ingest run
        let run_id = db.create_ingest_run(&source.id)?;

        // Delete existing documents for this source
        db.delete_documents_for_source(&source.id)?;

        let mut documents = Vec::new();
        let mut total_chunks = 0;
        let mut errors = Vec::new();

        // Process each file
        for (i, file) in files.iter().enumerate() {
            if cancel_token.is_cancelled() {
                db.complete_ingest_run(IngestRunCompletion {
                    run_id: &run_id,
                    status: "cancelled",
                    docs_added: documents.len() as i32,
                    docs_updated: 0,
                    docs_removed: 0,
                    chunks_added: total_chunks,
                    error_message: None,
                })?;
                return Err(IngestError::Cancelled);
            }

            // Report progress
            if let Some(progress) = progress {
                progress(IngestProgress {
                    phase: IngestPhase::Parsing,
                    current: i,
                    total: Some(total_files),
                    message: format!("Processing {}", file.relative_path),
                });
            }

            // Read file
            let content = match self.read_file(file) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!("{}: {}", file.relative_path, e));
                    continue;
                }
            };

            // Skip empty or binary files
            if content.is_empty() || content.contains('\0') {
                continue;
            }

            // Create parsed document with sections based on code headings
            let title = file.relative_path.clone();
            let headings = extract_code_headings(&content, &file.relative_path);
            let sections = build_sections_from_code(&content, &headings);
            let parsed = ParsedDocument {
                title: Some(title.clone()),
                sections,
            };

            // Chunk the document
            let indexer = KbIndexer::new();
            let chunks = indexer.chunk_document(&parsed);
            let chunk_count = chunks.len();
            let word_count = content.split_whitespace().count();

            if chunk_count == 0 {
                continue;
            }

            // Insert document
            let doc_id = uuid::Uuid::new_v4().to_string();
            let content_hash = {
                let mut hasher = Sha256::new();
                hasher.update(content.as_bytes());
                hex::encode(hasher.finalize())
            };

            db.conn().execute(
                "INSERT INTO kb_documents (id, file_path, file_hash, title, indexed_at, chunk_count,
                        namespace_id, source_type, source_id)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![
                    doc_id,
                    format!("{}/{}", source_uri, file.relative_path),
                    content_hash,
                    title,
                    now,
                    chunk_count as i32,
                    namespace_id,
                    "github",
                    source.id,
                ],
            )?;

            // Insert chunks
            for (j, chunk) in chunks.iter().enumerate() {
                let chunk_id = uuid::Uuid::new_v4().to_string();
                db.conn().execute(
                    "INSERT INTO kb_chunks (id, document_id, chunk_index, heading_path, content, word_count, namespace_id)
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                    rusqlite::params![
                        chunk_id,
                        doc_id,
                        j as i32,
                        chunk.heading_path,
                        chunk.content,
                        chunk.word_count as i32,
                        namespace_id,
                    ],
                )?;
            }

            total_chunks += chunk_count as i32;
            documents.push(IngestedDocument {
                id: doc_id,
                title,
                source_uri: format!("{}/{}", source_uri, file.relative_path),
                chunk_count,
                word_count,
            });
        }

        // Complete ingest run
        let error_msg = if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        };
        db.complete_ingest_run(IngestRunCompletion {
            run_id: &run_id,
            status: if errors.is_empty() {
                "completed"
            } else {
                "completed_with_errors"
            },
            docs_added: documents.len() as i32,
            docs_updated: 0,
            docs_removed: 0,
            chunks_added: total_chunks,
            error_message: error_msg.as_deref(),
        })?;

        // Report progress
        if let Some(progress) = progress {
            progress(IngestProgress {
                phase: IngestPhase::Complete,
                current: documents.len(),
                total: Some(total_files),
                message: format!(
                    "Indexed {} files ({} chunks)",
                    documents.len(),
                    total_chunks
                ),
            });
        }

        Ok(documents)
    }

    /// Ingest a local Git repository into the knowledge base
    pub fn ingest_local_repo(
        &self,
        db: &Database,
        repo_path: &Path,
        namespace_id: &str,
        cancel_token: &CancellationToken,
        progress: Option<&ProgressCallback>,
    ) -> IngestResult<Vec<IngestedDocument>> {
        // Validate path
        let validated_path = self.validate_repo_path(repo_path)?;

        // Check it's a Git repo
        if !Self::is_git_repo(&validated_path) {
            return Err(IngestError::InvalidSource(format!(
                "{} is not a Git repository",
                repo_path.display()
            )));
        }

        let repo_name = Self::repo_name(&validated_path);
        let source_uri = format!("github://local/{}", validated_path.to_string_lossy());
        let metadata = serde_json::json!({
            "path": validated_path.to_string_lossy(),
        });

        self.ingest_repo_internal(
            db,
            &validated_path,
            namespace_id,
            cancel_token,
            progress,
            source_uri,
            repo_name,
            metadata,
        )
    }

    /// Ingest a remote GitHub repository into the knowledge base
    pub fn ingest_remote_repo(
        &self,
        db: &Database,
        repo_url: &str,
        token: Option<&str>,
        namespace_id: &str,
        cancel_token: &CancellationToken,
        progress: Option<&ProgressCallback>,
    ) -> IngestResult<Vec<IngestedDocument>> {
        let remote = parse_https_repo_url(repo_url)?;

        if let Some(progress) = progress {
            progress(IngestProgress {
                phase: IngestPhase::Fetching,
                current: 0,
                total: None,
                message: format!("Cloning {}", remote.display_url()),
            });
        }

        let repo_dir = self.clone_or_update_repo(&remote, token, cancel_token)?;
        let source_uri = format!(
            "github://{}/{}/{}",
            remote.host_port, remote.owner, remote.repo
        );
        let metadata = serde_json::json!({
            "url": remote.display_url(),
            "cache_path": repo_dir.to_string_lossy(),
            "host": remote.host_port,
            "owner": remote.owner,
            "repo": remote.repo,
        });

        self.ingest_repo_internal(
            db,
            &repo_dir,
            namespace_id,
            cancel_token,
            progress,
            source_uri,
            format!("{}/{}", remote.owner, remote.repo),
            metadata,
        )
    }
}

/// Build sections from code content and headings
fn build_sections_from_code(content: &str, headings: &[(usize, String)]) -> Vec<Section> {
    if headings.is_empty() || headings.len() == 1 {
        // Just filename heading or no headings - treat as single section
        return vec![Section {
            heading: headings.first().map(|(_, h)| h.clone()),
            level: 1,
            content: content.to_string(),
        }];
    }

    // For code files, we use the full content as a single section
    // The indexer's code-aware chunking will handle the rest
    vec![Section {
        heading: headings.first().map(|(_, h)| h.clone()),
        level: 1,
        content: content.to_string(),
    }]
}

/// Extract pseudo-headings from code files
fn extract_code_headings(content: &str, filename: &str) -> Vec<(usize, String)> {
    let mut headings = vec![(1, filename.to_string())];

    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    // Extract function/class definitions as headings
    match ext {
        "py" => {
            // Python: class and def
            let class_re = regex_lite::Regex::new(r"^class\s+(\w+)").unwrap();
            let def_re = regex_lite::Regex::new(r"^def\s+(\w+)").unwrap();

            for line in content.lines() {
                if let Some(cap) = class_re.captures(line) {
                    if let Some(name) = cap.get(1) {
                        headings.push((2, format!("class {}", name.as_str())));
                    }
                } else if let Some(cap) = def_re.captures(line) {
                    if let Some(name) = cap.get(1) {
                        headings.push((3, format!("def {}", name.as_str())));
                    }
                }
            }
        }
        "rs" => {
            // Rust: fn, struct, impl, mod
            let fn_re = regex_lite::Regex::new(r"^\s*(pub\s+)?fn\s+(\w+)").unwrap();
            let struct_re = regex_lite::Regex::new(r"^\s*(pub\s+)?struct\s+(\w+)").unwrap();
            let impl_re = regex_lite::Regex::new(r"^\s*impl\s+(\w+)").unwrap();

            for line in content.lines() {
                if let Some(cap) = struct_re.captures(line) {
                    if let Some(name) = cap.get(2) {
                        headings.push((2, format!("struct {}", name.as_str())));
                    }
                } else if let Some(cap) = impl_re.captures(line) {
                    if let Some(name) = cap.get(1) {
                        headings.push((2, format!("impl {}", name.as_str())));
                    }
                } else if let Some(cap) = fn_re.captures(line) {
                    if let Some(name) = cap.get(2) {
                        headings.push((3, format!("fn {}", name.as_str())));
                    }
                }
            }
        }
        "js" | "ts" | "jsx" | "tsx" => {
            // JavaScript/TypeScript: function, class
            let class_re =
                regex_lite::Regex::new(r"^\s*(export\s+)?(class|interface)\s+(\w+)").unwrap();
            let fn_re =
                regex_lite::Regex::new(r"^\s*(export\s+)?(async\s+)?function\s+(\w+)").unwrap();

            for line in content.lines() {
                if let Some(cap) = class_re.captures(line) {
                    if let Some(name) = cap.get(3) {
                        headings.push((2, format!("class {}", name.as_str())));
                    }
                } else if let Some(cap) = fn_re.captures(line) {
                    if let Some(name) = cap.get(3) {
                        headings.push((3, format!("function {}", name.as_str())));
                    }
                }
            }
        }
        _ => {}
    }

    headings
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_is_git_repo() {
        let dir = tempdir().unwrap();
        assert!(!GitHubIngester::is_git_repo(dir.path()));

        std::fs::create_dir(dir.path().join(".git")).unwrap();
        assert!(GitHubIngester::is_git_repo(dir.path()));
    }

    #[test]
    fn test_repo_name() {
        assert_eq!(
            GitHubIngester::repo_name(Path::new("/home/user/my-repo")),
            "my-repo"
        );
        assert_eq!(
            GitHubIngester::repo_name(Path::new("/var/projects/awesome-project")),
            "awesome-project"
        );
    }

    #[test]
    fn test_parse_https_repo_url() {
        let repo = parse_https_repo_url("https://github.com/owner/repo").unwrap();
        assert_eq!(repo.host, "github.com");
        assert_eq!(repo.host_port, "github.com");
        assert_eq!(repo.owner, "owner");
        assert_eq!(repo.repo, "repo");

        let ghe = parse_https_repo_url("https://ghe.local:8443/team/core.git").unwrap();
        assert_eq!(ghe.host_port, "ghe.local:8443");
        assert_eq!(ghe.owner, "team");
        assert_eq!(ghe.repo, "core");
    }

    #[test]
    fn test_parse_https_repo_url_rejects_http() {
        let err = parse_https_repo_url("http://github.com/owner/repo").unwrap_err();
        assert!(err.to_string().contains("HTTPS"));
    }

    #[test]
    fn test_extract_code_headings_python() {
        let content = r#"
class MyClass:
    def __init__(self):
        pass

def my_function():
    pass
"#;
        let headings = extract_code_headings(content, "test.py");
        assert!(headings.iter().any(|(_, h)| h == "class MyClass"));
        assert!(headings.iter().any(|(_, h)| h == "def my_function"));
    }

    #[test]
    fn test_extract_code_headings_rust() {
        let content = r#"
pub struct MyStruct {
    field: i32,
}

impl MyStruct {
    pub fn new() -> Self {
        Self { field: 0 }
    }
}

fn helper() {}
"#;
        let headings = extract_code_headings(content, "test.rs");
        assert!(headings.iter().any(|(_, h)| h == "struct MyStruct"));
        assert!(headings.iter().any(|(_, h)| h == "impl MyStruct"));
        assert!(headings.iter().any(|(_, h)| h == "fn new"));
    }
}
