<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import VaultSetup from "$lib/components/VaultSetup.svelte";

  interface Todo {
    id: number;
    text: string;
    completed: boolean;
    created_at: string;
  }

  interface VaultConfig {
    vault_path: string;
    created_at: string;
  }

  // State using Svelte 5 runes
  let todos = $state<Todo[]>([]);
  let newTodoText = $state("");
  let filter = $state<"all" | "active" | "completed">("all");
  let isLoading = $state(false);
  let vaultConfig = $state<VaultConfig | null>(null);
  let showSettings = $state(false);
  let showVaultSetup = $state(false);
  let isCheckingVault = $state(true);

  // Derived states
  let filteredTodos = $derived.by(() => {
    switch (filter) {
      case "active":
        return todos.filter(todo => !todo.completed);
      case "completed":
        return todos.filter(todo => todo.completed);
      default:
        return todos;
    }
  });

  let completedCount = $derived(todos.filter(todo => todo.completed).length);
  let activeCount = $derived(todos.filter(todo => !todo.completed).length);
  let totalCount = $derived(todos.length);

  // Check for vault on mount
  onMount(async () => {
    await checkVaultSetup();
  });

  async function checkVaultSetup() {
    try {
      isCheckingVault = true;
      const config = await invoke<VaultConfig | null>("get_vault_config");
      
      if (config) {
        vaultConfig = config;
        await loadTodos();
      } else {
        showVaultSetup = true;
      }
    } catch (error) {
      console.error("Failed to check vault setup:", error);
      showVaultSetup = true;
    } finally {
      isCheckingVault = false;
    }
  }

  async function loadTodos() {
    try {
      isLoading = true;
      todos = await invoke<Todo[]>("load_todos");
    } catch (error) {
      console.error("Failed to load todos:", error);
      // If loading fails due to vault issues, show setup
      if (error?.toString().includes("No vault configured")) {
        showVaultSetup = true;
      }
    } finally {
      isLoading = false;
    }
  }

  async function addTodo(event: Event) {
    event.preventDefault();
    if (!newTodoText.trim()) return;

    try {
      const newTodo = await invoke<Todo>("add_todo", { text: newTodoText.trim() });
      todos = [...todos, newTodo];
      newTodoText = "";
    } catch (error) {
      console.error("Failed to add todo:", error);
    }
  }

  async function toggleTodo(id: number) {
    try {
      todos = await invoke<Todo[]>("toggle_todo", { id });
    } catch (error) {
      console.error("Failed to toggle todo:", error);
    }
  }

  async function deleteTodo(id: number) {
    try {
      todos = await invoke<Todo[]>("delete_todo", { id });
    } catch (error) {
      console.error("Failed to delete todo:", error);
    }
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString();
  }

  function handleVaultSetup(config: VaultConfig) {
    vaultConfig = config;
    showVaultSetup = false;
    loadTodos();
  }

  function handleVaultChanged() {
    checkVaultSetup();
  }
</script>

{#if isCheckingVault}
  <div class="loading-screen">
    <div class="loading-content">
      <div class="spinner"></div>
      <p>Checking vault configuration...</p>
    </div>
  </div>
{:else if showVaultSetup}
  <VaultSetup onVaultSetup={handleVaultSetup} />
{:else}
  <main class="app">
    <header class="header">
      <div class="header-content">
        <div class="title-section">
          <h1>📝 ToDo</h1>
          <p class="subtitle">Organized in your vault</p>
        </div>
        <div class="header-actions">
          <div class="vault-info">
            <span class="vault-path">{vaultConfig?.vault_path}</span>
          </div>
          <button class="settings-btn" onclick={() => showSettings = true} title="Settings">
            ⚙️
          </button>
        </div>
      </div>
    </header>

    <div class="todo-container">
      <!-- Add Todo Form -->
      <form class="add-todo-form" onsubmit={addTodo}>
        <div class="input-group">
          <input
            type="text"
            placeholder="What needs to be done?"
            bind:value={newTodoText}
            class="todo-input"
            disabled={isLoading}
          />
          <button type="submit" class="add-btn" disabled={isLoading || !newTodoText.trim()}>
            <span class="add-icon">+</span>
            Add
          </button>
        </div>
      </form>

      <!-- Stats Bar -->
      <div class="stats-bar">
        <div class="stats">
          <span class="stat">
            <span class="stat-number">{totalCount}</span>
            <span class="stat-label">Total</span>
          </span>
          <span class="stat">
            <span class="stat-number">{activeCount}</span>
            <span class="stat-label">Active</span>
          </span>
          <span class="stat">
            <span class="stat-number">{completedCount}</span>
            <span class="stat-label">Done</span>
          </span>
        </div>

        <!-- Filters -->
        <div class="filters">
          <button
            class="filter-btn"
            class:active={filter === "all"}
            onclick={() => filter = "all"}
          >
            All
          </button>
          <button
            class="filter-btn"
            class:active={filter === "active"}
            onclick={() => filter = "active"}
          >
            Active
          </button>
          <button
            class="filter-btn"
            class:active={filter === "completed"}
            onclick={() => filter = "completed"}
          >
            Completed
          </button>
        </div>
      </div>

      <!-- Todo List -->
      <div class="todo-list">
        {#if isLoading}
          <div class="loading">Loading todos...</div>
        {:else if filteredTodos.length === 0}
          <div class="empty-state">
            {#if filter === "all"}
              <p>🌟 No todos yet. Add your first task above!</p>
            {:else if filter === "active"}
              <p>🎉 No active tasks. You're all caught up!</p>
            {:else}
              <p>📋 No completed tasks yet.</p>
            {/if}
          </div>
        {:else}
          {#each filteredTodos as todo (todo.id)}
            <div class="todo-item" class:completed={todo.completed}>
              <label class="todo-checkbox">
                <input
                  type="checkbox"
                  checked={todo.completed}
                  onchange={() => toggleTodo(todo.id)}
                />
                <span class="checkmark"></span>
              </label>
              
              <div class="todo-content">
                <span class="todo-text" class:completed={todo.completed}>
                  {todo.text}
                </span>
                <span class="todo-date">{formatDate(todo.created_at)}</span>
              </div>
              
              <button
                class="delete-btn"
                onclick={() => deleteTodo(todo.id)}
                title="Delete todo"
              >
                <span class="delete-icon">×</span>
              </button>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </main>

  <SettingsModal bind:isOpen={showSettings} onVaultChanged={handleVaultChanged} />
{/if}

<style>
  :root {
    --primary-color: #667eea;
    --primary-dark: #5a67d8;
    --success-color: #48bb78;
    --danger-color: #f56565;
    --warning-color: #ed8936;
    
    --text-primary: #2d3748;
    --text-secondary: #718096;
    --text-muted: #a0aec0;
    
    --bg-primary: #ffffff;
    --bg-secondary: #f7fafc;
    --bg-card: #ffffff;
    
    --border-color: #e2e8f0;
    --border-hover: #cbd5e0;
    
    --shadow-sm: 0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06);
    --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
    --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
    
    --radius: 0.5rem;
    --radius-sm: 0.25rem;
    --radius-lg: 0.75rem;
  }

  * {
    box-sizing: border-box;
  }

  .app {
    min-height: 100vh;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    padding: 2rem 1rem;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  .header {
    color: white;
    margin-bottom: 2rem;
  }

  .header-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
    max-width: 600px;
    margin: 0 auto;
  }

  .title-section h1 {
    font-size: 3rem;
    font-weight: 700;
    margin: 0;
    text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .subtitle {
    font-size: 1.1rem;
    opacity: 0.9;
    margin: 0.5rem 0 0 0;
    font-weight: 300;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .vault-info {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    font-size: 0.75rem;
    opacity: 0.8;
  }

  .vault-path {
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    background: rgba(255, 255, 255, 0.1);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .settings-btn {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: white;
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 50%;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.25rem;
  }

  .settings-btn:hover {
    background: rgba(255, 255, 255, 0.2);
    border-color: rgba(255, 255, 255, 0.3);
    transform: scale(1.05);
  }

  .loading-screen {
    min-height: 100vh;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  .loading-content {
    text-align: center;
    color: white;
  }

  .loading-content p {
    margin-top: 1rem;
    font-size: 1.125rem;
    opacity: 0.9;
  }

  .spinner {
    width: 3rem;
    height: 3rem;
    border: 3px solid rgba(255, 255, 255, 0.3);
    border-top: 3px solid white;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .todo-container {
    max-width: 600px;
    margin: 0 auto;
    background: var(--bg-card);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  .add-todo-form {
    padding: 1.5rem;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .input-group {
    display: flex;
    gap: 0.75rem;
  }

  .todo-input {
    flex: 1;
    padding: 0.75rem 1rem;
    border: 2px solid var(--border-color);
    border-radius: var(--radius);
    font-size: 1rem;
    background: var(--bg-primary);
    color: var(--text-primary);
    transition: all 0.2s ease;
  }

  .todo-input:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
  }

  .todo-input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .add-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.5rem;
    background: var(--primary-color);
    color: white;
    border: none;
    border-radius: var(--radius);
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .add-btn:hover:not(:disabled) {
    background: var(--primary-dark);
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }

  .add-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
  }

  .add-icon {
    font-size: 1.2rem;
    font-weight: 300;
  }

  .stats-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.5rem;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .stats {
    display: flex;
    gap: 1.5rem;
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .stat-number {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-top: 0.25rem;
  }

  .filters {
    display: flex;
    gap: 0.5rem;
  }

  .filter-btn {
    padding: 0.5rem 1rem;
    background: transparent;
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .filter-btn:hover {
    background: var(--bg-primary);
    border-color: var(--border-hover);
  }

  .filter-btn.active {
    background: var(--primary-color);
    color: white;
    border-color: var(--primary-color);
  }

  .todo-list {
    max-height: 400px;
    overflow-y: auto;
  }

  .loading,
  .empty-state {
    padding: 3rem 1.5rem;
    text-align: center;
    color: var(--text-secondary);
  }

  .empty-state p {
    font-size: 1.1rem;
    margin: 0;
  }

  .todo-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--border-color);
    transition: all 0.2s ease;
  }

  .todo-item:hover {
    background: var(--bg-secondary);
  }

  .todo-item.completed {
    opacity: 0.7;
  }

  .todo-checkbox {
    position: relative;
    cursor: pointer;
    display: flex;
    align-items: center;
  }

  .todo-checkbox input[type="checkbox"] {
    position: absolute;
    opacity: 0;
    cursor: pointer;
  }

  .checkmark {
    width: 1.25rem;
    height: 1.25rem;
    border: 2px solid var(--border-hover);
    border-radius: var(--radius-sm);
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .checkmark::after {
    content: "✓";
    color: white;
    font-size: 0.875rem;
    font-weight: 700;
    opacity: 0;
    transform: scale(0);
    transition: all 0.2s ease;
  }

  .todo-checkbox input:checked + .checkmark {
    background: var(--success-color);
    border-color: var(--success-color);
  }

  .todo-checkbox input:checked + .checkmark::after {
    opacity: 1;
    transform: scale(1);
  }

  .todo-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .todo-text {
    font-size: 1rem;
    color: var(--text-primary);
    line-height: 1.4;
    transition: all 0.2s ease;
  }

  .todo-text.completed {
    text-decoration: line-through;
    color: var(--text-muted);
  }

  .todo-date {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .delete-btn {
    width: 2rem;
    height: 2rem;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }

  .delete-btn:hover {
    background: var(--danger-color);
    color: white;
    transform: scale(1.1);
  }

  .delete-icon {
    font-size: 1.25rem;
    font-weight: 300;
    line-height: 1;
  }

  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    :root {
      --text-primary: #f7fafc;
      --text-secondary: #a0aec0;
      --text-muted: #718096;
      
      --bg-primary: #2d3748;
      --bg-secondary: #4a5568;
      --bg-card: #2d3748;
      
      --border-color: #4a5568;
      --border-hover: #718096;
    }
  }

  /* Mobile responsiveness */
  @media (max-width: 640px) {
    .app {
      padding: 1rem 0.5rem;
    }

    .title-section h1 {
      font-size: 2rem;
    }

    .header-content {
      flex-direction: column;
      gap: 1rem;
      text-align: center;
    }

    .header-actions {
      flex-direction: column-reverse;
      align-items: center;
      gap: 0.75rem;
    }

    .vault-info {
      align-items: center;
    }

    .vault-path {
      max-width: 250px;
    }

    .stats-bar {
      flex-direction: column;
      gap: 1rem;
      padding: 1rem;
    }

    .input-group {
      flex-direction: column;
    }

    .add-btn {
      justify-content: center;
    }

    .filters {
      justify-content: center;
    }

    .todo-item {
      padding: 0.75rem 1rem;
    }
  }
</style>
