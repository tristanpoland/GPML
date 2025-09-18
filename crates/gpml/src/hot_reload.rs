use crate::error::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, SystemTime};

/// Hot reload manager for GPML files
pub struct HotReloadManager {
    watcher: Option<RecommendedWatcher>,
    receiver: Option<Receiver<notify::Result<Event>>>,
    watched_files: HashSet<PathBuf>,
    last_change_times: std::collections::HashMap<PathBuf, SystemTime>,
    debounce_duration: Duration,
}

impl HotReloadManager {
    pub fn new() -> Self {
        Self {
            watcher: None,
            receiver: None,
            watched_files: HashSet::new(),
            last_change_times: std::collections::HashMap::new(),
            debounce_duration: Duration::from_millis(100),
        }
    }

    pub fn with_debounce_duration(mut self, duration: Duration) -> Self {
        self.debounce_duration = duration;
        self
    }

    /// Start watching a file or directory for changes
    pub fn start_watching(&mut self, path: impl AsRef<Path>) -> GPMLResult<()> {
        let path = path.as_ref();
        tracing::info!("HotReloadManager: Starting to watch path: {:?}", path);
        
        if !path.exists() {
            tracing::error!("HotReloadManager: Path does not exist: {:?}", path);
            return Err(GPMLError::FileNotFound {
                path: path.display().to_string(),
            });
        }
        
        if self.watcher.is_none() {
            tracing::info!("HotReloadManager: Creating new file watcher");
            let (sender, receiver) = mpsc::channel();
            let mut watcher = notify::recommended_watcher(sender)
                .map_err(|e| GPMLError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create file watcher: {}", e)
                )))?;

            watcher.watch(path, RecursiveMode::Recursive)
                .map_err(|e| GPMLError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to watch path: {}", e)
                )))?;

            self.watcher = Some(watcher);
            self.receiver = Some(receiver);
            tracing::info!("HotReloadManager: File watcher created and configured");
        } else if let Some(ref mut watcher) = self.watcher {
            tracing::info!("HotReloadManager: Adding path to existing watcher");
            watcher.watch(path, RecursiveMode::Recursive)
                .map_err(|e| GPMLError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to watch path: {}", e)
                )))?;
        }

        self.add_watched_file(path);
        tracing::info!("HotReloadManager: Now watching {} files total", self.watched_files.len());
        Ok(())
    }

    /// Add a specific file to the watch list
    pub fn add_watched_file(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        
        if path.is_file() {
            self.watched_files.insert(path.to_path_buf());
        } else if path.is_dir() {
            // Add all .gpml files in the directory
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.extension().and_then(|s| s.to_str()) == Some("gpml") {
                        self.watched_files.insert(entry_path);
                    }
                }
            }
        }
    }

    /// Check for file changes and return the paths of changed files
    pub fn check_for_changes(&mut self) -> Vec<PathBuf> {
        let mut changed_files = Vec::new();

        if let Some(receiver) = self.receiver.take() {
            // Process all pending events
            let mut event_count = 0;
            while let Ok(event_result) = receiver.try_recv() {
                event_count += 1;
                if let Ok(event) = event_result {
                    tracing::debug!("HotReloadManager: Received file event: {:?}", event);
                    if let Some(changed_file) = self.process_event(event) {
                        if !changed_files.contains(&changed_file) {
                            tracing::info!("HotReloadManager: File changed: {:?}", changed_file);
                            changed_files.push(changed_file);
                        }
                    }
                } else {
                    tracing::warn!("HotReloadManager: File watcher error: {:?}", event_result);
                }
            }
            
            if event_count > 0 {
                tracing::debug!("HotReloadManager: Processed {} events, {} files changed", event_count, changed_files.len());
            }
            
            // Put the receiver back
            self.receiver = Some(receiver);
        }

        changed_files
    }

    fn process_event(&mut self, event: Event) -> Option<PathBuf> {
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                for path in event.paths {
                    tracing::debug!("HotReloadManager: Checking event path: {:?}", path);
                    if self.is_watched_file(&path) && self.should_process_change(&path) {
                        tracing::info!("HotReloadManager: Processing change for: {:?}", path);
                        self.last_change_times.insert(path.clone(), SystemTime::now());
                        return Some(path);
                    } else {
                        tracing::debug!("HotReloadManager: Ignoring change for: {:?} (not watched or too recent)", path);
                    }
                }
            }
            _ => {
                tracing::debug!("HotReloadManager: Ignoring event kind: {:?}", event.kind);
            }
        }
        None
    }

    fn is_watched_file(&self, path: &Path) -> bool {
        // Check if this is a GPML file
        let is_gpml = path.extension().and_then(|s| s.to_str()) == Some("gpml");
        
        if !is_gpml {
            tracing::debug!("HotReloadManager: Not a GPML file: {:?}", path);
            return false;
        }

        // Check if it's in our watch list or if we're watching its directory
        let is_directly_watched = self.watched_files.contains(path);
        let is_in_watched_dir = self.watched_files.iter().any(|watched| {
            watched.is_dir() && path.starts_with(watched)
        });
        
        let result = is_directly_watched || is_in_watched_dir;
        
        tracing::debug!("HotReloadManager: File {:?} watched: {} (direct: {}, in_dir: {})", 
            path, result, is_directly_watched, is_in_watched_dir);
            
        if result {
            tracing::debug!("HotReloadManager: Watched files: {:?}", self.watched_files);
        }
        
        result
    }

    fn should_process_change(&self, path: &Path) -> bool {
        if let Some(last_change) = self.last_change_times.get(path) {
            if let Ok(elapsed) = SystemTime::now().duration_since(*last_change) {
                elapsed >= self.debounce_duration
            } else {
                true
            }
        } else {
            true
        }
    }

    /// Stop watching all files
    pub fn stop_watching(&mut self) {
        self.watcher = None;
        self.receiver = None;
        self.watched_files.clear();
        self.last_change_times.clear();
    }

    /// Remove a file from the watch list
    pub fn remove_watched_file(&mut self, path: impl AsRef<Path>) {
        self.watched_files.remove(path.as_ref());
        self.last_change_times.remove(path.as_ref());
    }

    /// Get all watched files
    pub fn get_watched_files(&self) -> &HashSet<PathBuf> {
        &self.watched_files
    }

    /// Check if currently watching any files
    pub fn is_watching(&self) -> bool {
        self.watcher.is_some() && !self.watched_files.is_empty()
    }
}

impl Default for HotReloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for HotReloadManager {
    fn drop(&mut self) {
        self.stop_watching();
    }
}

/// File change notification for GPML files
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub timestamp: SystemTime,
    pub change_type: FileChangeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
}

impl FileChangeEvent {
    pub fn new(path: PathBuf, change_type: FileChangeType) -> Self {
        Self {
            path,
            timestamp: SystemTime::now(),
            change_type,
        }
    }
}

/// Async hot reload manager for use in async contexts
pub struct AsyncHotReloadManager {
    manager: HotReloadManager,
    change_sender: Option<Sender<FileChangeEvent>>,
}

impl AsyncHotReloadManager {
    pub fn new() -> Self {
        Self {
            manager: HotReloadManager::new(),
            change_sender: None,
        }
    }

    pub fn with_change_channel(&mut self) -> Receiver<FileChangeEvent> {
        let (sender, receiver) = mpsc::channel();
        self.change_sender = Some(sender);
        receiver
    }

    pub async fn start_watching(&mut self, path: impl AsRef<Path>) -> GPMLResult<()> {
        self.manager.start_watching(path)?;
        
        // Start background task to check for changes
        if let Some(sender) = &self.change_sender {
            let sender = sender.clone();
            let mut manager = std::mem::take(&mut self.manager);
            
            tokio::spawn(async move {
                loop {
                    let changes = manager.check_for_changes();
                    for changed_path in changes {
                        let event = FileChangeEvent::new(changed_path, FileChangeType::Modified);
                        if sender.send(event).is_err() {
                            break; // Channel closed
                        }
                    }
                    
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            });
        }

        Ok(())
    }
}

impl Default for AsyncHotReloadManager {
    fn default() -> Self {
        Self::new()
    }
}
