<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import PermissionControls from "./PermissionControls.svelte";

  interface VaultConfig {
    vault_path: string;
    created_at: string;
  }

  interface VaultInfo {
    path: string;
    is_empty: boolean;
    exists: boolean;
    has_nexus_folder: boolean;
    database_exists: boolean;
  }

  interface Permissions {
    share_with_ai: boolean;
    share_with_cloud: boolean;
    read_only: boolean;
    expires_at: string | null;
  }

  interface VaultObject {
    id: number;
    schema_name: string;
    content: any;
    permissions: Permissions;
    file_path: string | null;
    updated_at: string;
    created_at: string;
  }

  interface SyncStatus {
    is_syncing: boolean;
    last_sync: string | null;
    pending_changes: number;
    errors: string[];
  }

  interface PluginMetadata {
    name: string;
    id: string;
    version: string;
    description: string;
    author: string;
    main: string;
    permissions: {
      network: boolean;
      filesystem: boolean;
      system: boolean;
    };
    capabilities: string[];
    category: string;
    tags: string[];
  }

  interface InstalledPlugin {
    metadata: PluginMetadata;
    path: string;
    enabled: boolean;
    installed_at: string;
    last_used: string | null;
  }

  interface PluginStatus {
    plugin_id: string;
    status: string;
    last_ping: string | null;
    error_message: string | null;
  }

  let { isOpen = $bindable(), onVaultChanged } = $props<{
    isOpen: boolean;
    onVaultChanged?: () => void;
  }>();

  let activeTab = $state<"vault" | "permissions" | "plugins" | "general">("vault");
  let currentVault = $state<VaultConfig | null>(null);
  let isLoading = $state(false);
  let selectedPath = $state("");
  let directoryInfo = $state<VaultInfo | null>(null);
  let showWarning = $state(false);
  
  // Permissions tab state
  let vaultObjects = $state<VaultObject[]>([]);
  let syncStatus = $state<SyncStatus | null>(null);
  let permissionsLoading = $state(false);
  let permissionStats = $state({
    total: 0,
    aiShared: 0,
    cloudShared: 0,
    readOnly: 0
  });

  // Plugins tab state
  let installedPlugins = $state<InstalledPlugin[]>([]);
  let pluginStatuses = $state<Map<string, PluginStatus>>(new Map());
  let pluginsLoading = $state(false);
  let testingPlugin = $state<string | null>(null);

  // Load current vault config when modal opens
  $effect(() => {
    if (isOpen) {
      loadCurrentVault();
    }
  });

  async function loadCurrentVault() {
    try {
      isLoading = true;
      const config = await invoke<VaultConfig | null>("get_vault_config");
      currentVault = config;
    } catch (error) {
      console.error("Failed to load vault config:", error);
    } finally {
      isLoading = false;
    }
  }

  async function loadPermissionsData() {
    if (!currentVault) return;
    
    try {
      permissionsLoading = true;
      
      // Load all vault objects
      const objects = await invoke<VaultObject[]>("get_all_vault_objects");
      vaultObjects = objects;
      
      // Load sync status
      const status = await invoke<SyncStatus>("get_sync_status");
      syncStatus = status;
      
      // Calculate stats
      updatePermissionStats();
      
    } catch (error) {
      console.error("Failed to load permissions data:", error);
    } finally {
      permissionsLoading = false;
    }
  }

  function updatePermissionStats() {
    permissionStats = {
      total: vaultObjects.length,
      aiShared: vaultObjects.filter(obj => obj.permissions.share_with_ai).length,
      cloudShared: vaultObjects.filter(obj => obj.permissions.share_with_cloud).length,
      readOnly: vaultObjects.filter(obj => obj.permissions.read_only).length
    };
  }

  async function loadPluginsData() {
    try {
      pluginsLoading = true;
      const plugins = await invoke<InstalledPlugin[]>("discover_plugins");
      installedPlugins = plugins;
    } catch (error) {
      console.error("Failed to load plugins:", error);
    } finally {
      pluginsLoading = false;
    }
  }

  async function testPlugin(pluginId: string) {
    try {
      testingPlugin = pluginId;
      const status = await invoke<PluginStatus>("test_plugin", { pluginId });
      pluginStatuses.set(pluginId, status);
      // Force reactivity update
      pluginStatuses = new Map(pluginStatuses);
    } catch (error) {
      console.error("Failed to test plugin:", error);
      pluginStatuses.set(pluginId, {
        plugin_id: pluginId,
        status: "error",
        last_ping: new Date().toISOString(),
        error_message: error?.toString() || "Unknown error"
      });
      pluginStatuses = new Map(pluginStatuses);
    } finally {
      testingPlugin = null;
    }
  }

  function formatTimestamp(timestamp: string | null): string {
    if (!timestamp) return "Never";
    return new Date(timestamp).toLocaleString();
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case "active": return "#28a745";
      case "error": return "#dc3545";
      default: return "#6c757d";
    }
  }

  async function switchTab(newTab: "vault" | "permissions" | "plugins" | "general") {
    activeTab = newTab;
    
    if (newTab === "permissions" && currentVault && vaultObjects.length === 0) {
      await loadPermissionsData();
    } else if (newTab === "plugins" && installedPlugins.length === 0) {
      await loadPluginsData();
    }
  }

  async function openDirectoryPicker() {
    try {
      const path = await invoke<string | null>("open_directory_dialog");
      if (path) {
        selectedPath = path;
        await checkDirectory(path);
      }
    } catch (error) {
      console.error("Failed to open directory picker:", error);
    }
  }

  async function checkDirectory(path: string) {
    try {
      directoryInfo = await invoke<VaultInfo>("check_directory_info", { path });
      showWarning = directoryInfo.exists && !directoryInfo.is_empty;
    } catch (error) {
      console.error("Failed to check directory:", error);
    }
  }

  async function setVault() {
    if (!selectedPath) return;

    try {
      isLoading = true;
      const config = await invoke<VaultConfig>("set_vault_path", { 
        vaultPath: selectedPath 
      });
      currentVault = config;
      selectedPath = "";
      directoryInfo = null;
      showWarning = false;
      
      // Notify parent component
      onVaultChanged?.();
      
      // Close modal
      isOpen = false;
    } catch (error) {
      console.error("Failed to set vault:", error);
    } finally {
      isLoading = false;
    }
  }

  function closeModal() {
    isOpen = false;
    selectedPath = "";
    directoryInfo = null;
    showWarning = false;
    activeTab = "vault";
    vaultObjects = [];
    syncStatus = null;
  }

  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      closeModal();
    }
  }

  function handleEscapeKey(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      closeModal();
    }
  }
</script>

{#if isOpen}
  <div 
    class="modal-overlay" 
    onclick={handleOverlayClick} 
    onkeydown={handleEscapeKey}
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
    tabindex="-1"
  >
    <div class="modal-content">
      <div class="modal-header">
        <h2 id="modal-title">Settings</h2>
        <button class="close-btn" onclick={closeModal} aria-label="Close settings">×</button>
      </div>

      <div class="tabs">
        <button 
          class="tab-btn" 
          class:active={activeTab === "vault"}
          onclick={() => switchTab("vault")}
        >
          Vault
        </button>
        <button 
          class="tab-btn" 
          class:active={activeTab === "permissions"}
          onclick={() => switchTab("permissions")}
        >
          Permissions
        </button>
        <button 
          class="tab-btn" 
          class:active={activeTab === "plugins"}
          onclick={() => switchTab("plugins")}
        >
          Plugins
        </button>
        <button 
          class="tab-btn" 
          class:active={activeTab === "general"}
          onclick={() => switchTab("general")}
        >
          General
        </button>
      </div>

      <div class="tab-content">
        {#if activeTab === "vault"}
          <div class="vault-settings">
            <h3>Vault Configuration</h3>
            
            {#if isLoading}
              <div class="loading">Loading...</div>
            {:else if currentVault}
              <div class="current-vault">
                <h4>Current Vault</h4>
                <div class="vault-info">
                  <p><strong>Path:</strong> {currentVault.vault_path}</p>
                  <p><strong>Created:</strong> {new Date(currentVault.created_at).toLocaleString()}</p>
                </div>
              </div>
            {:else}
              <div class="no-vault">
                <p>No vault configured. Please select a vault directory.</p>
              </div>
            {/if}

            <div class="vault-selector">
              <h4>Change Vault Location</h4>
              <div class="input-group">
                <input 
                  type="text" 
                  placeholder="Select a directory..." 
                  bind:value={selectedPath}
                  readonly
                  class="path-input"
                />
                <button onclick={openDirectoryPicker} class="browse-btn">
                  Browse
                </button>
              </div>

              {#if directoryInfo}
                <div class="directory-info">
                  {#if !directoryInfo.exists}
                    <div class="warning">
                      ⚠️ Directory does not exist
                    </div>
                  {:else if showWarning}
                    <div class="warning">
                      ⚠️ Directory is not empty. Existing files may be affected.
                    </div>
                  {:else}
                    <div class="success">
                      ✅ Directory is empty and ready for vault setup
                    </div>
                  {/if}
                </div>
              {/if}

              {#if selectedPath && directoryInfo?.exists}
                <button 
                  onclick={setVault} 
                  class="set-vault-btn"
                  disabled={isLoading}
                >
                  {isLoading ? "Setting up..." : "Set as Vault"}
                </button>
              {/if}
            </div>
          </div>
        {:else if activeTab === "permissions"}
          <div class="permissions-settings">
            <h3>Permission Management</h3>
            
            {#if permissionsLoading}
              <div class="loading">Loading permissions data...</div>
            {:else}
              <div class="permissions-stats">
                <div class="stat-card">
                  <span class="stat-label">Total Objects</span>
                  <span class="stat-value">{permissionStats.total}</span>
                </div>
                <div class="stat-card">
                  <span class="stat-label">AI Shared</span>
                  <span class="stat-value">{permissionStats.aiShared}</span>
                </div>
                <div class="stat-card">
                  <span class="stat-label">Cloud Shared</span>
                  <span class="stat-value">{permissionStats.cloudShared}</span>
                </div>
                <div class="stat-card">
                  <span class="stat-label">Read Only</span>
                  <span class="stat-value">{permissionStats.readOnly}</span>
                </div>
              </div>

              <div class="objects-list">
                <h4>Vault Objects ({vaultObjects.length})</h4>
                
                {#if vaultObjects.length === 0}
                  <div class="empty-state">
                    <p>No objects found in vault. Create some todos or documents to see them here.</p>
                  </div>
                {:else}
                  <div class="objects-grid">
                    {#each vaultObjects as object}
                      <div class="object-card">
                        <div class="object-header">
                          <span class="object-id">#{object.id}</span>
                          <span class="object-schema">{object.schema_name}</span>
                        </div>
                        <div class="object-path" title={object.file_path}>
                          {object.file_path}
                        </div>
                        <div class="object-dates">
                          <small>Created: {new Date(object.created_at).toLocaleDateString()}</small>
                          <small>Updated: {new Date(object.updated_at).toLocaleDateString()}</small>
                        </div>
                        
                        <div class="permission-controls">
                          <PermissionControls 
                            objectId={object.id}
                            bind:permissions={object.permissions}
                          />
                        </div>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        {:else if activeTab === "plugins"}
          <div class="plugins-settings">
            <h3>Plugin Management</h3>
            
            {#if pluginsLoading}
              <div class="loading">Loading plugins...</div>
            {:else if installedPlugins.length === 0}
              <div class="no-plugins">
                <p>No plugins found. To install plugins, place them in the plugins directory:</p>
                <code>plugins/</code>
                <p>Each plugin should have a <code>plugin.json</code> configuration file.</p>
              </div>
            {:else}
              <div class="plugins-list">
                <h4>Installed Plugins ({installedPlugins.length})</h4>
                
                {#each installedPlugins as plugin (plugin.metadata.id)}
                  {@const status = pluginStatuses.get(plugin.metadata.id)}
                  <div class="plugin-card">
                    <div class="plugin-header">
                      <div class="plugin-info">
                        <h5>{plugin.metadata.name}</h5>
                        <span class="plugin-version">v{plugin.metadata.version}</span>
                        {#if status}
                          <span 
                            class="plugin-status" 
                            style="color: {getStatusColor(status.status)}"
                          >
                            {status.status}
                          </span>
                        {/if}
                      </div>
                      <div class="plugin-actions">
                        <button 
                          class="test-plugin-btn"
                          onclick={() => testPlugin(plugin.metadata.id)}
                          disabled={testingPlugin === plugin.metadata.id}
                        >
                          {testingPlugin === plugin.metadata.id ? "Testing..." : "Test"}
                        </button>
                      </div>
                    </div>
                    
                    <div class="plugin-details">
                      <p class="plugin-description">{plugin.metadata.description}</p>
                      <div class="plugin-meta">
                        <span><strong>Author:</strong> {plugin.metadata.author}</span>
                        <span><strong>Category:</strong> {plugin.metadata.category}</span>
                        <span><strong>Installed:</strong> {formatTimestamp(plugin.installed_at)}</span>
                        {#if plugin.last_used}
                          <span><strong>Last Used:</strong> {formatTimestamp(plugin.last_used)}</span>
                        {/if}
                      </div>
                      
                      {#if plugin.metadata.capabilities.length > 0}
                        <div class="plugin-capabilities">
                          <strong>Capabilities:</strong>
                          {#each plugin.metadata.capabilities as capability}
                            <span class="capability-tag">{capability}</span>
                          {/each}
                        </div>
                      {/if}
                      
                      {#if status && status.error_message}
                        <div class="plugin-error">
                          <strong>Error:</strong> {status.error_message}
                        </div>
                      {/if}
                      
                      {#if status && status.last_ping}
                        <div class="plugin-last-ping">
                          <strong>Last Ping:</strong> {formatTimestamp(status.last_ping)}
                        </div>
                      {/if}
                    </div>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {:else if activeTab === "general"}
          <div class="general-settings">
            <h3>General Settings</h3>
            <p>General settings will be added here in future updates.</p>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
  }

  .modal-content {
    background: white;
    border-radius: 12px;
    max-width: 600px;
    width: 100%;
    max-height: 80vh;
    overflow: auto;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid #e5e7eb;
  }

  .modal-header h2 {
    margin: 0;
    color: #1f2937;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 2rem;
    cursor: pointer;
    color: #6b7280;
    line-height: 1;
    padding: 0;
    width: 2rem;
    height: 2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: all 0.2s ease;
  }

  .close-btn:hover {
    background: #f3f4f6;
    color: #374151;
  }

  .tabs {
    display: flex;
    border-bottom: 1px solid #e5e7eb;
  }

  .tab-btn {
    flex: 1;
    padding: 1rem;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1rem;
    font-weight: 500;
    color: #6b7280;
    transition: all 0.2s ease;
    border-bottom: 2px solid transparent;
  }

  .tab-btn:hover {
    color: #374151;
    background: #f9fafb;
  }

  .tab-btn.active {
    color: #667eea;
    border-bottom-color: #667eea;
    background: #f8faff;
  }

  .tab-content {
    padding: 1.5rem;
  }

  .vault-settings h3,
  .general-settings h3 {
    margin-top: 0;
    margin-bottom: 1rem;
    color: #1f2937;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .loading {
    text-align: center;
    padding: 2rem;
    color: #6b7280;
  }

  .current-vault {
    background: #f8faff;
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1.5rem;
    border: 1px solid #e0e7ff;
  }

  .current-vault h4 {
    margin: 0 0 0.5rem 0;
    color: #667eea;
    font-size: 1rem;
    font-weight: 600;
  }

  .vault-info p {
    margin: 0.25rem 0;
    color: #4b5563;
    font-size: 0.875rem;
  }

  .no-vault {
    background: #fef3cd;
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1.5rem;
    border: 1px solid #f59e0b;
  }

  .no-vault p {
    margin: 0;
    color: #92400e;
  }

  .vault-selector h4 {
    margin: 0 0 1rem 0;
    color: #374151;
    font-size: 1rem;
    font-weight: 600;
  }

  .input-group {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .path-input {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    font-size: 0.875rem;
    background: #f9fafb;
    color: #374151;
  }

  .path-input:focus {
    outline: none;
    border-color: #667eea;
    box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
  }

  .browse-btn {
    padding: 0.75rem 1.5rem;
    background: #667eea;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .browse-btn:hover {
    background: #5a67d8;
  }

  .directory-info {
    margin-bottom: 1rem;
  }

  .warning {
    padding: 0.75rem;
    background: #fef3cd;
    border: 1px solid #f59e0b;
    border-radius: 6px;
    color: #92400e;
    font-size: 0.875rem;
  }

  .success {
    padding: 0.75rem;
    background: #d1fae5;
    border: 1px solid #10b981;
    border-radius: 6px;
    color: #065f46;
    font-size: 0.875rem;
  }

  .set-vault-btn {
    width: 100%;
    padding: 0.75rem;
    background: #667eea;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .set-vault-btn:hover:not(:disabled) {
    background: #5a67d8;
  }

  .set-vault-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Permissions Settings Styles */
  .permissions-settings {
    padding: 1.5rem;
  }

  .permissions-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .stat-card {
    background: #f8fafc;
    padding: 1rem;
    border-radius: 8px;
    border: 1px solid #e2e8f0;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .stat-label {
    font-size: 0.875rem;
    color: #64748b;
    margin-bottom: 0.5rem;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 600;
    color: #1e293b;
  }

  .objects-list h4 {
    margin: 0 0 1rem 0;
    color: #374151;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .empty-state {
    text-align: center;
    padding: 2rem;
    background: #f9fafb;
    border-radius: 8px;
    border: 1px solid #e5e7eb;
  }

  .empty-state p {
    margin: 0;
    color: #6b7280;
    font-size: 0.875rem;
  }

  .objects-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
    gap: 1rem;
  }

  .object-card {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 1rem;
    transition: box-shadow 0.2s ease;
  }

  .object-card:hover {
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  }

  .object-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .object-id {
    font-size: 0.875rem;
    font-weight: 600;
    color: #667eea;
  }

  .object-schema {
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    background: #e0e7ff;
    color: #5b21b6;
    border-radius: 4px;
    font-weight: 500;
  }

  .object-path {
    font-size: 0.875rem;
    color: #6b7280;
    margin-bottom: 0.5rem;
    word-break: break-all;
    max-height: 2.5em;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .object-dates {
    display: flex;
    justify-content: space-between;
    margin-bottom: 1rem;
  }

  .object-dates small {
    color: #9ca3af;
    font-size: 0.75rem;
  }

  .permission-controls {
    border-top: 1px solid #e5e7eb;
    padding-top: 1rem;
  }

  .general-settings p {
    color: #6b7280;
    margin: 0;
  }

  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    .modal-content {
      background: #1f2937;
      color: #f9fafb;
    }

    .modal-header {
      border-bottom-color: #374151;
    }

    .modal-header h2 {
      color: #f9fafb;
    }

    .close-btn {
      color: #9ca3af;
    }

    .close-btn:hover {
      background: #374151;
      color: #d1d5db;
    }

    .tabs {
      border-bottom-color: #374151;
    }

    .tab-btn {
      color: #9ca3af;
    }

    .tab-btn:hover {
      color: #d1d5db;
      background: #374151;
    }

    .tab-btn.active {
      background: #1e3a8a;
    }

    .current-vault {
      background: #1e3a8a;
      border-color: #3b82f6;
    }

    .vault-info p {
      color: #d1d5db;
    }

    .path-input {
      background: #374151;
      border-color: #4b5563;
      color: #f9fafb;
    }
  }

  /* Plugin Settings Styles */
  .plugins-settings {
    padding: 1rem;
  }

  .no-plugins {
    text-align: center;
    color: #6c757d;
    padding: 2rem;
    background: #f8f9fa;
    border-radius: 8px;
    border: 2px dashed #dee2e6;
  }

  .no-plugins code {
    background: #e9ecef;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-family: monospace;
  }

  .plugins-list h4 {
    margin-bottom: 1rem;
    color: #495057;
  }

  .plugin-card {
    border: 1px solid #dee2e6;
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
    background: #fff;
  }

  .plugin-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 0.75rem;
  }

  .plugin-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .plugin-info h5 {
    margin: 0;
    font-size: 1.1rem;
    color: #212529;
  }

  .plugin-version {
    background: #e9ecef;
    color: #495057;
    padding: 0.25rem 0.5rem;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .plugin-status {
    font-weight: 500;
    font-size: 0.875rem;
  }

  .test-plugin-btn {
    background: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    padding: 0.375rem 0.75rem;
    cursor: pointer;
    font-size: 0.875rem;
    transition: background-color 0.2s;
  }

  .test-plugin-btn:hover:not(:disabled) {
    background: #0056b3;
  }

  .test-plugin-btn:disabled {
    background: #6c757d;
    cursor: not-allowed;
  }

  .plugin-details {
    border-top: 1px solid #f1f3f4;
    padding-top: 0.75rem;
  }

  .plugin-description {
    margin: 0 0 0.75rem 0;
    color: #495057;
    line-height: 1.4;
  }

  .plugin-meta {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 0.5rem;
    margin-bottom: 0.75rem;
    font-size: 0.875rem;
    color: #6c757d;
  }

  .plugin-capabilities {
    margin-bottom: 0.75rem;
    font-size: 0.875rem;
  }

  .capability-tag {
    display: inline-block;
    background: #e7f3ff;
    color: #0066cc;
    padding: 0.25rem 0.5rem;
    border-radius: 12px;
    font-size: 0.75rem;
    margin-right: 0.5rem;
    margin-top: 0.25rem;
  }

  .plugin-error {
    background: #f8d7da;
    color: #721c24;
    padding: 0.5rem;
    border-radius: 4px;
    margin-bottom: 0.5rem;
    font-size: 0.875rem;
  }

  .plugin-last-ping {
    font-size: 0.875rem;
    color: #6c757d;
  }
</style>
