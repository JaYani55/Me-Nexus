use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri_plugin_shell::{ShellExt, process::CommandEvent};
use tokio::sync::{mpsc, oneshot, Mutex};

#[derive(Serialize, Clone, Debug)]
pub struct RpcRequest {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Deserialize, Debug)]
pub struct RpcResponse {
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub struct SidecarManager {
    pub tx: mpsc::Sender<RpcRequest>,
    response_handlers: Arc<Mutex<HashMap<u64, oneshot::Sender<RpcResponse>>>>,
    next_id: Arc<Mutex<u64>>,
}

impl SidecarManager {
    pub async fn new(app_handle: tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (request_tx, mut request_rx): (mpsc::Sender<RpcRequest>, mpsc::Receiver<RpcRequest>) =
            mpsc::channel(100);
        
        let response_handlers: Arc<Mutex<HashMap<u64, oneshot::Sender<RpcResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        
        let response_handlers_clone = response_handlers.clone();

        // Spawn the deno process using the shell plugin
        // Try multiple deno paths in order of preference
        let deno_paths = [
            "deno", // If it's in PATH
            &format!("{}/.deno/bin/deno.exe", std::env::var("USERPROFILE").unwrap_or_default()),
            "C:\\Users\\%USERNAME%\\.deno\\bin\\deno.exe",
        ];
        
        let mut deno_command = None;
        for deno_path in &deno_paths {
            match app_handle.shell().command(deno_path).args(["run", "--allow-read", "--allow-net", "sidecars/plugin_manager.ts"]).spawn() {
                Ok(result) => {
                    deno_command = Some(result);
                    log::info!("Found deno at: {}", deno_path);
                    break;
                }
                Err(e) => {
                    log::debug!("Failed to spawn deno at {}: {}", deno_path, e);
                    continue;
                }
            }
        }
        
        let (mut rx, mut child) = deno_command.ok_or("Could not find deno executable")?;

        // Task for writing to the sidecar's stdin
        tauri::async_runtime::spawn(async move {
            while let Some(request) = request_rx.recv().await {
                let json_string = serde_json::to_string(&request).unwrap();
                let line = format!("{}\n", json_string);
                if let Err(e) = child.write(line.as_bytes()) {
                    log::error!("Failed to write to sidecar stdin: {}", e);
                    break;
                }
            }
        });

        // Task for reading from the sidecar's events
        tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stdout(data) => {
                        let line = String::from_utf8_lossy(&data);
                        for line in line.lines() {
                            if line.trim().is_empty() {
                                continue;
                            }
                            match serde_json::from_str::<RpcResponse>(&line.trim()) {
                                Ok(response) => {
                                    log::info!("[Deno Response]: {:?}", response);
                                    
                                    // Find and notify the waiting handler
                                    let mut handlers = response_handlers_clone.lock().await;
                                    if let Some(sender) = handlers.remove(&response.id) {
                                        let _ = sender.send(response);
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to parse response from sidecar: {} (line: {})", e, line);
                                }
                            }
                        }
                    }
                    CommandEvent::Stderr(data) => {
                        log::warn!("Sidecar stderr: {}", String::from_utf8_lossy(&data));
                    }
                    CommandEvent::Error(error) => {
                        log::error!("Sidecar error: {}", error);
                    }
                    CommandEvent::Terminated(payload) => {
                        log::info!("Sidecar terminated with code: {:?}", payload.code);
                        break;
                    }
                    _ => {
                        // Handle any other event types
                    }
                }
            }
            log::info!("Sidecar event handler finished");
        });

        Ok(SidecarManager {
            tx: request_tx,
            response_handlers,
            next_id: Arc::new(Mutex::new(1)),
        })
    }

    pub async fn send_request(&self, method: String, params: serde_json::Value) -> Result<RpcResponse, Box<dyn std::error::Error + Send + Sync>> {
        let id = {
            let mut next_id = self.next_id.lock().await;
            let current_id = *next_id;
            *next_id += 1;
            current_id
        };

        let request = RpcRequest { id, method, params };
        
        let (response_tx, response_rx) = oneshot::channel();
        
        // Register the response handler
        {
            let mut handlers = self.response_handlers.lock().await;
            handlers.insert(id, response_tx);
        }

        // Send the request
        self.tx.send(request).await?;

        // Wait for the response
        match response_rx.await {
            Ok(response) => Ok(response),
            Err(_) => Err("Request timeout or sidecar disconnected".into()),
        }
    }
}
