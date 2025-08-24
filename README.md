# 📝 Me-Nexus

A modern, vault-based todo application built with Tauri and Svelte 5. Organize your tasks in user-defined vaults with cross-platform desktop performance.

## ✨ Features

- **Vault System**: Choose any folder as your data vault (similar to Obsidian)
- **Plugin System**: Extensible architecture with secure Deno-based plugins
- **Modern UI**: Built with Svelte 5 runes and responsive design
- **Cross-Platform**: Native desktop app for Windows, macOS, and Linux
- **Local Storage**: All data stored locally in JSON files within your vault
- **First-Time Setup**: Guided vault configuration on initial launch

## 🛠️ Tech Stack

- **Frontend**: Svelte 5, TypeScript, Vite
- **Backend**: Rust, Tauri 2.x
- **Data**: JSON files in user-selected directories

## 🚀 Development Setup

### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or later)
- [Rust](https://rustup.rs/) (latest stable)
- [Git](https://git-scm.com/)

### Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/JaYani55/Me-Nexus.git
   cd Me-Nexus
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Start development server**
   ```bash
   npm run tauri dev
   ```

4. **Build for production**
   ```bash
   npm run tauri build
   ```

## 📖 About Tauri

Tauri is a framework for building lightweight, secure desktop applications using web technologies. It combines:

- **Rust backend** for system APIs and performance
- **Web frontend** (HTML/CSS/JS) for UI flexibility  
- **Small bundle size** (~10MB vs ~100MB+ for Electron)
- **Native OS integration** with proper file dialogs and system APIs

## 📁 Project Structure

```
me-nexus/
├── src/                    # Svelte frontend
│   ├── routes/            # SvelteKit routes
│   └── lib/components/    # Reusable components
├── src-tauri/             # Rust backend
│   ├── src/               # Rust source code
│   ├── sidecars/          # Deno plugin manager
│   └── Cargo.toml         # Rust dependencies
├── plugins/               # Plugin directory
│   └── test-plugin/       # Example plugin
├── docs/                  # Documentation
└── static/                # Static assets
```

## 🎯 Usage

1. **First Launch**: Select or create a vault folder
2. **Add Todos**: Use the input field to create new tasks
3. **Manage Tasks**: Check off completed items, filter views
4. **Settings**: Change vault location anytime via settings menu

Your todos are saved as `todos.json` in your vault's `/ToDo` folder.

## 🔌 Plugin Development

Me-Nexus supports a powerful plugin system built on Deno for secure, extensible functionality.

### Quick Plugin Creation

1. **Create Plugin Directory**
   ```
   plugins/my-plugin/
   ├── plugin.json    # Plugin metadata
   ├── index.ts       # Plugin implementation
   └── README.md      # Documentation
   ```

2. **Define Plugin Metadata** (`plugin.json`)
   ```json
   {
     "name": "My Plugin",
     "id": "my-plugin",
     "version": "1.0.0",
     "description": "A simple example plugin",
     "author": "Your Name",
     "main": "index.ts",
     "permissions": {
       "network": false,
       "filesystem": false,
       "system": false
     },
     "capabilities": ["ping", "custom_action"],
     "category": "utility"
   }
   ```

3. **Implement Plugin Logic** (`index.ts`)
   ```typescript
   interface PluginAPI {
     ping(): Promise<string>;
     getInfo(): Promise<any>;
   }

   class MyPlugin implements PluginAPI {
     async ping(): Promise<string> {
       return "pong";
     }

     async getInfo(): Promise<any> {
       return {
         name: "My Plugin",
         version: "1.0.0",
         status: "active"
       };
     }

     async customAction(params: any): Promise<any> {
       // Your custom logic here
       return { success: true, message: "Hello from my plugin!" };
     }
   }

   export default new MyPlugin();
   ```

### Plugin Installation

1. **Development**: Place plugin folder in `plugins/` directory
2. **Test Plugin**: Open Settings → Plugins tab, click "Test" on your plugin
3. **Production**: Copy plugin to app's data directory `plugins/` folder

### Plugin Management

- **Discover Plugins**: Settings → Plugins tab shows all installed plugins
- **Test Functionality**: Built-in testing validates plugin communication
- **Security**: Sandboxed execution with configurable permissions
- **Hot Reload**: Code changes picked up without restart (in development)

For detailed plugin development documentation, see [`docs/plugin_sidecar.md`](docs/plugin_sidecar.md).

## 📄 License

MIT License - see
