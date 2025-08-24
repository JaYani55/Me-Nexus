# 🔌 Plugin System & Sidecar Documentation

Me-Nexus implements a secure, extensible plugin system using Tauri's sidecar functionality with Deno as the runtime. This document provides comprehensive technical details about the architecture, implementation, and development workflows.

## 📋 Table of Contents

- [Architecture Overview](#architecture-overview)
- [Component Details](#component-details)
- [Plugin Development](#plugin-development)
- [Security Model](#security-model)
- [API Reference](#api-reference)
- [Troubleshooting](#troubleshooting)

## 🏗️ Architecture Overview

### System Architecture

```
┌─────────────────┐    JSON-RPC     ┌─────────────────┐    Dynamic     ┌─────────────────┐
│   Rust Backend │ ◄─────────────► │ Deno Sidecar   │ ◄─────────────► │   Plugin Files  │
│   (Tauri Core) │                 │ (plugin_manager)│                 │   (TypeScript)  │
└─────────────────┘                 └─────────────────┘                 └─────────────────┘
         ▲                                    ▲                                    ▲
         │                                    │                                    │
         │ IPC Commands                       │ Secure Sandbox                     │ File System
         │                                    │ - Permissions                      │ - plugin.json
         ▼                                    │ - Resource Limits                  │ - index.ts
┌─────────────────┐                          │ - Network Isolation               │ - README.md
│ Svelte Frontend │                          ▼                                    │
│  (Settings UI)  │                 ┌─────────────────┐                          │
└─────────────────┘                 │   Deno Runtime  │                          │
                                    │   - V8 Engine   │                          │
                                    │   - TypeScript  │                          │
                                    │   - Security    │                          │
                                    └─────────────────┘                          │
                                             ▲                                    │
                                             │                                    │
                                             └────────────────────────────────────┘
```

### Communication Flow

1. **Frontend Request**: User interacts with plugin management UI
2. **Tauri IPC**: Frontend calls Rust backend via `invoke()` commands
3. **Plugin Discovery**: Backend scans `plugins/` directory for valid plugins
4. **Sidecar Communication**: Backend communicates with Deno sidecar via JSON-RPC
5. **Plugin Execution**: Sidecar dynamically loads and executes plugin code
6. **Response Chain**: Results flow back through the same chain

## 🔧 Component Details

### 1. Rust Backend (`src-tauri/src/lib.rs`)

#### Plugin Discovery System

```rust
#[tauri::command]
async fn discover_plugins(app: AppHandle) -> Result<Vec<InstalledPlugin>, String> {
    let plugins_dir = get_plugins_directory(&app)?;
    // Scans directory for plugin.json files
    // Validates plugin metadata
    // Returns structured plugin information
}
```

**Key Functions:**
- `get_plugins_directory()`: Resolves plugin directory path (dev vs production)
- `load_plugin_metadata()`: Parses and validates `plugin.json` files
- `discover_plugins()`: Main discovery command exposed to frontend
- `test_plugin()`: Plugin testing and validation

#### Development vs Production Paths

```rust
fn get_plugins_directory(_app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        // Development: ./plugins (project root)
        let current_dir = std::env::current_dir()?;
        let mut project_root = current_dir.clone();
        
        if project_root.ends_with("src-tauri") {
            project_root = project_root.parent().unwrap().to_path_buf();
        }
        
        Ok(project_root.join("plugins"))
    }
    
    #[cfg(not(debug_assertions))]
    {
        // Production: App data directory
        let app_data_dir = _app.path().app_data_dir()?;
        Ok(app_data_dir.join("plugins"))
    }
}
```

### 2. Deno Sidecar (`src-tauri/sidecars/plugin_manager.ts`)

#### JSON-RPC Protocol Implementation

```typescript
interface RpcRequest {
  id: number;
  method: string;
  params: unknown;
}

interface RpcResponse {
  id: number;
  result?: unknown;
  error?: string;
}
```

#### Command Handlers

- **`ping`**: Health check and connectivity test
- **`get_info`**: Runtime information and capabilities
- **`list_plugins`**: Available plugin enumeration
- **`test_plugin`**: Plugin validation and testing

#### Main Event Loop

```typescript
async function main() {
  for await (const line of readLines(Deno.stdin)) {
    try {
      const request: RpcRequest = JSON.parse(line);
      
      // Route to appropriate handler
      switch (request.method) {
        case "ping": result = await handlePing(); break;
        case "test_plugin": result = await handleTestPlugin(request.params); break;
        // ... other handlers
      }
      
      // Send response back to Rust
      console.log(JSON.stringify({ id: request.id, result }));
    } catch (e) {
      // Error handling with proper type safety
    }
  }
}
```

### 3. Frontend Integration (`src/lib/components/SettingsModal.svelte`)

#### Plugin Management UI

```svelte
<!-- Plugin Discovery -->
{#if activeTab === 'plugins'}
  <div class="plugins-content">
    <div class="plugins-header">
      <h3>Plugin Management</h3>
      <button onclick={refreshPlugins}>Refresh</button>
    </div>

    <!-- Plugin List -->
    {#each plugins as plugin}
      <div class="plugin-card">
        <div class="plugin-info">
          <h4>{plugin.metadata.name}</h4>
          <p>{plugin.metadata.description}</p>
          <div class="plugin-meta">
            <span>v{plugin.metadata.version}</span>
            <span>{plugin.metadata.author}</span>
          </div>
        </div>
        <div class="plugin-actions">
          <button onclick={() => testPlugin(plugin.metadata.id)}>
            Test
          </button>
        </div>
      </div>
    {/each}
  </div>
{/if}
```

#### State Management with Svelte 5 Runes

```svelte
<script lang="ts">
  // Plugin state management
  let plugins = $state<InstalledPlugin[]>([]);
  let pluginTestResults = $state<Record<string, string>>({});
  let isLoadingPlugins = $state(false);

  // Reactive plugin discovery
  async function refreshPlugins() {
    isLoadingPlugins = true;
    try {
      plugins = await invoke<InstalledPlugin[]>('discover_plugins');
    } catch (error) {
      console.error('Failed to discover plugins:', error);
    } finally {
      isLoadingPlugins = false;
    }
  }
</script>
```

## 🛠️ Plugin Development

### Plugin Structure

Every plugin must follow this directory structure:

```
plugins/
└── your-plugin-name/
    ├── plugin.json      # Plugin metadata (required)
    ├── index.ts         # Main plugin code (required)
    └── README.md        # Documentation (recommended)
```

### Plugin Metadata (`plugin.json`)

```json
{
  "name": "Your Plugin Name",
  "id": "your-plugin-id",
  "version": "1.0.0",
  "description": "Plugin description",
  "author": "Your Name",
  "main": "index.ts",
  "permissions": {
    "network": false,
    "filesystem": false,
    "system": false
  },
  "capabilities": [
    "ping",
    "custom_action"
  ],
  "category": "utility",
  "tags": ["example", "tutorial"]
}
```

#### Metadata Fields

- **`name`**: Human-readable plugin name
- **`id`**: Unique identifier (kebab-case recommended)
- **`version`**: Semantic version string
- **`description`**: Brief functionality description
- **`author`**: Plugin creator information
- **`main`**: Entry point file (relative to plugin directory)
- **`permissions`**: Security permissions object
- **`capabilities`**: Array of supported operations
- **`category`**: Plugin category for organization
- **`tags`**: Search and filtering tags

### Plugin Implementation (`index.ts`)

```typescript
// Plugin API interface
interface PluginAPI {
  // Core methods every plugin should implement
  ping(): Promise<string>;
  getInfo(): Promise<PluginInfo>;
  
  // Custom plugin methods
  [key: string]: any;
}

// Plugin information structure
interface PluginInfo {
  name: string;
  version: string;
  status: 'active' | 'inactive' | 'error';
  capabilities: string[];
}

// Example plugin implementation
class MyPlugin implements PluginAPI {
  async ping(): Promise<string> {
    return "pong";
  }

  async getInfo(): Promise<PluginInfo> {
    return {
      name: "My Plugin",
      version: "1.0.0",
      status: "active",
      capabilities: ["ping", "custom_action"]
    };
  }

  async customAction(params: any): Promise<any> {
    // Your custom plugin logic here
    return { success: true, message: "Action completed" };
  }
}

// Export plugin instance
export default new MyPlugin();
```

### Security Permissions

The permission system controls what system resources plugins can access:

```json
{
  "permissions": {
    "network": false,     // HTTP/HTTPS requests
    "filesystem": false,  // File system access
    "system": false       // System commands and processes
  }
}
```

**Security Levels:**
- **No Permissions**: Sandbox execution only
- **Filesystem**: Read/write access to designated directories
- **Network**: HTTP requests with configurable domains
- **System**: Process execution and system API access

## 🔒 Security Model

### Sandbox Isolation

1. **Deno Runtime**: Secure-by-default execution environment
2. **Permission Controls**: Granular access to system resources
3. **Process Isolation**: Plugins run in separate Deno process
4. **Resource Limits**: Memory and CPU constraints
5. **Network Restrictions**: Configurable domain allowlists

### Communication Security

- **JSON-RPC**: Structured, validated message format
- **Type Safety**: TypeScript ensures message contract compliance
- **Error Isolation**: Plugin errors don't crash main application
- **Input Validation**: All plugin inputs validated before processing

## 📚 API Reference

### Tauri Commands

#### `discover_plugins() -> Vec<InstalledPlugin>`
Scans plugin directory and returns list of valid plugins.

**Returns:**
```typescript
interface InstalledPlugin {
  metadata: PluginMetadata;
  status: PluginStatus;
  path: string;
}
```

#### `test_plugin(plugin_id: string) -> PluginTestResult`
Tests plugin functionality and communication.

**Parameters:**
- `plugin_id`: Unique plugin identifier

**Returns:**
```typescript
interface PluginTestResult {
  plugin_id: string;
  status: 'success' | 'error';
  message: string;
  timestamp: string;
}
```

### Sidecar RPC Methods

#### `ping() -> "pong"`
Health check for sidecar communication.

#### `get_info() -> SidecarInfo`
Returns sidecar runtime information.

```typescript
interface SidecarInfo {
  version: string;
  runtime: "Deno";
  denoVersion: string;
  capabilities: string[];
  timestamp: string;
}
```

#### `test_plugin(params: {plugin_id: string}) -> PluginTestResult`
Executes plugin test sequence.

## 🐛 Troubleshooting

### Common Issues

#### Plugin Not Discovered
1. **Check Directory Structure**: Ensure `plugin.json` exists in plugin folder
2. **Validate JSON**: Use JSON validator to check metadata syntax
3. **Verify Permissions**: Ensure plugin directory is readable
4. **Check Logs**: Look for discovery errors in terminal output

#### Sidecar Communication Errors
1. **Deno Installation**: Verify Deno is installed and accessible
2. **Process Permissions**: Check if Deno can be executed
3. **JSON-RPC Format**: Validate message structure in logs
4. **Timeout Issues**: Increase communication timeout if needed

#### Plugin Execution Failures
1. **TypeScript Errors**: Check plugin code for syntax issues
2. **Permission Denied**: Verify plugin has required permissions
3. **Dependency Issues**: Ensure all imports are accessible
4. **Runtime Errors**: Check sidecar stderr for detailed errors

### Debug Logging

Enable detailed logging by setting environment variables:

```bash
# Rust backend logging
RUST_LOG=debug

# Tauri development mode
npm run tauri dev
```

### Log Analysis

Key log patterns to watch for:

```
[INFO] Development mode: plugins directory resolved to: "path/to/plugins"
[INFO] Looking for plugins in directory: "path"
[INFO] Checking path: "path/to/plugin"
[DEBUG] Test plugin handled, result: {...}
```

## 🚀 Performance Considerations

### Startup Time
- Plugin discovery runs once at application start
- Metadata parsing is cached for subsequent requests
- Sidecar initialization adds ~100ms to startup

### Memory Usage
- Each plugin runs in isolated Deno process
- Base overhead: ~10MB per active plugin
- Memory limits can be configured per plugin

### Execution Speed
- JSON-RPC adds ~1-5ms communication overhead
- TypeScript compilation cached by Deno
- Plugin complexity affects execution time

## 🔄 Development Workflow

### Plugin Development Cycle

1. **Create Plugin Directory**: `plugins/my-plugin/`
2. **Write Metadata**: Complete `plugin.json` specification
3. **Implement Plugin**: Write TypeScript implementation
4. **Test Locally**: Use Settings UI to test plugin
5. **Debug Issues**: Check logs and error messages
6. **Iterate**: Refine implementation based on testing

### Hot Reloading

During development:
- Plugin metadata changes require app restart
- Plugin code changes are picked up on next execution
- Sidecar restarts automatically on code changes

## 📈 Future Enhancements

### Planned Features

1. **Plugin Marketplace**: Central repository for plugin discovery
2. **Visual Plugin Builder**: GUI tool for plugin creation
3. **Advanced Permissions**: Fine-grained resource control
4. **Plugin Dependencies**: Support for inter-plugin communication
5. **Performance Monitoring**: Plugin resource usage tracking

### Extension Points

The current architecture supports future expansion:

- **Custom UI Components**: Plugin-defined interface elements
- **Event System**: Plugin subscription to application events
- **Data Persistence**: Plugin-specific data storage
- **External Integrations**: Third-party service connections

---

*This documentation is maintained alongside the codebase. For the latest updates, refer to the source code and commit history.*