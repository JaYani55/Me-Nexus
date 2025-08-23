use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u32,
    pub text: String,
    pub completed: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoList {
    pub todos: Vec<Todo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultConfig {
    pub vault_path: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultInfo {
    pub path: String,
    pub is_empty: bool,
    pub exists: bool,
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
    };
    
    // Save config to app data
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    
    let config_file = app_dir.join("vault_config.json");
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_file, content).map_err(|e| e.to_string())?;
    
    // Create vault structure
    create_vault_structure(&vault_path)?;
    
    Ok(config)
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
    
    Ok(VaultInfo {
        path: path.clone(),
        exists,
        is_empty,
    })
}

#[tauri::command]
async fn open_directory_dialog(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::sync::{Arc, Mutex};
    use tokio::time::{sleep, Duration};
    
    // Get the main window
    let window = app.get_webview_window("main")
        .ok_or("Failed to get main window".to_string())?;
    
    // Create a shared result container
    let result: Arc<Mutex<Option<Option<String>>>> = Arc::new(Mutex::new(None));
    let result_clone = result.clone();
    
    // Open folder picker dialog with callback
    app.dialog()
        .file()
        .set_title("Select Vault Directory")
        .set_parent(&window)
        .pick_folder(move |folder_path| {
            let mut guard = result_clone.lock().unwrap();
            match folder_path {
                Some(path) => {
                    let path_str = path.to_string();
                    *guard = Some(Some(path_str));
                },
                None => *guard = Some(None), // User cancelled
            }
        });
    
    // Wait for the dialog result with timeout
    for _ in 0..200 { // 10 seconds timeout (50ms * 200)
        {
            let guard = result.lock().unwrap();
            if let Some(path_result) = &*guard {
                return Ok(path_result.clone());
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    
    Err("Dialog timeout - no response from file picker".to_string())
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
    
    // Create vault info file
    let vault_info_file = nexus_dir.join("vault_info.json");
    let vault_info = serde_json::json!({
        "created_at": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0",
        "structure": {
            "Todo": {
                "type": "todo_manager",
                "created_at": chrono::Utc::now().to_rfc3339()
            }
        }
    });
    let content = serde_json::to_string_pretty(&vault_info).map_err(|e| e.to_string())?;
    fs::write(&vault_info_file, content).map_err(|e| e.to_string())?;
    
    Ok(())
}

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
    
    let new_id = todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let new_todo = Todo {
        id: new_id,
        text,
        completed: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    todos.push(new_todo.clone());
    save_todos(app, todos).await?;
    
    Ok(new_todo)
}

#[tauri::command]
async fn toggle_todo(app: AppHandle, id: u32) -> Result<Vec<Todo>, String> {
    let mut todos = load_todos(app.clone()).await?;
    
    if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
        todo.completed = !todo.completed;
    }
    
    save_todos(app, todos.clone()).await?;
    Ok(todos)
}

#[tauri::command]
async fn delete_todo(app: AppHandle, id: u32) -> Result<Vec<Todo>, String> {
    let mut todos = load_todos(app.clone()).await?;
    todos.retain(|t| t.id != id);
    
    save_todos(app, todos.clone()).await?;
    Ok(todos)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_vault_config,
            set_vault_path,
            check_directory_info,
            open_directory_dialog,
            load_todos,
            save_todos,
            add_todo,
            toggle_todo,
            delete_todo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
