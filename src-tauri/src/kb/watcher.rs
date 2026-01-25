//! File watcher for KB auto-sync
//! Watches KB folder for changes and triggers incremental indexing

use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, FileIdMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;

use super::indexer::DocumentType;

/// Maximum number of files to watch (prevent resource exhaustion)
const MAX_WATCHED_FILES: usize = 10_000;

/// Debounce duration for file events
const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);

#[derive(Debug, Error)]
pub enum WatcherError {
    #[error("Watcher error: {0}")]
    Notify(#[from] notify::Error),
    #[error("Path not found: {0}")]
    PathNotFound(String),
    #[error("Too many files to watch (max: {0})")]
    TooManyFiles(usize),
    #[error("Watcher already running")]
    AlreadyRunning,
    #[error("Watcher not running")]
    NotRunning,
}

/// File change event for KB
#[derive(Debug, Clone, serde::Serialize)]
pub enum KbFileEvent {
    Created { path: String },
    Modified { path: String },
    Removed { path: String },
}

/// KB file watcher
pub struct KbWatcher {
    folder: PathBuf,
    running: Arc<AtomicBool>,
    event_tx: Option<mpsc::Sender<KbFileEvent>>,
}

impl KbWatcher {
    /// Create a new watcher for a KB folder
    pub fn new(folder: &Path) -> Result<Self, WatcherError> {
        if !folder.exists() {
            return Err(WatcherError::PathNotFound(folder.display().to_string()));
        }

        Ok(Self {
            folder: folder.to_path_buf(),
            running: Arc::new(AtomicBool::new(false)),
            event_tx: None,
        })
    }

    /// Check if watcher is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Start watching for file changes
    /// Returns a receiver for file events
    pub fn start(&mut self) -> Result<mpsc::Receiver<KbFileEvent>, WatcherError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(WatcherError::AlreadyRunning);
        }

        // Count files to ensure we don't exceed limit
        let file_count = count_files(&self.folder)?;
        if file_count > MAX_WATCHED_FILES {
            return Err(WatcherError::TooManyFiles(MAX_WATCHED_FILES));
        }

        let (tx, rx) = mpsc::channel(100);
        self.event_tx = Some(tx.clone());
        self.running.store(true, Ordering::Relaxed);

        let folder = self.folder.clone();
        let running = self.running.clone();

        // Spawn watcher in background thread
        std::thread::spawn(move || {
            if let Err(e) = run_watcher(folder, tx, running) {
                eprintln!("Watcher error: {}", e);
            }
        });

        Ok(rx)
    }

    /// Stop watching
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.event_tx = None;
    }
}

impl Drop for KbWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Count files in directory (recursive)
fn count_files(dir: &Path) -> Result<usize, WatcherError> {
    let mut count = 0;

    for entry in walkdir::WalkDir::new(dir)
        .max_depth(10) // Limit depth to prevent infinite loops
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if DocumentType::from_extension(ext).is_some() {
                    count += 1;
                    if count > MAX_WATCHED_FILES {
                        return Err(WatcherError::TooManyFiles(MAX_WATCHED_FILES));
                    }
                }
            }
        }
    }

    Ok(count)
}

/// Run the file watcher (blocking, runs in separate thread)
fn run_watcher(
    folder: PathBuf,
    tx: mpsc::Sender<KbFileEvent>,
    running: Arc<AtomicBool>,
) -> Result<(), WatcherError> {
    let (debounce_tx, debounce_rx) = std::sync::mpsc::channel();

    let mut debouncer: Debouncer<RecommendedWatcher, FileIdMap> = new_debouncer(
        DEBOUNCE_DURATION,
        None,
        debounce_tx,
    )?;

    // Debouncer now implements Watcher directly
    debouncer.watch(&folder, RecursiveMode::Recursive)?;

    while running.load(Ordering::Relaxed) {
        match debounce_rx.recv_timeout(Duration::from_millis(500)) {
            Ok(Ok(events)) => {
                for event in events {
                    if let Some(kb_event) = process_event(&event) {
                        // Non-blocking send
                        let _ = tx.try_send(kb_event);
                    }
                }
            }
            Ok(Err(errors)) => {
                for error in errors {
                    eprintln!("Watcher error: {:?}", error);
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Check if we should stop
                continue;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    Ok(())
}

/// Process a notify event into a KB event
fn process_event(event: &DebouncedEvent) -> Option<KbFileEvent> {
    // Only process supported file types
    let path = event.paths.first()?;
    let ext = path.extension().and_then(|e| e.to_str())?;

    if DocumentType::from_extension(ext).is_none() {
        return None;
    }

    let path_str = path.display().to_string();

    match &event.kind {
        EventKind::Create(_) => Some(KbFileEvent::Created { path: path_str }),
        EventKind::Modify(_) => Some(KbFileEvent::Modified { path: path_str }),
        EventKind::Remove(_) => Some(KbFileEvent::Removed { path: path_str }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_count_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("test.md"), "# Test").unwrap();
        fs::write(dir.path().join("test.txt"), "Test").unwrap();
        fs::write(dir.path().join("ignore.xyz"), "Ignored").unwrap();

        let count = count_files(dir.path()).unwrap();
        assert_eq!(count, 2); // .md and .txt are supported
    }

    #[test]
    fn test_watcher_creation() {
        let dir = tempdir().unwrap();
        let watcher = KbWatcher::new(dir.path());
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_watcher_nonexistent_path() {
        let result = KbWatcher::new(Path::new("/nonexistent/path/12345"));
        assert!(matches!(result, Err(WatcherError::PathNotFound(_))));
    }
}
