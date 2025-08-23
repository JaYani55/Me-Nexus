# 📝 Me-Nexus

A modern, vault-based todo application built with Tauri and Svelte 5. Organize your tasks in user-defined vaults with cross-platform desktop performance.

## ✨ Features

- **Vault System**: Choose any folder as your data vault (similar to Obsidian)
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
│   └── Cargo.toml         # Rust dependencies
└── static/                # Static assets
```

## 🎯 Usage

1. **First Launch**: Select or create a vault folder
2. **Add Todos**: Use the input field to create new tasks
3. **Manage Tasks**: Check off completed items, filter views
4. **Settings**: Change vault location anytime via settings menu

Your todos are saved as `todos.json` in your vault's `/ToDo` folder.

## 📄 License

MIT License - see
