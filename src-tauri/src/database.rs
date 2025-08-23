use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::{Connection, params, OptionalExtension};
use chrono::Utc;

use crate::error::{NexusError, Result};
use crate::models::{
    Schema, Permissions, AppObject
};

#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<Connection>>,
    vault_path: PathBuf,
}

impl Database {
    pub async fn new(vault_path: &Path) -> Result<Self> {
        let nexus_dir = vault_path.join(".nexus");
        tokio::fs::create_dir_all(&nexus_dir).await?;
        
        let db_path = nexus_dir.join("vault.sqlite");
        let connection = Connection::open(&db_path)?;
        
        let db = Self {
            connection: Arc::new(Mutex::new(connection)),
            vault_path: vault_path.to_path_buf(),
        };
        
        db.initialize_schema().await?;
        db.register_core_schemas().await?;
        
        Ok(db)
    }

    async fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection.lock().await;
        
        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        
        // Create schemas table - registry for all data types
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schemas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                schema_name TEXT NOT NULL UNIQUE,
                definition_json TEXT NOT NULL,
                version TEXT NOT NULL DEFAULT '1.0.0',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )?;

        // Create data_objects table - central registry of all content
        conn.execute(
            "CREATE TABLE IF NOT EXISTS data_objects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                schema_id INTEGER NOT NULL,
                file_path TEXT UNIQUE,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (schema_id) REFERENCES schemas (id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create object_content table - stores the actual data as JSON
        conn.execute(
            "CREATE TABLE IF NOT EXISTS object_content (
                object_id INTEGER PRIMARY KEY,
                content_json TEXT NOT NULL,
                FOREIGN KEY (object_id) REFERENCES data_objects (id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create object_permissions table - granular sharing controls
        conn.execute(
            "CREATE TABLE IF NOT EXISTS object_permissions (
                object_id INTEGER PRIMARY KEY,
                share_with_ai BOOLEAN NOT NULL DEFAULT FALSE,
                share_with_cloud BOOLEAN NOT NULL DEFAULT FALSE,
                read_only BOOLEAN NOT NULL DEFAULT FALSE,
                expires_at TEXT,
                FOREIGN KEY (object_id) REFERENCES data_objects (id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indexes for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_data_objects_schema_id ON data_objects(schema_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_data_objects_file_path ON data_objects(file_path)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_data_objects_updated_at ON data_objects(updated_at)",
            [],
        )?;

        log::info!("Database schema initialized successfully");
        Ok(())
    }

    async fn register_core_schemas(&self) -> Result<()> {
        // Register the core Todo schema
        let todo_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "id": {"type": ["number", "null"]},
                "text": {"type": "string"},
                "completed": {"type": "boolean"},
                "created_at": {"type": "string", "format": "date-time"},
                "updated_at": {"type": ["string", "null"], "format": "date-time"},
                "due_date": {"type": ["string", "null"], "format": "date-time"},
                "priority": {"type": ["string", "null"], "enum": ["low", "medium", "high"]},
                "tags": {"type": ["array", "null"], "items": {"type": "string"}}
            },
            "required": ["text", "completed", "created_at"]
        });

        self.register_schema("core.todo", &todo_schema.to_string()).await?;
        
        log::info!("Core schemas registered successfully");
        Ok(())
    }

    pub async fn register_schema(&self, schema_name: &str, definition_json: &str) -> Result<i64> {
        let conn = self.connection.lock().await;
        
        // Validate JSON schema
        serde_json::from_str::<serde_json::Value>(definition_json)
            .map_err(|e| NexusError::InvalidSchema(e.to_string()))?;
        
        let now = Utc::now().to_rfc3339();
        
        match conn.execute(
            "INSERT OR REPLACE INTO schemas (schema_name, definition_json, created_at) 
             VALUES (?1, ?2, ?3)",
            params![schema_name, definition_json, now],
        ) {
            Ok(_) => {
                let schema_id = conn.last_insert_rowid();
                log::info!("Schema '{}' registered with ID: {}", schema_name, schema_id);
                Ok(schema_id)
            }
            Err(e) => Err(NexusError::Database(e))
        }
    }

    pub async fn get_schema_by_name(&self, schema_name: &str) -> Result<Option<Schema>> {
        let conn = self.connection.lock().await;
        
        let result = conn.query_row(
            "SELECT id, schema_name, definition_json, version, created_at FROM schemas WHERE schema_name = ?1",
            params![schema_name],
            |row| {
                Ok(Schema {
                    id: Some(row.get(0)?),
                    schema_name: row.get(1)?,
                    definition_json: row.get(2)?,
                    version: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        ).optional()?;
        
        Ok(result)
    }

    pub async fn save_object<T: serde::Serialize>(
        &self,
        schema_name: &str,
        content: &T,
        file_path: Option<&str>,
        permissions: Option<&Permissions>,
    ) -> Result<i64> {
        let conn = self.connection.lock().await;
        
        // Get schema ID
        let schema_id = match conn.query_row(
            "SELECT id FROM schemas WHERE schema_name = ?1",
            params![schema_name],
            |row| row.get::<_, i64>(0),
        ).optional()? {
            Some(id) => id,
            None => return Err(NexusError::SchemaNotFound(schema_name.to_string())),
        };

        let now = Utc::now().to_rfc3339();
        let content_json = serde_json::to_string(content)?;

        // Insert data object
        conn.execute(
            "INSERT INTO data_objects (schema_id, file_path, updated_at, created_at) 
             VALUES (?1, ?2, ?3, ?4)",
            params![schema_id, file_path, now, now],
        )?;
        
        let object_id = conn.last_insert_rowid();

        // Insert content
        conn.execute(
            "INSERT INTO object_content (object_id, content_json) VALUES (?1, ?2)",
            params![object_id, content_json],
        )?;

        // Insert permissions
        let default_perms = Permissions::default();
        let perms = permissions.unwrap_or(&default_perms);
        conn.execute(
            "INSERT INTO object_permissions 
             (object_id, share_with_ai, share_with_cloud, read_only, expires_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                object_id,
                perms.share_with_ai,
                perms.share_with_cloud,
                perms.read_only,
                perms.expires_at
            ],
        )?;

        log::info!("Object saved with ID: {} for schema: {}", object_id, schema_name);
        Ok(object_id)
    }

    pub async fn load_object<T>(&self, object_id: i64) -> Result<AppObject<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let conn = self.connection.lock().await;
        
        let result = conn.query_row(
            "SELECT 
                do.id, s.schema_name, oc.content_json, do.file_path, do.updated_at, do.created_at,
                op.share_with_ai, op.share_with_cloud, op.read_only, op.expires_at
             FROM data_objects do
             JOIN schemas s ON do.schema_id = s.id
             JOIN object_content oc ON do.id = oc.object_id
             JOIN object_permissions op ON do.id = op.object_id
             WHERE do.id = ?1",
            params![object_id],
            |row| {
                let content_json: String = row.get(2)?;
                let content: T = serde_json::from_str(&content_json)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(
                        2, 
                        format!("JSON deserialization error: {}", e).into(), 
                        rusqlite::types::Type::Text
                    ))?;

                Ok(AppObject {
                    id: row.get(0)?,
                    schema_name: row.get(1)?,
                    content,
                    file_path: row.get(3)?,
                    updated_at: row.get(4)?,
                    created_at: row.get(5)?,
                    permissions: Permissions {
                        share_with_ai: row.get(6)?,
                        share_with_cloud: row.get(7)?,
                        read_only: row.get(8)?,
                        expires_at: row.get(9)?,
                    },
                })
            },
        ).optional()?;

        result.ok_or(NexusError::ObjectNotFound(object_id))
    }

    pub async fn load_objects_by_schema<T>(&self, schema_name: &str) -> Result<Vec<AppObject<T>>>
    where
        T: serde::de::DeserializeOwned,
    {
        let conn = self.connection.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT 
                do.id, s.schema_name, oc.content_json, do.file_path, do.updated_at, do.created_at,
                op.share_with_ai, op.share_with_cloud, op.read_only, op.expires_at
             FROM data_objects do
             JOIN schemas s ON do.schema_id = s.id
             JOIN object_content oc ON do.id = oc.object_id
             JOIN object_permissions op ON do.id = op.object_id
             WHERE s.schema_name = ?1
             ORDER BY do.created_at DESC"
        )?;

        let rows = stmt.query_map(params![schema_name], |row| {
            let content_json: String = row.get(2)?;
            let content: T = serde_json::from_str(&content_json)
                .map_err(|e| rusqlite::Error::InvalidColumnType(
                    2, 
                    format!("JSON deserialization error: {}", e).into(), 
                    rusqlite::types::Type::Text
                ))?;

            Ok(AppObject {
                id: row.get(0)?,
                schema_name: row.get(1)?,
                content,
                file_path: row.get(3)?,
                updated_at: row.get(4)?,
                created_at: row.get(5)?,
                permissions: Permissions {
                    share_with_ai: row.get(6)?,
                    share_with_cloud: row.get(7)?,
                    read_only: row.get(8)?,
                    expires_at: row.get(9)?,
                },
            })
        })?;

        let mut objects = Vec::new();
        for row in rows {
            objects.push(row?);
        }

        Ok(objects)
    }

    pub async fn update_object_permissions(
        &self,
        object_id: i64,
        permissions: &Permissions,
    ) -> Result<()> {
        let conn = self.connection.lock().await;
        
        let updated = conn.execute(
            "UPDATE object_permissions 
             SET share_with_ai = ?1, share_with_cloud = ?2, read_only = ?3, expires_at = ?4
             WHERE object_id = ?5",
            params![
                permissions.share_with_ai,
                permissions.share_with_cloud,
                permissions.read_only,
                permissions.expires_at,
                object_id
            ],
        )?;

        if updated == 0 {
            return Err(NexusError::ObjectNotFound(object_id));
        }

        // Update the object's timestamp
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE data_objects SET updated_at = ?1 WHERE id = ?2",
            params![now, object_id],
        )?;

        log::info!("Permissions updated for object ID: {}", object_id);
        Ok(())
    }

    pub async fn delete_object(&self, object_id: i64) -> Result<()> {
        let conn = self.connection.lock().await;
        
        let deleted = conn.execute(
            "DELETE FROM data_objects WHERE id = ?1",
            params![object_id],
        )?;

        if deleted == 0 {
            return Err(NexusError::ObjectNotFound(object_id));
        }

        log::info!("Object deleted with ID: {}", object_id);
        Ok(())
    }

    pub async fn update_object_from_file_path(&self, file_path: &str) -> Result<Option<i64>> {
        let conn = self.connection.lock().await;
        
        // Find the object by file path
        let object_id: Option<i64> = conn.query_row(
            "SELECT id FROM data_objects WHERE file_path = ?1",
            params![file_path],
            |row| row.get(0),
        ).optional()?;

        if let Some(id) = object_id {
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE data_objects SET updated_at = ?1 WHERE id = ?2",
                params![now, id],
            )?;
            log::info!("Updated timestamp for object at path: {}", file_path);
        }

        Ok(object_id)
    }

    pub async fn get_sync_info(&self) -> Result<(usize, String)> {
        let conn = self.connection.lock().await;
        
        let count: usize = conn.query_row(
            "SELECT COUNT(*) FROM data_objects",
            [],
            |row| row.get::<_, i64>(0).map(|n| n as usize),
        )?;

        let last_updated: String = conn.query_row(
            "SELECT MAX(updated_at) FROM data_objects",
            [],
            |row| row.get::<_, Option<String>>(0).map(|opt| opt.unwrap_or_else(|| "Never".to_string())),
        )?;

        Ok((count, last_updated))
    }
}
