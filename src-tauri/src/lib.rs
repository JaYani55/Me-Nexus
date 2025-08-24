use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

// Import our new modules
mod error;
mod models;
mod database;
mod sync_service;
mod sidecar;

use models::{VaultConfig, VaultInfo, Todo, Permissions, PluginMetadata, InstalledPlugin, PluginStatus};

// Application state for managing the database and sync service
pub struct AppState {
    database: Option<Arc<database::Database>>,
    sync_service: Option<Arc<Mutex<sync_service::SyncService>>>,
    sidecar_manager: Option<Arc<sidecar::SidecarManager>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            database: None,
            sync_service: None,
            sidecar_manager: None,
        }
    }
}

// Legacy structures for backward compatibility
#[derive(Debug, Serialize, Deserialize)]
pub struct TodoList {
    pub todos: Vec<Todo>,
}

// Initialize logging
fn init_logging() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Vault Management Commands
#[tauri::command]
async fn get_vault_config(app: AppHandle) -> Result<Option<VaultConfig>, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let config_file = app_dir.join("vault_config.json");
    
    if !config_file.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(&config_file).map_err(|e| e.to_string())?;
    let config: VaultConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    
    Ok(Some(config))
}

#[tauri::command]
async fn set_vault_path(app: AppHandle, vault_path: String) -> Result<VaultConfig, String> {
    let path = Path::new(&vault_path);
    
    if !path.exists() {
        return Err("Selected path does not exist".to_string());
    }
    
    if !path.is_dir() {
        return Err("Selected path is not a directory".to_string());
    }
    
    // Create vault config
    let config = VaultConfig {
        vault_path: vault_path.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        version: "1.0.0".to_string(),
        encryption_enabled: false,
    };
    
    // Save config to app data
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    
    let config_file = app_dir.join("vault_config.json");
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_file, content).map_err(|e| e.to_string())?;
    
    // Create vault structure
    create_vault_structure(&vault_path)?;
    
    // Initialize the database and sync service
    match initialize_vault_backend(&app, &vault_path).await {
        Ok(_) => {
            log::info!("Vault backend initialized successfully");
        }
        Err(e) => {
            log::error!("Failed to initialize vault backend: {}", e);
            // Don't fail the vault setup if backend initialization fails
        }
    }
    
    Ok(config)
}

// Initialize the database and sync service for a vault
async fn initialize_vault_backend(app: &AppHandle, vault_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let vault_path = Path::new(vault_path);
    
    // Create database
    let database = Arc::new(database::Database::new(vault_path).await?);
    
    // Create sync service
    let mut sync_service = sync_service::SyncService::new(Arc::clone(&database), vault_path).await?;
    sync_service.start().await?;
    let sync_service = Arc::new(Mutex::new(sync_service));
    
    // Store in app state
    let state = app.state::<Mutex<AppState>>();
    let mut app_state = state.lock().await;
    app_state.database = Some(database);
    app_state.sync_service = Some(sync_service);
    
    log::info!("Vault backend initialized for path: {}", vault_path.display());
    Ok(())
}

#[tauri::command]
async fn check_directory_info(path: String) -> Result<VaultInfo, String> {
    let dir_path = Path::new(&path);
    
    let exists = dir_path.exists();
    let is_empty = if exists && dir_path.is_dir() {
        fs::read_dir(dir_path)
            .map_err(|e| e.to_string())?
            .next()
            .is_none()
    } else {
        false
    };
    
    let nexus_dir = dir_path.join(".nexus");
    let has_nexus_folder = nexus_dir.exists();
    let database_exists = nexus_dir.join("vault.sqlite").exists();
    
    Ok(VaultInfo {
        path: path.clone(),
        exists,
        is_empty,
        has_nexus_folder,
        database_exists,
    })
}

fn create_vault_structure(vault_path: &str) -> Result<(), String> {
    let vault_dir = Path::new(vault_path);
    
    // Create main vault directory if it doesn't exist
    fs::create_dir_all(vault_dir).map_err(|e| e.to_string())?;
    
    // Create Todo directory
    let todo_dir = vault_dir.join("Todo");
    fs::create_dir_all(&todo_dir).map_err(|e| e.to_string())?;
    
    // Create initial todos.json file
    let todos_file = todo_dir.join("todos.json");
    if !todos_file.exists() {
        let empty_list = TodoList { todos: vec![] };
        let content = serde_json::to_string_pretty(&empty_list).map_err(|e| e.to_string())?;
        fs::write(&todos_file, content).map_err(|e| e.to_string())?;
    }
    
    // Create a .nexus directory for metadata
    let nexus_dir = vault_dir.join(".nexus");
    fs::create_dir_all(&nexus_dir).map_err(|e| e.to_string())?;
    
    // Create plugins directory
    let plugins_dir = vault_dir.join("plugins");
    fs::create_dir_all(&plugins_dir).map_err(|e| e.to_string())?;
    
    // Create vault info file
    let vault_info_file = nexus_dir.join("vault_info.json");
    let vault_info = serde_json::json!({
        "created_at": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0",
        "structure": {
            "Todo": {
                "type": "todo_manager",
                "created_at": chrono::Utc::now().to_rfc3339()
            },
            "plugins": {
                "type": "plugin_directory",
                "created_at": chrono::Utc::now().to_rfc3339()
            }
        }
    });
    let content = serde_json::to_string_pretty(&vault_info).map_err(|e| e.to_string())?;
    fs::write(&vault_info_file, content).map_err(|e| e.to_string())?;
    
    Ok(())
}

// Legacy Todo commands for backward compatibility
fn get_vault_todos_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config = match get_vault_config_sync(app)? {
        Some(config) => config,
        None => return Err("No vault configured. Please set up a vault first.".to_string()),
    };
    
    let vault_path = Path::new(&config.vault_path);
    if !vault_path.exists() {
        return Err("Vault directory no longer exists. Please reconfigure vault.".to_string());
    }
    
    Ok(vault_path.join("Todo").join("todos.json"))
}

fn get_vault_config_sync(app: &AppHandle) -> Result<Option<VaultConfig>, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let config_file = app_dir.join("vault_config.json");
    
    if !config_file.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(&config_file).map_err(|e| e.to_string())?;
    let config: VaultConfig = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    
    Ok(Some(config))
}

#[tauri::command]
async fn load_todos(app: AppHandle) -> Result<Vec<Todo>, String> {
    let todos_file = get_vault_todos_path(&app)?;
    
    if !todos_file.exists() {
        return Ok(vec![]);
    }
    
    let content = fs::read_to_string(&todos_file).map_err(|e| e.to_string())?;
    let todo_list: TodoList = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    
    Ok(todo_list.todos)
}

#[tauri::command]
async fn save_todos(app: AppHandle, todos: Vec<Todo>) -> Result<(), String> {
    let todos_file = get_vault_todos_path(&app)?;
    
    // Ensure the directory exists
    if let Some(parent) = todos_file.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    let todo_list = TodoList { todos };
    let content = serde_json::to_string_pretty(&todo_list).map_err(|e| e.to_string())?;
    fs::write(&todos_file, content).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn add_todo(app: AppHandle, text: String) -> Result<Todo, String> {
    let mut todos = load_todos(app.clone()).await?;
    
    let new_id = todos.iter().filter_map(|t| t.id).max().unwrap_or(0) + 1;
    let mut new_todo = Todo::new(text);
    new_todo.id = Some(new_id);
    
    todos.push(new_todo.clone());
    save_todos(app, todos).await?;
    
    Ok(new_todo)
}

#[tauri::command]
async fn toggle_todo(app: AppHandle, id: u32) -> Result<Vec<Todo>, String> {
    let mut todos = load_todos(app.clone()).await?;
    
    if let Some(todo) = todos.iter_mut().find(|t| t.id == Some(id)) {
        todo.completed = !todo.completed;
        todo.mark_updated();
    }
    
    save_todos(app, todos.clone()).await?;
    Ok(todos)
}

// New backend-powered Todo commands
#[tauri::command]
async fn load_todos_v2(app: AppHandle) -> Result<Vec<models::AppObject<Todo>>, String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().await;
    
    if let Some(database) = &app_state.database {
        let todos = database.load_objects_by_schema("core.todo").await.map_err(|e| e.to_string())?;
        Ok(todos)
    } else {
        Err("Database not initialized. Please configure a vault first.".to_string())
    }
}

#[tauri::command]
async fn add_todo_v2(app: AppHandle, text: String) -> Result<models::AppObject<Todo>, String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().await;
    
    if let Some(database) = &app_state.database {
        let todo = Todo::new(text);
        let object_id = database.save_object(
            "core.todo",
            &todo,
            None, // We could specify a file path here
            None, // Default permissions
        ).await.map_err(|e| e.to_string())?;
        
        let saved_todo = database.load_object(object_id).await.map_err(|e| e.to_string())?;
        Ok(saved_todo)
    } else {
        Err("Database not initialized. Please configure a vault first.".to_string())
    }
}

#[tauri::command]
async fn update_todo_permissions(
    app: AppHandle,
    object_id: i64,
    permissions: Permissions,
) -> Result<(), String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().await;
    
    if let Some(database) = &app_state.database {
        database.update_object_permissions(object_id, &permissions).await.map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Database not initialized. Please configure a vault first.".to_string())
    }
}

#[tauri::command]
async fn get_sync_status(app: AppHandle) -> Result<models::SyncStatus, String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().await;
    
    if let Some(sync_service) = &app_state.sync_service {
        let service = sync_service.lock().await;
        let status = service.get_status().await;
        Ok(status)
    } else {
        Err("Sync service not initialized. Please configure a vault first.".to_string())
    }
}

#[tauri::command]
async fn get_all_vault_objects(app: AppHandle) -> Result<Vec<models::AppObject<serde_json::Value>>, String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().await;
    
    if let Some(database) = &app_state.database {
        // Get all objects from all schemas
        let todos: Vec<models::AppObject<serde_json::Value>> = database
            .load_objects_by_schema("core.todo")
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|obj: models::AppObject<Todo>| models::AppObject {
                id: obj.id,
                schema_name: obj.schema_name,
                content: serde_json::to_value(&obj.content).unwrap_or_default(),
                permissions: obj.permissions,
                file_path: obj.file_path,
                updated_at: obj.updated_at,
                created_at: obj.created_at,
            })
            .collect();
        
        Ok(todos)
    } else {
        Err("Database not initialized. Please configure a vault first.".to_string())
    }
}

#[tauri::command]
async fn update_object_permissions(
    app: AppHandle,
    object_id: i64,
    permissions: Permissions,
) -> Result<(), String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().await;
    
    if let Some(database) = &app_state.database {
        database.update_object_permissions(object_id, &permissions).await.map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Database not initialized. Please configure a vault first.".to_string())
    }
}

#[tauri::command]
async fn delete_todo(app: AppHandle, id: u32) -> Result<Vec<Todo>, String> {
    let mut todos = load_todos(app.clone()).await?;
    todos.retain(|t| t.id != Some(id));
    
    save_todos(app, todos.clone()).await?;
    Ok(todos)
}

// Plugin system test command
#[tauri::command]
async fn ping_plugins(app: AppHandle) -> Result<String, String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().await;
    
    if let Some(ref manager) = app_state.sidecar_manager {
        match manager.send_request("ping".to_string(), serde_json::Value::Null).await {
            Ok(response) => {
                if let Some(error) = response.error {
                    Err(format!("Sidecar error: {}", error))
                } else if let Some(result) = response.result {
                    Ok(format!("Plugin response: {}", result))
                } else {
                    Ok("Plugin responded successfully".to_string())
                }
            }
            Err(e) => Err(format!("Failed to communicate with plugins: {}", e))
        }
    } else {
        Err("Plugin system not initialized".to_string())
    }
}

// Get plugin manager information
#[tauri::command]
async fn get_plugin_info(app: AppHandle) -> Result<serde_json::Value, String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().await;
    
    if let Some(ref manager) = app_state.sidecar_manager {
        match manager.send_request("get_info".to_string(), serde_json::Value::Null).await {
            Ok(response) => {
                if let Some(error) = response.error {
                    Err(format!("Sidecar error: {}", error))
                } else if let Some(result) = response.result {
                    Ok(result)
                } else {
                    Err("No result from plugin manager".to_string())
                }
            }
            Err(e) => Err(format!("Failed to communicate with plugins: {}", e))
        }
    } else {
        Err("Plugin system not initialized".to_string())
    }
}

// Plugin management commands
#[tauri::command]
async fn discover_plugins(app: AppHandle) -> Result<Vec<InstalledPlugin>, String> {
    let plugins_dir = get_plugins_directory(&app)?;
    log::info!("Looking for plugins in directory: {:?}", plugins_dir);
    let mut plugins = Vec::new();

    if !plugins_dir.exists() {
        log::info!("Plugins directory does not exist, creating it: {:?}", plugins_dir);
        fs::create_dir_all(&plugins_dir).map_err(|e| format!("Failed to create plugins directory: {}", e))?;
        return Ok(plugins);
    }

    let entries = fs::read_dir(&plugins_dir).map_err(|e| format!("Failed to read plugins directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        log::info!("Checking path: {:?}", path);

        if path.is_dir() {
            let plugin_json_path = path.join("plugin.json");
            if plugin_json_path.exists() {
                match load_plugin_metadata(&plugin_json_path) {
                    Ok(metadata) => {
                        let plugin = InstalledPlugin {
                            metadata,
                            path: path.to_string_lossy().to_string(),
                            enabled: true, // Default to enabled
                            installed_at: chrono::Utc::now().to_rfc3339(),
                            last_used: None,
                        };
                        plugins.push(plugin);
                    }
                    Err(e) => {
                        log::warn!("Failed to load plugin metadata from {:?}: {}", plugin_json_path, e);
                    }
                }
            }
        }
    }

    Ok(plugins)
}

#[tauri::command]
async fn test_plugin(app: AppHandle, plugin_id: String) -> Result<PluginStatus, String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().await;
    
    if let Some(ref manager) = app_state.sidecar_manager {
        let params = serde_json::json!({ "plugin_id": plugin_id });
        match manager.send_request("test_plugin".to_string(), params).await {
            Ok(response) => {
                if let Some(error) = response.error {
                    Ok(PluginStatus {
                        plugin_id: plugin_id.clone(),
                        status: "error".to_string(),
                        last_ping: Some(chrono::Utc::now().to_rfc3339()),
                        error_message: Some(error),
                    })
                } else {
                    Ok(PluginStatus {
                        plugin_id: plugin_id.clone(),
                        status: "active".to_string(),
                        last_ping: Some(chrono::Utc::now().to_rfc3339()),
                        error_message: None,
                    })
                }
            }
            Err(e) => Ok(PluginStatus {
                plugin_id: plugin_id.clone(),
                status: "error".to_string(),
                last_ping: Some(chrono::Utc::now().to_rfc3339()),
                error_message: Some(e.to_string()),
            })
        }
    } else {
        Err("Plugin system not initialized".to_string())
    }
}

fn get_plugins_directory(app: &AppHandle) -> Result<PathBuf, String> {
    // Get the current vault configuration to find the vault path
    if let Some(config) = get_vault_config_sync(app)? {
        let plugins_path = Path::new(&config.vault_path).join("plugins");
        log::info!("Using vault plugins directory: {:?}", plugins_path);
        
        // Ensure the plugins directory exists
        if let Err(e) = fs::create_dir_all(&plugins_path) {
            log::warn!("Failed to create plugins directory: {}", e);
        }
        
        Ok(plugins_path)
    } else {
        Err("No vault configuration found. Please set up a vault first.".to_string())
    }
}

fn load_plugin_metadata(plugin_json_path: &Path) -> Result<PluginMetadata, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(plugin_json_path)?;
    let metadata: PluginMetadata = serde_json::from_str(&content)?;
    Ok(metadata)
}

// Plugin installation commands
#[tauri::command]
async fn open_plugin_file_dialog() -> Result<Option<String>, String> {
    // This will be called from the frontend which will then call install_plugin_from_path
    Ok(None) // Placeholder - frontend will handle file dialog
}

#[tauri::command]
async fn install_plugin_from_path(app: AppHandle, file_path: String) -> Result<String, String> {
    let plugins_dir = get_plugins_directory(&app)?;
    
    // Ensure plugins directory exists
    if !plugins_dir.exists() {
        fs::create_dir_all(&plugins_dir).map_err(|e| format!("Failed to create plugins directory: {}", e))?;
    }

    let archive_path = Path::new(&file_path);
    if !archive_path.exists() {
        return Err("File does not exist".to_string());
    }

    extract_plugin_archive(archive_path, &plugins_dir)?;
    Ok(format!("Plugin installed from: {}", file_path))
}

#[tauri::command]
async fn install_plugin_from_github(app: AppHandle, github_url: String) -> Result<String, String> {
    use std::process::Command;
    
    let plugins_dir = get_plugins_directory(&app)?;
    
    // Ensure plugins directory exists
    if !plugins_dir.exists() {
        fs::create_dir_all(&plugins_dir).map_err(|e| format!("Failed to create plugins directory: {}", e))?;
    }

    // Validate GitHub URL
    if !github_url.starts_with("https://github.com/") && !github_url.starts_with("git@github.com:") {
        return Err("Invalid GitHub URL. Must start with https://github.com/ or git@github.com:".to_string());
    }

    // Extract repository name for the folder
    let repo_name = github_url
        .split('/')
        .last()
        .unwrap_or("unknown-plugin")
        .replace(".git", "");

    let plugin_path = plugins_dir.join(&repo_name);

    // Clone the repository
    let output = Command::new("git")
        .args(&["clone", &github_url, plugin_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to execute git clone: {}", e))?;

    if output.status.success() {
        // Verify the plugin has the required files
        let plugin_json = plugin_path.join("plugin.json");
        if plugin_json.exists() {
            Ok(format!("Plugin '{}' installed successfully from GitHub", repo_name))
        } else {
            // Clean up invalid plugin
            let _ = fs::remove_dir_all(&plugin_path);
            Err("Invalid plugin: plugin.json not found in repository".to_string())
        }
    } else {
        Err(format!("Git clone failed: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

#[tauri::command]
async fn remove_plugin(app: AppHandle, plugin_id: String) -> Result<String, String> {
    let plugins_dir = get_plugins_directory(&app)?;
    let plugin_path = plugins_dir.join(&plugin_id);

    if plugin_path.exists() {
        fs::remove_dir_all(&plugin_path).map_err(|e| format!("Failed to remove plugin: {}", e))?;
        Ok(format!("Plugin '{}' removed successfully", plugin_id))
    } else {
        Err(format!("Plugin '{}' not found", plugin_id))
    }
}

fn extract_plugin_archive(archive_path: &Path, plugins_dir: &Path) -> Result<(), String> {
    use std::process::Command;
    
    let extension = archive_path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "zip" => {
            // Use built-in zip extraction
            let file = fs::File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;
            let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {}", e))?;
            
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|e| format!("Failed to read zip entry: {}", e))?;
                let outpath = plugins_dir.join(file.mangled_name());

                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
                }

                if !file.name().ends_with('/') {
                    let mut outfile = fs::File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
                    std::io::copy(&mut file, &mut outfile).map_err(|e| format!("Failed to extract file: {}", e))?;
                }
            }
            Ok(())
        }
        "rar" | "7z" => {
            // Use 7zip for rar and 7z files
            let output = Command::new("7z")
                .args(&["x", archive_path.to_str().unwrap(), &format!("-o{}", plugins_dir.to_str().unwrap())])
                .output()
                .map_err(|e| format!("Failed to extract with 7z: {}. Make sure 7-Zip is installed.", e))?;

            if output.status.success() {
                Ok(())
            } else {
                Err(format!("7z extraction failed: {}", String::from_utf8_lossy(&output.stderr)))
            }
        }
        _ => Err(format!("Unsupported archive format: {}", extension))
    }
}

// Initialize existing vault on app startup
async fn initialize_existing_vault(app: &AppHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(config) = get_vault_config_sync(app)? {
        log::info!("Found existing vault configuration, initializing...");
        initialize_vault_backend(app, &config.vault_path).await?;
        log::info!("Existing vault initialized successfully");
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(AppState::new()))
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_handle_clone = app_handle.clone();
            
            tauri::async_runtime::spawn(async move {
                if let Err(e) = initialize_existing_vault(&app_handle).await {
                    log::error!("Failed to initialize existing vault: {}", e);
                }
            });

            // Initialize the sidecar manager
            tauri::async_runtime::spawn(async move {
                match sidecar::SidecarManager::new(app_handle_clone.clone()).await {
                    Ok(manager) => {
                        let state = app_handle_clone.state::<Mutex<AppState>>();
                        let mut app_state = state.lock().await;
                        app_state.sidecar_manager = Some(Arc::new(manager));
                        log::info!("Sidecar manager initialized successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to initialize sidecar manager: {}", e);
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_vault_config,
            set_vault_path,
            check_directory_info,
            load_todos,
            save_todos,
            add_todo,
            toggle_todo,
            delete_todo,
            // New backend-powered commands
            load_todos_v2,
            add_todo_v2,
            update_todo_permissions,
            get_sync_status,
            get_all_vault_objects,
            update_object_permissions,
            // Plugin system commands
            ping_plugins,
            get_plugin_info,
            discover_plugins,
            test_plugin,
            open_plugin_file_dialog,
            install_plugin_from_path,
            install_plugin_from_github,
            remove_plugin
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
