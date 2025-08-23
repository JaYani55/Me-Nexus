use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use notify::{RecommendedWatcher, Watcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, FileIdMap};

use crate::error::{NexusError, Result};
use crate::database::Database;
use crate::models::{SyncStatus, Todo};

pub struct SyncService {
    database: Arc<Database>,
    vault_path: PathBuf,
    status: Arc<RwLock<SyncStatus>>,
    _watcher: Option<Debouncer<RecommendedWatcher, FileIdMap>>,
}

impl SyncService {
    pub async fn new(database: Arc<Database>, vault_path: &Path) -> Result<Self> {
        let status = Arc::new(RwLock::new(SyncStatus {
            is_syncing: false,
            last_sync: None,
            pending_changes: 0,
            errors: Vec::new(),
        }));

        let service = Self {
            database,
            vault_path: vault_path.to_path_buf(),
            status,
            _watcher: None,
        };

        Ok(service)
    }

    pub async fn start(&mut self) -> Result<()> {
        log::info!("Starting sync service for vault: {:?}", self.vault_path);

        // Perform initial scan
        self.perform_initial_scan().await?;

        // Set up file watcher
        let (tx, mut rx) = mpsc::channel(100);
        let database = Arc::clone(&self.database);
        let status = Arc::clone(&self.status);
        let vault_path = self.vault_path.clone();

        let mut debouncer = new_debouncer(
            Duration::from_millis(250),
            None,
            move |result: notify_debouncer_full::DebounceEventResult| {
                let tx = tx.clone();
                tokio::spawn(async move {
                    match result {
                        Ok(events) => {
                            for event in events {
                                if let Err(e) = tx.send(event).await {
                                    log::error!("Failed to send file event: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("File watcher error: {:?}", e);
                        }
                    }
                });
            },
        ).map_err(NexusError::from)?;

        // Watch the vault directory recursively
        debouncer
            .watcher()
            .watch(&self.vault_path, notify::RecursiveMode::Recursive)
            .map_err(NexusError::from)?;

        self._watcher = Some(debouncer);

        // Spawn background task to handle file events
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Err(e) = Self::handle_file_event(&database, &status, &vault_path, event).await {
                    log::error!("Error handling file event: {}", e);
                    let mut status_guard = status.write().await;
                    status_guard.errors.push(e.to_string());
                }
            }
        });

        log::info!("Sync service started successfully");
        Ok(())
    }

    async fn perform_initial_scan(&self) -> Result<()> {
        log::info!("Performing initial vault scan...");
        
        let mut status = self.status.write().await;
        status.is_syncing = true;
        status.pending_changes = 0;
        status.errors.clear();
        drop(status);

        // Scan for todos
        let todos_path = self.vault_path.join("Todo").join("todos.json");
        if todos_path.exists() {
            if let Err(e) = self.sync_todos_file(&todos_path).await {
                log::error!("Failed to sync todos file during initial scan: {}", e);
                let mut status = self.status.write().await;
                status.errors.push(format!("Initial todos sync failed: {}", e));
            }
        }

        // Update status
        let mut status = self.status.write().await;
        status.is_syncing = false;
        status.last_sync = Some(chrono::Utc::now().to_rfc3339());
        
        log::info!("Initial vault scan completed");
        Ok(())
    }

    async fn handle_file_event(
        database: &Arc<Database>,
        status: &Arc<RwLock<SyncStatus>>,
        vault_path: &Path,
        event: DebouncedEvent,
    ) -> Result<()> {
        use notify::EventKind;

        let mut status_guard = status.write().await;
        status_guard.is_syncing = true;
        status_guard.pending_changes += 1;
        drop(status_guard);

        for path in &event.paths {
            // Skip .nexus directory to avoid infinite loops
            if path.starts_with(vault_path.join(".nexus")) {
                continue;
            }

            // Skip temporary files and hidden files
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with('.') || file_name.starts_with('~') || file_name.ends_with(".tmp") {
                    continue;
                }
            }

            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        Self::handle_json_file_change(database, path).await?;
                    }
                }
                EventKind::Remove(_) => {
                    Self::handle_file_deletion(database, path).await?;
                }
                _ => {}
            }
        }

        let mut status_guard = status.write().await;
        status_guard.is_syncing = false;
        status_guard.pending_changes = status_guard.pending_changes.saturating_sub(1);
        status_guard.last_sync = Some(chrono::Utc::now().to_rfc3339());

        Ok(())
    }

    async fn handle_json_file_change(database: &Arc<Database>, file_path: &Path) -> Result<()> {
        let path_str = file_path.to_string_lossy().to_string();
        log::info!("Handling JSON file change: {}", path_str);

        // Check if this is a todos file
        if file_path.file_name().and_then(|n| n.to_str()) == Some("todos.json") {
            Self::sync_todos_file_from_db(database, file_path).await?;
        }

        // Update the database timestamp for this file
        database.update_object_from_file_path(&path_str).await?;

        Ok(())
    }

    async fn handle_file_deletion(database: &Arc<Database>, file_path: &Path) -> Result<()> {
        let path_str = file_path.to_string_lossy().to_string();
        log::info!("Handling file deletion: {}", path_str);

        // For now, we'll just log the deletion
        // In a full implementation, we might mark objects as deleted or remove them
        database.update_object_from_file_path(&path_str).await?;

        Ok(())
    }

    async fn sync_todos_file(&self, todos_path: &Path) -> Result<()> {
        if !todos_path.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(todos_path).await?;
        let todo_list: serde_json::Value = serde_json::from_str(&content)?;
        
        if let Some(todos_array) = todo_list.get("todos").and_then(|v| v.as_array()) {
            for todo_value in todos_array {
                let todo: Todo = serde_json::from_value(todo_value.clone())?;
                
                // Save to database
                self.database.save_object(
                    "core.todo",
                    &todo,
                    Some(&todos_path.to_string_lossy()),
                    None,
                ).await?;
            }
        }

        log::info!("Synced todos file: {:?}", todos_path);
        Ok(())
    }

    async fn sync_todos_file_from_db(database: &Arc<Database>, _file_path: &Path) -> Result<()> {
        // Load todos from database
        let todos: Vec<crate::models::AppObject<Todo>> = database
            .load_objects_by_schema("core.todo")
            .await?;

        log::info!("Loaded {} todos from database for sync", todos.len());
        
        // In a full implementation, we would update the file here
        // For now, we just log the sync operation
        
        Ok(())
    }

    pub async fn get_status(&self) -> SyncStatus {
        self.status.read().await.clone()
    }

    pub async fn force_sync(&self) -> Result<()> {
        log::info!("Force sync requested");
        self.perform_initial_scan().await
    }

    pub async fn get_vault_stats(&self) -> Result<(usize, String)> {
        self.database.get_sync_info().await
    }
}

// Helper function for manual sync operations
pub async fn sync_vault_to_database(_database: &Database, vault_path: &Path) -> Result<()> {
    log::info!("Performing manual vault to database sync");
    
    let database_arc = Arc::new(
        Database::new(vault_path).await?
    );
    let sync_service = SyncService::new(database_arc, vault_path).await?;
    sync_service.perform_initial_scan().await?;
    
    Ok(())
}

impl Clone for SyncStatus {
    fn clone(&self) -> Self {
        Self {
            is_syncing: self.is_syncing,
            last_sync: self.last_sync.clone(),
            pending_changes: self.pending_changes,
            errors: self.errors.clone(),
        }
    }
}
