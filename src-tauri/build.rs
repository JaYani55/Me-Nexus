use std::path::Path;

fn main() {
    // Check for plugin_manager sidecar (warn but don't fail build)
    let sidecar_path = Path::new("sidecars/plugin_manager.ts");
    if !sidecar_path.exists() {
        println!("cargo:warning=Plugin manager sidecar not found at: {:?}", sidecar_path);
    }

    tauri_build::build()
}
