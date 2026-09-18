//! Long-lived filesystem watcher with a durable, coalesced change journal.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use berry_domain::{FilesystemChange, Folder};
use berry_scan::{ScanStats, Scanner};
use berry_storage::Database;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

const EVENT_QUIET_PERIOD: Duration = Duration::from_millis(750);
const RETRY_DELAY: Duration = Duration::from_secs(5);
const JOURNAL_BATCH_SIZE: usize = 512;

#[derive(Debug, Clone)]
struct WatchRoot {
    folder_id: i64,
    path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct LibraryFilesChanged {
    pub folder_id: i64,
    pub stats: ScanStats,
}

/// Owns the platform watcher. Dropping it closes the worker channel.
pub struct LibraryWatcher {
    watcher: RecommendedWatcher,
    roots: Arc<RwLock<Vec<WatchRoot>>>,
}

impl LibraryWatcher {
    pub fn new(app: AppHandle, db_path: PathBuf) -> Result<Self, String> {
        let roots = Arc::new(RwLock::new(Vec::new()));
        let worker_roots = Arc::clone(&roots);
        let (sender, receiver) = mpsc::channel();
        let watcher = notify::recommended_watcher(move |event| {
            let _ = sender.send(event);
        })
        .map_err(|error| error.to_string())?;

        std::thread::Builder::new()
            .name("berry-filesystem-watcher".to_string())
            .spawn(move || run_worker(app, db_path, worker_roots, receiver))
            .map_err(|error| error.to_string())?;

        Ok(Self { watcher, roots })
    }

    pub fn watch_folder(&mut self, folder: &Folder) -> Result<(), String> {
        let path = PathBuf::from(&folder.path);
        if !path.is_dir() {
            return Err(format!(
                "watch root is not an existing directory: {}",
                folder.path
            ));
        }
        {
            let mut roots = self
                .roots
                .write()
                .map_err(|_| "watch roots lock poisoned".to_string())?;
            roots.retain(|root| root.folder_id != folder.id);
            roots.push(WatchRoot {
                folder_id: folder.id,
                path: path.clone(),
            });
        }
        if let Err(error) = self.watcher.watch(&path, RecursiveMode::Recursive) {
            if let Ok(mut roots) = self.roots.write() {
                roots.retain(|root| root.folder_id != folder.id);
            }
            return Err(error.to_string());
        }
        Ok(())
    }

    pub fn unwatch_folder(&mut self, folder: &Folder) -> Result<(), String> {
        let path = PathBuf::from(&folder.path);
        let unwatch_result = self
            .watcher
            .unwatch(&path)
            .map_err(|error| error.to_string());
        self.roots
            .write()
            .map_err(|_| "watch roots lock poisoned".to_string())?
            .retain(|root| root.folder_id != folder.id);
        unwatch_result
    }
}

fn run_worker(
    app: AppHandle,
    db_path: PathBuf,
    roots: Arc<RwLock<Vec<WatchRoot>>>,
    receiver: mpsc::Receiver<notify::Result<Event>>,
) {
    let journal = match Database::connect(&db_path) {
        Ok(database) => database,
        Err(error) => {
            eprintln!("filesystem watcher could not open the journal database: {error}");
            return;
        }
    };

    let mut next_flush = Instant::now();
    loop {
        let timeout = next_flush.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(timeout) {
            Ok(Ok(event)) => {
                let changes = event_to_changes(&event, &roots);
                if !changes.is_empty() {
                    if let Err(error) = journal.record_filesystem_changes(&changes) {
                        eprintln!("filesystem watcher could not persist changes: {error}");
                    }
                    next_flush = Instant::now() + EVENT_QUIET_PERIOD;
                }
            }
            Ok(Err(error)) => {
                eprintln!("filesystem watcher backend error: {error}");
                next_flush = Instant::now() + RETRY_DELAY;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let succeeded = reconcile_journal(&app, &db_path, &roots, &journal);
                let has_pending = journal
                    .list_filesystem_changes(1)
                    .map(|changes| !changes.is_empty())
                    .unwrap_or(true);
                next_flush = Instant::now()
                    + if !succeeded {
                        RETRY_DELAY
                    } else if has_pending {
                        Duration::from_millis(10)
                    } else {
                        Duration::from_secs(60)
                    };
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = reconcile_journal(&app, &db_path, &roots, &journal);
                break;
            }
        }
    }
}

fn event_to_changes(event: &Event, roots: &Arc<RwLock<Vec<WatchRoot>>>) -> Vec<FilesystemChange> {
    if matches!(event.kind, EventKind::Access(_)) {
        return Vec::new();
    }
    let observed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let event_kind = match event.kind {
        EventKind::Create(_) => "create",
        EventKind::Modify(_) => "modify",
        EventKind::Remove(_) => "remove",
        EventKind::Any => "any",
        EventKind::Other => "other",
        EventKind::Access(_) => return Vec::new(),
    };
    let roots = match roots.read() {
        Ok(roots) => roots,
        Err(_) => return Vec::new(),
    };

    if event.need_rescan() {
        return roots
            .iter()
            .map(|root| FilesystemChange {
                folder_id: root.folder_id,
                path: root.path.to_string_lossy().to_string(),
                event_kind: "rescan".to_string(),
                observed_at,
            })
            .collect();
    }

    event
        .paths
        .iter()
        .filter_map(|path| {
            matching_root(&roots, path).map(|root| FilesystemChange {
                folder_id: root.folder_id,
                path: path.to_string_lossy().to_string(),
                event_kind: event_kind.to_string(),
                observed_at,
            })
        })
        .collect()
}

fn matching_root<'a>(roots: &'a [WatchRoot], path: &Path) -> Option<&'a WatchRoot> {
    roots
        .iter()
        .filter(|root| path.starts_with(&root.path))
        .max_by_key(|root| root.path.components().count())
}

fn reconcile_journal(
    app: &AppHandle,
    db_path: &Path,
    roots: &Arc<RwLock<Vec<WatchRoot>>>,
    journal: &Database,
) -> bool {
    let changes = match journal.list_filesystem_changes(JOURNAL_BATCH_SIZE) {
        Ok(changes) => changes,
        Err(error) => {
            eprintln!("filesystem watcher could not read its journal: {error}");
            return false;
        }
    };
    if changes.is_empty() {
        return true;
    }

    let roots = match roots.read() {
        Ok(roots) => roots.clone(),
        Err(_) => return false,
    };
    let mut by_folder: HashMap<i64, Vec<FilesystemChange>> = HashMap::new();
    for change in changes {
        by_folder.entry(change.folder_id).or_default().push(change);
    }

    let mut all_succeeded = true;
    for (folder_id, folder_changes) in by_folder {
        let Some(root) = roots.iter().find(|root| root.folder_id == folder_id) else {
            // Startup registers roots immediately after creating the worker.
            // Keep durable events until that registration is visible. Events
            // for truly removed folders are deleted by the journal FK cascade.
            all_succeeded = false;
            continue;
        };
        let paths = folder_changes
            .iter()
            .map(|change| PathBuf::from(&change.path))
            .collect::<Vec<_>>();
        let scanner = Scanner::with_default_extractor(db_path.to_path_buf());
        match scanner.reconcile_paths(folder_id, &root.path, &paths, |progress| {
            let _ = app.emit("scan-progress", progress);
        }) {
            Ok(stats) => {
                if let Err(error) = journal.delete_filesystem_changes(&folder_changes) {
                    eprintln!("filesystem watcher could not acknowledge changes: {error}");
                    all_succeeded = false;
                } else {
                    let _ = app.emit(
                        "library-files-changed",
                        LibraryFilesChanged { folder_id, stats },
                    );
                }
            }
            Err(error) => {
                eprintln!("filesystem watcher reconciliation failed: {error}");
                all_succeeded = false;
            }
        }
    }
    all_succeeded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_root_prefers_the_most_specific_registered_path() {
        let roots = vec![
            WatchRoot {
                folder_id: 1,
                path: PathBuf::from("/library"),
            },
            WatchRoot {
                folder_id: 2,
                path: PathBuf::from("/library/project"),
            },
        ];
        assert_eq!(
            matching_root(&roots, Path::new("/library/project/image.png"))
                .map(|root| root.folder_id),
            Some(2)
        );
    }
}
