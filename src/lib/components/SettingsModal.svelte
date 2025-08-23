<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface VaultConfig {
    vault_path: string;
    created_at: string;
  }

  interface VaultInfo {
    path: string;
    is_empty: boolean;
    exists: boolean;
  }

  let { isOpen = $bindable(), onVaultChanged } = $props<{
    isOpen: boolean;
    onVaultChanged?: () => void;
  }>();

  let activeTab = $state<"vault" | "general">("vault");
  let currentVault = $state<VaultConfig | null>(null);
  let isLoading = $state(false);
  let selectedPath = $state("");
  let directoryInfo = $state<VaultInfo | null>(null);
  let showWarning = $state(false);

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
          onclick={() => activeTab = "vault"}
        >
          Vault
        </button>
        <button 
          class="tab-btn" 
          class:active={activeTab === "general"}
          onclick={() => activeTab = "general"}
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
</style>
