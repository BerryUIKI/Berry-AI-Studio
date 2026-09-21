//! Long-lived filesystem watcher with a durable, coalesced change journal.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use berry_domain::{FilesystemChange, Folder};
use berry_scan::{ScanStats, Scanner};
use berry_storage::Database;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

const EVENT_QUIET_PERIOD: Duration = Duration::from_millis(750);
const RETRY_DELAY: Duration = Duration::from_secs(5);
const JOURNAL_BATCH_SIZE: usize = 512;
const EVENT_STORM_DRAIN_LIMIT: usize = 1024;

#[derive(Debug, Clone)]
struct WatchRoot {
    folder_id: i64,
    path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherStatus {
    pub is_active: bool,
    pub watched_roots_count: usize,
    pub pending_journal_count: usize,
    pub last_reconcile_time: Option<i64>,
    pub last_error: Option<String>,
}

#[derive(Debug, Default)]
pub struct WatcherHealth {
    pub last_reconcile_time: Option<i64>,
    pub last_error: Option<String>,
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
    health: Arc<RwLock<WatcherHealth>>,
    db_path: PathBuf,
}

impl LibraryWatcher {
    pub fn new(app: AppHandle, db_path: PathBuf) -> Result<Self, String> {
        let roots = Arc::new(RwLock::new(Vec::new()));
        let worker_roots = Arc::clone(&roots);
        let health = Arc::new(RwLock::new(WatcherHealth::default()));
        let worker_health = Arc::clone(&health);
        let (sender, receiver) = mpsc::channel();
        let watcher = notify::recommended_watcher(move |event| {
            let _ = sender.send(event);
        })
        .map_err(|error| error.to_string())?;

        let worker_db_path = db_path.clone();
        std::thread::Builder::new()
            .name("berry-filesystem-watcher".to_string())
            .spawn(move || run_worker(app, worker_db_path, worker_roots, worker_health, receiver))
            .map_err(|error| error.to_string())?;

        Ok(Self {
            watcher,
            roots,
            health,
            db_path,
        })
    }

    pub fn get_status(&self) -> WatcherStatus {
        let watched_roots_count = self.roots.read().map(|r| r.len()).unwrap_or(0);
        let (last_reconcile_time, last_error) = self
            .health
            .read()
            .map(|h| (h.last_reconcile_time, h.last_error.clone()))
            .unwrap_or((None, None));
        let pending_journal_count = Database::connect(&self.db_path)
            .and_then(|db| db.list_filesystem_changes(10_000))
            .map(|c| c.len())
            .unwrap_or(0);
        WatcherStatus {
            is_active: true,
            watched_roots_count,
            pending_journal_count,
            last_reconcile_time,
            last_error,
        }
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
            if let Ok(mut health) = self.health.write() {
                health.last_error = Some(format!("Failed to watch {}: {error}", folder.path));
            }
            return Err(error.to_string());
        }
        if let Ok(mut health) = self.health.write() {
            health.last_error = None;
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
    health: Arc<RwLock<WatcherHealth>>,
    receiver: mpsc::Receiver<notify::Result<Event>>,
) {
    let journal = match Database::connect(&db_path) {
        Ok(database) => database,
        Err(error) => {
            eprintln!("filesystem watcher could not open the journal database: {error}");
            if let Ok(mut h) = health.write() {
                h.last_error = Some(format!("Could not open journal database: {error}"));
            }
            return;
        }
    };

    let mut next_flush = Instant::now();
    loop {
        let timeout = next_flush.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(timeout) {
            Ok(Ok(event)) => {
                // Mitigate event storms by batch-draining pending events in channel
                let mut batch_events = vec![event];
                while let Ok(Ok(more_event)) = receiver.try_recv() {
                    batch_events.push(more_event);
                    if batch_events.len() >= EVENT_STORM_DRAIN_LIMIT {
                        break;
                    }
                }

                let mut all_changes = Vec::new();
                for ev in batch_events {
                    let mut changes = event_to_changes(&ev, &roots);
                    all_changes.append(&mut changes);
                }

                if !all_changes.is_empty() {
                    if let Err(error) = journal.record_filesystem_changes(&all_changes) {
                        eprintln!("filesystem watcher could not persist changes: {error}");
                        if let Ok(mut h) = health.write() {
                            h.last_error = Some(format!("Journal persist error: {error}"));
                        }
                    }
                    next_flush = Instant::now() + EVENT_QUIET_PERIOD;
                }
            }
            Ok(Err(error)) => {
                eprintln!("filesystem watcher backend error: {error}");
                if let Ok(mut h) = health.write() {
                    h.last_error = Some(format!("Watcher backend error: {error}"));
                }
                next_flush = Instant::now() + RETRY_DELAY;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let (succeeded, last_err) = reconcile_journal(&app, &db_path, &roots, &journal);
                if let Ok(mut h) = health.write() {
                    let now_secs = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    h.last_reconcile_time = Some(now_secs);
                    if !succeeded {
                        h.last_error = last_err;
                    } else if h.last_error.is_some() {
                        h.last_error = None;
                    }
                }
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
) -> (bool, Option<String>) {
    let changes = match journal.list_filesystem_changes(JOURNAL_BATCH_SIZE) {
        Ok(changes) => changes,
        Err(error) => {
            let msg = format!("filesystem watcher could not read its journal: {error}");
            eprintln!("{msg}");
            return (false, Some(msg));
        }
    };
    if changes.is_empty() {
        return (true, None);
    }

    let roots = match roots.read() {
        Ok(roots) => roots.clone(),
        Err(_) => return (false, Some("watch roots lock poisoned".to_string())),
    };
    let mut by_folder: HashMap<i64, Vec<FilesystemChange>> = HashMap::new();
    for change in changes {
        by_folder.entry(change.folder_id).or_default().push(change);
    }

    let mut all_succeeded = true;
    let mut last_error = None;
    for (folder_id, folder_changes) in by_folder {
        let Some(root) = roots.iter().find(|root| root.folder_id == folder_id) else {
            // Startup registers roots immediately after creating the worker.
            // Keep durable events until that registration is visible. Events
            // for truly removed folders are deleted by the journal FK cascade.
            all_succeeded = false;
            last_error = Some(format!("folder root {folder_id} not registered yet"));
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
                    let msg = format!("filesystem watcher could not acknowledge changes: {error}");
                    eprintln!("{msg}");
                    last_error = Some(msg);
                    all_succeeded = false;
                } else {
                    let _ = app.emit(
                        "library-files-changed",
                        LibraryFilesChanged { folder_id, stats },
                    );
                }
            }
            Err(error) => {
                let msg = format!("filesystem watcher reconciliation failed: {error}");
                eprintln!("{msg}");
                last_error = Some(msg);
                all_succeeded = false;
            }
        }
    }
    (all_succeeded, last_error)
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

    #[test]
    fn tracks_watcher_status_and_health() {
        let health = WatcherHealth {
            last_reconcile_time: Some(1726000000),
            last_error: None,
        };
        assert_eq!(health.last_reconcile_time, Some(1726000000));
        assert!(health.last_error.is_none());

        let status = WatcherStatus {
            is_active: true,
            watched_roots_count: 3,
            pending_journal_count: 0,
            last_reconcile_time: Some(1726000000),
            last_error: None,
        };
        assert!(status.is_active);
        assert_eq!(status.watched_roots_count, 3);
    }

    #[test]
    fn benchmark_event_storm_coalescing() {
        let temp_dir =
            std::env::temp_dir().join(format!("berry_storm_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("storm.db");
        let db = Database::connect(&db_path).expect("open db");
        let folder = db.add_folder("/storm/root").expect("add folder");

        // Simulate 10,000 events with 500 distinct paths
        let mut changes = Vec::with_capacity(10_000);
        for i in 0..10_000 {
            let path_idx = i % 500;
            changes.push(FilesystemChange {
                folder_id: folder.id,
                path: format!("/storm/root/file_{path_idx}.png"),
                event_kind: if i % 2 == 0 {
                    "modify".to_string()
                } else {
                    "create".to_string()
                },
                observed_at: 1000 + i as i64,
            });
        }

        let start = Instant::now();
        // Insert in batches like our event-storm receiver
        for chunk in changes.chunks(1024) {
            db.record_filesystem_changes(chunk).expect("record changes");
        }
        let elapsed = start.elapsed();

        // 10k items inserted and coalesced in SQLite
        let stored = db.list_filesystem_changes(1000).expect("list changes");
        assert_eq!(
            stored.len(),
            500,
            "10,000 events must coalesce into 500 distinct paths"
        );
        assert!(
            elapsed.as_millis() < 2000,
            "Event storm processing took too long: {:?}",
            elapsed
        );

        drop(db);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
