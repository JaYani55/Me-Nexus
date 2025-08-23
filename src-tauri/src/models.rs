use serde::{Deserialize, Serialize};
use chrono::Utc;

// Core data structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Todo {
    pub id: Option<u32>,
    pub text: String,
    pub completed: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub due_date: Option<String>,
    pub priority: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Permissions {
    pub share_with_ai: bool,
    pub share_with_cloud: bool,
    pub read_only: bool,
    pub expires_at: Option<String>,
}

// Generic container for any object type with metadata
#[derive(Debug, Serialize)]
pub struct AppObject<T> {
    pub id: i64,
    pub schema_name: String,
    pub content: T,
    pub permissions: Permissions,
    pub file_path: Option<String>,
    pub updated_at: String,
    pub created_at: String,
}

// Database schema structures
#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    pub id: Option<i64>,
    pub schema_name: String,
    pub definition_json: String,
    pub version: String,
    pub created_at: String,
}

#[derive(Debug)]
pub struct DataObject {
    pub id: Option<i64>,
    pub schema_id: i64,
    pub file_path: Option<String>,
    pub updated_at: String,
    pub created_at: String,
}

#[derive(Debug)]
pub struct ObjectContent {
    pub object_id: i64,
    pub content_json: String,
}

#[derive(Debug)]
pub struct ObjectPermissions {
    pub object_id: i64,
    pub permissions: Permissions,
}

// Vault configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultConfig {
    pub vault_path: String,
    pub created_at: String,
    pub version: String,
    pub encryption_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultInfo {
    pub path: String,
    pub is_empty: bool,
    pub exists: bool,
    pub has_nexus_folder: bool,
    pub database_exists: bool,
}

// Plugin system structures
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub schemas: Vec<PluginSchema>,
    pub permissions: Vec<String>,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginSchema {
    pub name: String,
    pub version: String,
    pub definition: serde_json::Value,
    pub file_extensions: Vec<String>,
}

// Sync service structures
#[derive(Debug)]
pub enum SyncEvent {
    FileCreated(String),
    FileModified(String),
    FileDeleted(String),
    DirectoryCreated(String),
    DirectoryDeleted(String),
}

#[derive(Debug, Serialize)]
pub struct SyncStatus {
    pub is_syncing: bool,
    pub last_sync: Option<String>,
    pub pending_changes: usize,
    pub errors: Vec<String>,
}

// Helper implementations
impl Todo {
    pub fn new(text: String) -> Self {
        Self {
            id: None,
            text,
            completed: false,
            created_at: Utc::now().to_rfc3339(),
            updated_at: None,
            due_date: None,
            priority: None,
            tags: None,
        }
    }

    pub fn mark_updated(&mut self) {
        self.updated_at = Some(Utc::now().to_rfc3339());
    }
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            vault_path: String::new(),
            created_at: Utc::now().to_rfc3339(),
            version: "1.0.0".to_string(),
            encryption_enabled: false,
        }
    }
}

impl Schema {
    pub fn new(schema_name: String, definition_json: String) -> Self {
        Self {
            id: None,
            schema_name,
            definition_json,
            version: "1.0.0".to_string(),
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

impl DataObject {
    pub fn new(schema_id: i64, file_path: Option<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: None,
            schema_id,
            file_path,
            updated_at: now.clone(),
            created_at: now,
        }
    }
}
