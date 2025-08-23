# Me-Nexus Local Database & Sync System

## Overview

Me-Nexus implements a sophisticated **dual-storage architecture** that combines the benefits of file-based storage with the performance of a structured database. This system provides users with complete data sovereignty while enabling powerful features like real-time synchronization, granular permissions, and extensible plugin architecture.

## Architecture Philosophy

### Local-First Approach

The system is built on the principle of **local-first computing**:
- All data resides on the user's machine by default
- Human-readable files serve as the source of truth
- No vendor lock-in - users can access their data with any tool
- Optional sharing requires explicit user consent

### Dual-Storage Model

```
┌─────────────────┐    Real-time Sync    ┌──────────────────┐
│  File System    │ ←──────────────────→ │  SQLite Database │
│  (Source Truth) │                      │  (Performance)   │
├─────────────────┤                      ├──────────────────┤
│ • todos.json    │                      │ • Structured     │
│ • notes.md      │                      │ • Indexed        │
│ • calendar.json │                      │ • Queryable      │
│ • ...           │                      │ • Permissions    │
└─────────────────┘                      └──────────────────┘
```

**Benefits**:
1. **User Sovereignty**: Files remain human-readable and portable
2. **Performance**: Database enables fast queries and complex operations
3. **Offline-First**: No internet dependency for core functionality
4. **Privacy**: Granular control over what data can be shared

---

## Database Schema

### Core Tables

The database schema is designed for extensibility, allowing any plugin to register custom data types without modifying the core schema.

#### 1. Schemas Table
```sql
CREATE TABLE IF NOT EXISTS schemas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schema_name TEXT NOT NULL UNIQUE,  -- e.g., 'core.todo', 'plugin.journal'
    definition_json TEXT NOT NULL,     -- JSON Schema for validation
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);
```

**Purpose**: Registry of all data types in the system
- Core app registers schemas like `core.todo`
- Plugins register their own schemas like `plugin.calendar.event`
- JSON Schema definitions enable validation and UI generation

#### 2. Data Objects Table
```sql
CREATE TABLE IF NOT EXISTS data_objects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schema_id INTEGER NOT NULL,
    file_path TEXT UNIQUE,             -- Links to actual file
    updated_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (schema_id) REFERENCES schemas (id)
);
```

**Purpose**: Central registry of all data objects
- Every piece of data gets an entry here
- Links structured data to physical files
- Enables cross-plugin queries and relationships

#### 3. Object Content Table
```sql
CREATE TABLE IF NOT EXISTS object_content (
    object_id INTEGER PRIMARY KEY,
    content_json TEXT NOT NULL,
    FOREIGN KEY (object_id) REFERENCES data_objects (id) ON DELETE CASCADE
);
```

**Purpose**: Stores the actual data as JSON
- Generic storage works with any data structure
- Fast queries without file I/O
- Maintains referential integrity

#### 4. Object Permissions Table
```sql
CREATE TABLE IF NOT EXISTS object_permissions (
    object_id INTEGER PRIMARY KEY,
    share_with_ai BOOLEAN NOT NULL DEFAULT FALSE,
    share_with_cloud BOOLEAN NOT NULL DEFAULT FALSE,
    read_only BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at TEXT NULL,              -- Optional expiration for sharing
    FOREIGN KEY (object_id) REFERENCES data_objects (id) ON DELETE CASCADE
);
```

**Purpose**: Granular permission control
- **Zero-trust model**: Nothing shared by default
- User explicitly controls each sharing decision
- Future-proof for additional permission types

---

## Synchronization System

### File System Watching

The sync service uses the `notify` crate to monitor file changes in real-time:

```rust
use notify::{RecommendedWatcher, Watcher, Event, EventKind};
use notify_debouncer_full::{new_debouncer, DebounceEventResult};
```

#### Key Components

1. **File Watcher**: Monitors vault directory for changes
2. **Debouncer**: Prevents duplicate events during rapid file saves
3. **Event Handler**: Processes file changes and updates database
4. **Bidirectional Sync**: Changes flow both ways (file ↔ database)

#### Sync Flow

```
File Change Detected
        ↓
Event Debounced (300ms)
        ↓
Parse File Content
        ↓
Validate Against Schema
        ↓
Update Database
        ↓
Emit UI Update Event
```

### Conflict Resolution

The system uses **timestamp-based conflict resolution**:

1. **File Modified**: If file timestamp > database timestamp, file wins
2. **Database Modified**: If database timestamp > file timestamp, database wins
3. **Simultaneous Changes**: File takes precedence (preserves user intent)

### Performance Optimizations

- **Incremental Sync**: Only processes changed files
- **Batch Operations**: Groups multiple changes into single transaction
- **Connection Pooling**: Reuses database connections
- **Async Processing**: Non-blocking operations prevent UI freezing

---

## Permission System

### Zero-Trust Security Model

The permission system implements a **zero-trust approach**:
- No data is shared by default
- Every sharing decision requires explicit user consent
- Permissions are granular and object-specific
- Users can revoke permissions at any time

### Permission Types

#### 1. AI Sharing (`share_with_ai`)
- **Purpose**: Allow AI services to read and process this data
- **Use Cases**: Content analysis, summarization, intelligent suggestions
- **Default**: `FALSE`
- **UI**: 🧠 AI toggle in permission controls

#### 2. Cloud Sharing (`share_with_cloud`)
- **Purpose**: Enable synchronization to cloud services
- **Use Cases**: Backup, device sync, collaborative features
- **Default**: `FALSE`
- **UI**: ☁️ Sync toggle in permission controls

#### 3. Read-Only Mode (`read_only`)
- **Purpose**: Prevent accidental modification of important data
- **Use Cases**: Archive mode, protecting critical information
- **Default**: `FALSE`
- **UI**: 🔒 Lock toggle in permission controls

#### 4. Expiration (`expires_at`)
- **Purpose**: Automatically revoke sharing after specified time
- **Use Cases**: Temporary collaboration, time-limited AI access
- **Default**: `NULL` (no expiration)
- **UI**: Calendar picker for expiration date

### Permission Management UI

The permission system provides multiple interfaces for user control:

#### Settings Modal Interface
```svelte
<!-- Permissions tab in settings -->
<div class="permissions-tab">
  <!-- Statistics Dashboard -->
  <div class="permission-stats">
    <div class="stat-card">
      <span class="stat-number">{totalObjects}</span>
      <span class="stat-label">Total Objects</span>
    </div>
    <div class="stat-card">
      <span class="stat-number">{aiSharedCount}</span>
      <span class="stat-label">AI Shared</span>
    </div>
    <!-- ... more stats -->
  </div>

  <!-- Object Grid -->
  <div class="objects-grid">
    {#each vaultObjects as object}
      <div class="object-card">
        <div class="object-info">
          <h4>{object.schema_name}</h4>
          <p>ID: {object.id}</p>
          <small>{formatDate(object.created_at)}</small>
        </div>
        <PermissionControls bind:permissions={object.permissions} />
      </div>
    {/each}
  </div>
</div>
```

#### Inline Permission Controls
```svelte
<!-- Reusable component for any data object -->
<div class="permission-controls">
  <label title="Allow AI services to read and process this item">
    🧠 AI
    <input 
      type="checkbox" 
      bind:checked={permissions.share_with_ai} 
      on:change={updatePermissions}
    />
  </label>
  
  <label title="Allow this item to be synced to your cloud account">
    ☁️ Sync
    <input 
      type="checkbox" 
      bind:checked={permissions.share_with_cloud} 
      on:change={updatePermissions}
    />
  </label>
  
  <label title="Prevent accidental modification of this item">
    🔒 Read Only
    <input 
      type="checkbox" 
      bind:checked={permissions.read_only} 
      on:change={updatePermissions}
    />
  </label>
</div>
```

---

## Plugin Architecture

### Schema Registration

Plugins extend the system by registering custom schemas:

```rust
// Example plugin schema registration
let journal_schema = Schema {
    schema_name: "plugin.journal.entry".to_string(),
    definition_json: r#"{
        "type": "object",
        "properties": {
            "title": {"type": "string"},
            "content": {"type": "string"},
            "mood": {"type": "string", "enum": ["happy", "sad", "neutral"]},
            "tags": {"type": "array", "items": {"type": "string"}},
            "created_at": {"type": "string", "format": "date-time"}
        },
        "required": ["title", "content", "created_at"]
    }"#.to_string(),
    version: 1,
    created_at: Utc::now().to_rfc3339(),
};

database.register_schema(journal_schema).await?;
```

### Plugin Data Flow

```
Plugin Creates Data
        ↓
Validates Against Schema
        ↓
Stores in Generic Tables
        ↓
Files Written to Vault
        ↓
Available in Core Queries
        ↓
Inherits Permission System
```

### Benefits of Generic Architecture

1. **No Schema Changes**: Core database schema never needs modification
2. **Cross-Plugin Queries**: Search across all data types
3. **Unified Permissions**: Same permission model for all plugins
4. **Automatic UI**: Permission controls work for any plugin data

---

## API Reference

### Tauri Commands

#### Vault Management
```rust
#[tauri::command]
async fn get_vault_config() -> Result<Option<VaultConfig>, String>

#[tauri::command]
async fn set_vault_path(path: String) -> Result<VaultConfig, String>

#[tauri::command]
async fn check_directory_info(path: String) -> Result<VaultInfo, String>
```

#### Object Operations
```rust
#[tauri::command]
async fn get_all_vault_objects() -> Result<Vec<VaultObject>, String>

#[tauri::command]
async fn save_object(schema_name: String, content: serde_json::Value) -> Result<i64, String>

#[tauri::command]
async fn delete_object(object_id: i64) -> Result<(), String>
```

#### Permission Management
```rust
#[tauri::command]
async fn update_object_permissions(
    object_id: i64, 
    permissions: Permissions
) -> Result<(), String>

#[tauri::command]
async fn get_permission_stats() -> Result<PermissionStats, String>
```

#### Synchronization
```rust
#[tauri::command]
async fn get_sync_status() -> Result<SyncStatus, String>

#[tauri::command]
async fn force_sync() -> Result<(), String>
```

### Data Models

#### Core Types
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultConfig {
    pub vault_path: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Permissions {
    pub share_with_ai: bool,
    pub share_with_cloud: bool,
    pub read_only: bool,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VaultObject {
    pub id: i64,
    pub schema_name: String,
    pub file_path: Option<String>,
    pub permissions: Permissions,
    pub created_at: String,
    pub updated_at: String,
}
```

#### Generic Container
```rust
#[derive(Debug, Serialize)]
pub struct AppObject<T> {
    pub id: i64,
    pub schema_name: String,
    pub content: T,
    pub permissions: Permissions,
    pub updated_at: String,
}
```

---

## File System Layout

### Vault Structure
```
<vault_path>/
├── .nexus/                    # Hidden system directory
│   ├── vault.sqlite          # Local database
│   ├── config.json           # Vault configuration
│   └── logs/                 # System logs
├── todos.json                # Core todo data
├── notes/                    # User notes directory
│   ├── daily-notes/
│   └── project-notes/
├── calendar/                 # Calendar plugin data
│   └── events.json
└── plugins/                  # Plugin-specific directories
    ├── journal/
    └── habits/
```

### File Formats

#### Todo File Example (`todos.json`)
```json
{
  "schema": "core.todo",
  "version": "1.0",
  "data": [
    {
      "id": 1,
      "text": "Complete project documentation",
      "completed": false,
      "created_at": "2025-01-15T10:30:00Z",
      "updated_at": "2025-01-15T10:30:00Z"
    }
  ]
}
```

#### Permission File Example (`.nexus/permissions.json`)
```json
{
  "object_permissions": {
    "1": {
      "share_with_ai": false,
      "share_with_cloud": true,
      "read_only": false,
      "expires_at": null
    }
  }
}
```

---

## Error Handling

### Error Types
```rust
#[derive(Error, Debug)]
pub enum NexusError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Sync error: {0}")]
    Sync(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),
}
```

### Error Recovery

The system implements several error recovery mechanisms:

1. **Database Corruption**: Automatic backup restoration
2. **File Conflicts**: Timestamp-based resolution with user notification
3. **Permission Errors**: Graceful degradation with user prompts
4. **Sync Failures**: Retry with exponential backoff

---

## Performance Considerations

### Database Optimizations

#### Indexing Strategy
```sql
-- Core indexes for performance
CREATE INDEX IF NOT EXISTS idx_data_objects_schema_id ON data_objects(schema_id);
CREATE INDEX IF NOT EXISTS idx_data_objects_updated_at ON data_objects(updated_at);
CREATE INDEX IF NOT EXISTS idx_data_objects_file_path ON data_objects(file_path);

-- Permission-based queries
CREATE INDEX IF NOT EXISTS idx_permissions_ai ON object_permissions(share_with_ai);
CREATE INDEX IF NOT EXISTS idx_permissions_cloud ON object_permissions(share_with_cloud);
```

#### Connection Management
- **Connection Pooling**: Single shared connection with Arc<Mutex<>>
- **Async Operations**: Non-blocking database access with tokio-rusqlite
- **Transaction Batching**: Group related operations for consistency

### File System Optimizations

#### Debouncing Strategy
```rust
// Prevent excessive sync operations during rapid file changes
let debouncer = new_debouncer(
    Duration::from_millis(300),  // 300ms debounce
    None,                        // No custom tick rate
    move |result: DebounceEventResult| {
        // Process debounced events
    }
)?;
```

#### Selective Watching
- Only monitor vault directory and subdirectories
- Ignore system files and temporary files
- Filter events by file extension and path patterns

---

## Security Considerations

### Data Privacy

1. **Local Storage**: All data stays on user's machine by default
2. **Encryption**: Future support for at-rest encryption
3. **Access Control**: File system permissions protect vault
4. **Audit Trail**: Logging of permission changes and data access

### Permission Enforcement

```rust
// Example permission check before AI access
async fn check_ai_permission(object_id: i64) -> Result<bool, NexusError> {
    let permissions = database.get_object_permissions(object_id).await?;
    
    // Check if AI sharing is enabled
    if !permissions.share_with_ai {
        return Err(NexusError::PermissionDenied(
            "AI access not permitted for this object".to_string()
        ));
    }
    
    // Check expiration
    if let Some(expires_at) = permissions.expires_at {
        let expiry = DateTime::parse_from_rfc3339(&expires_at)?;
        if Utc::now() > expiry {
            return Err(NexusError::PermissionDenied(
                "AI access permission has expired".to_string()
            ));
        }
    }
    
    Ok(true)
}
```

### Future Security Enhancements

1. **End-to-End Encryption**: For cloud sync
2. **Digital Signatures**: Verify data integrity
3. **Access Tokens**: Fine-grained API access control
4. **Audit Logging**: Track all data access and modifications

---

## Troubleshooting

### Common Issues

#### 1. Database Lock Errors
**Symptom**: "Database is locked" errors
**Cause**: Multiple processes accessing database simultaneously
**Solution**: 
```rust
// Use connection with timeout
let mut conn = Connection::open_with_flags(
    db_path,
    OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
)?;
conn.busy_timeout(Duration::from_secs(30))?;
```

#### 2. File Sync Delays
**Symptom**: Changes in files don't appear in UI immediately
**Cause**: File watcher debouncing or processing delays
**Solution**: Check debounce timing and ensure sync service is running

#### 3. Permission Changes Not Persisting
**Symptom**: Permission toggles reset on app restart
**Cause**: Database transaction not committing
**Solution**: Verify database write permissions and transaction handling

#### 4. High Memory Usage
**Symptom**: Application memory grows over time
**Cause**: File watcher events accumulating
**Solution**: Implement event cleanup and connection pooling

### Diagnostic Commands

```bash
# Check database integrity
sqlite3 .nexus/vault.sqlite "PRAGMA integrity_check;"

# View current schemas
sqlite3 .nexus/vault.sqlite "SELECT * FROM schemas;"

# Check permissions distribution
sqlite3 .nexus/vault.sqlite "
  SELECT 
    share_with_ai, 
    share_with_cloud, 
    COUNT(*) as count 
  FROM object_permissions 
  GROUP BY share_with_ai, share_with_cloud;
"
```

---

## Development Guidelines

### Adding New Data Types

1. **Define Schema**:
```rust
let schema = Schema {
    schema_name: "plugin.mydata.item".to_string(),
    definition_json: r#"{"type": "object", "properties": {...}}"#.to_string(),
    version: 1,
    created_at: Utc::now().to_rfc3339(),
};
```

2. **Register Schema**:
```rust
database.register_schema(schema).await?;
```

3. **Use Generic APIs**:
```rust
let object_id = database.save_object("plugin.mydata.item", content).await?;
```

### Testing Strategy

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_permission_updates() {
        let db = Database::new_in_memory().await.unwrap();
        // Test permission changes...
    }
}
```

#### Integration Tests
- Test file sync with temporary directories
- Verify permission enforcement across operations
- Validate schema registration and data storage

---

## Roadmap

### Short-term (v1.1)
- [ ] File encryption for sensitive data
- [ ] Performance monitoring dashboard
- [ ] Advanced query builder UI
- [ ] Bulk permission management

### Medium-term (v1.2)
- [ ] Plugin marketplace
- [ ] Real-time collaboration features
- [ ] Advanced conflict resolution
- [ ] Data visualization tools

### Long-term (v2.0)
- [ ] Distributed vault synchronization
- [ ] AI-powered data insights
- [ ] Advanced workflow automation
- [ ] Enterprise deployment options

---

## Conclusion

The Me-Nexus dual-storage architecture provides a robust foundation for a local-first personal data platform. By combining file-based storage with a high-performance database, the system delivers user sovereignty, application performance, and extensibility through a plugin architecture.

The granular permission system ensures user privacy while enabling powerful features like AI integration and cloud synchronization. The real-time synchronization keeps the dual storage model transparent to users, providing a seamless experience whether they interact through the application or directly with files.

This architecture establishes Me-Nexus as a platform that grows with users' needs while always respecting their data sovereignty and privacy preferences.