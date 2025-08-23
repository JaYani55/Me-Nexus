use thiserror::Error;

#[derive(Error, Debug)]
pub enum NexusError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("File system watcher error: {0}")]
    Notify(#[from] notify::Error),
    
    #[error("Vault not configured")]
    VaultNotConfigured,
    
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),
    
    #[error("Object not found: {0}")]
    ObjectNotFound(i64),
    
    #[error("Invalid schema definition: {0}")]
    InvalidSchema(String),
    
    #[error("Sync error: {0}")]
    Sync(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub type Result<T> = std::result::Result<T, NexusError>;

impl From<NexusError> for String {
    fn from(error: NexusError) -> Self {
        error.to_string()
    }
}
