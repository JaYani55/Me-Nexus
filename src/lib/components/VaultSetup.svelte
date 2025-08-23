<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface VaultInfo {
    path: string;
    is_empty: boolean;
    exists: boolean;
  }

  interface VaultConfig {
    vault_path: string;
    created_at: string;
  }

  let { onVaultSetup } = $props<{
    onVaultSetup: (config: VaultConfig) => void;
  }>();

  let currentStep = $state<"welcome" | "select" | "confirm">("welcome");
  let selectedPath = $state("");
  let directoryInfo = $state<VaultInfo | null>(null);
  let isLoading = $state(false);
  let showWarning = $state(false);

  async function openDirectoryPicker() {
    try {
      const path = await invoke<string | null>("open_directory_dialog");
      if (path) {
        selectedPath = path;
        await checkDirectory(path);
        currentStep = "confirm";
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

  async function setupVault() {
    if (!selectedPath) return;

    try {
      isLoading = true;
      const config = await invoke<VaultConfig>("set_vault_path", { 
        vaultPath: selectedPath 
      });
      onVaultSetup(config);
    } catch (error) {
      console.error("Failed to set up vault:", error);
    } finally {
      isLoading = false;
    }
  }

  function goBack() {
    if (currentStep === "confirm") {
      currentStep = "select";
      selectedPath = "";
      directoryInfo = null;
      showWarning = false;
    } else if (currentStep === "select") {
      currentStep = "welcome";
    }
  }
</script>

<div class="vault-setup">
  <div class="setup-container">
    {#if currentStep === "welcome"}
      <div class="welcome-step">
        <div class="icon">🗂️</div>
        <h1>Welcome to Personal Nexus</h1>
        <p class="subtitle">A vault-based personal organization system</p>
        
        <div class="features">
          <div class="feature">
            <span class="feature-icon">📁</span>
            <div>
              <h3>Vault-Based Storage</h3>
              <p>All your data is stored in a folder you choose on your system</p>
            </div>
          </div>
          <div class="feature">
            <span class="feature-icon">🔒</span>
            <div>
              <h3>Local & Private</h3>
              <p>Your data stays on your computer, completely under your control</p>
            </div>
          </div>
          <div class="feature">
            <span class="feature-icon">📋</span>
            <div>
              <h3>Organized Structure</h3>
              <p>Each data type gets its own folder within your vault</p>
            </div>
          </div>
        </div>

        <button class="primary-btn" onclick={() => currentStep = "select"}>
          Get Started
        </button>
      </div>
    {:else if currentStep === "select"}
      <div class="select-step">
        <button class="back-btn" onclick={goBack}>← Back</button>
        
        <h2>Choose Your Vault Location</h2>
        <p>Select a folder where Personal Nexus will store all your data.</p>
        
        <div class="recommendations">
          <h3>💡 Recommendations:</h3>
          <ul>
            <li>Choose a location you can easily back up</li>
            <li>Use an empty folder or create a new one</li>
            <li>Avoid system folders or program directories</li>
            <li>Consider cloud storage folders for sync across devices</li>
          </ul>
        </div>

        <button class="select-folder-btn" onclick={openDirectoryPicker}>
          <span class="folder-icon">📁</span>
          Select Folder
        </button>
      </div>
    {:else if currentStep === "confirm"}
      <div class="confirm-step">
        <button class="back-btn" onclick={goBack}>← Back</button>
        
        <h2>Confirm Vault Setup</h2>
        
        <div class="selected-path">
          <h3>Selected Location:</h3>
          <div class="path-display">
            <span class="path-text">{selectedPath}</span>
          </div>
        </div>

        {#if directoryInfo}
          <div class="directory-status">
            {#if !directoryInfo.exists}
              <div class="status error">
                <span class="status-icon">❌</span>
                <div>
                  <strong>Directory does not exist</strong>
                  <p>Please select a valid directory.</p>
                </div>
              </div>
            {:else if showWarning}
              <div class="status warning">
                <span class="status-icon">⚠️</span>
                <div>
                  <strong>Directory is not empty</strong>
                  <p>This folder contains existing files. Personal Nexus will create its structure alongside your existing files.</p>
                </div>
              </div>
            {:else}
              <div class="status success">
                <span class="status-icon">✅</span>
                <div>
                  <strong>Perfect choice!</strong>
                  <p>This empty directory is ready for your vault.</p>
                </div>
              </div>
            {/if}
          </div>
        {/if}

        <div class="vault-structure">
          <h3>What will be created:</h3>
          <div class="structure-preview">
            <div class="folder-item">
              <span class="folder-icon">📁</span>
              <span class="folder-name">{selectedPath.split('\\').pop() || 'Vault'}</span>
            </div>
            <div class="folder-item nested">
              <span class="folder-icon">📋</span>
              <span class="folder-name">Todo</span>
              <span class="folder-desc">- Todo management</span>
            </div>
            <div class="folder-item nested">
              <span class="folder-icon">⚙️</span>
              <span class="folder-name">.nexus</span>
              <span class="folder-desc">- App metadata</span>
            </div>
          </div>
        </div>

        {#if directoryInfo?.exists}
          <button 
            class="setup-btn" 
            onclick={setupVault}
            disabled={isLoading}
          >
            {#if isLoading}
              <span class="spinner"></span>
              Setting up vault...
            {:else}
              Create Vault
            {/if}
          </button>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .vault-setup {
    min-height: 100vh;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  .setup-container {
    background: white;
    border-radius: 16px;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
    max-width: 600px;
    width: 100%;
    overflow: hidden;
  }

  .welcome-step,
  .select-step,
  .confirm-step {
    padding: 3rem 2rem;
    text-align: center;
  }

  .icon {
    font-size: 4rem;
    margin-bottom: 1rem;
  }

  .welcome-step h1 {
    font-size: 2.5rem;
    font-weight: 700;
    color: #1f2937;
    margin: 0 0 0.5rem 0;
  }

  .subtitle {
    font-size: 1.25rem;
    color: #6b7280;
    margin: 0 0 2rem 0;
  }

  .features {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    margin: 2rem 0;
    text-align: left;
  }

  .feature {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
  }

  .feature-icon {
    font-size: 2rem;
    flex-shrink: 0;
  }

  .feature h3 {
    font-size: 1.125rem;
    font-weight: 600;
    color: #1f2937;
    margin: 0 0 0.25rem 0;
  }

  .feature p {
    font-size: 0.875rem;
    color: #6b7280;
    margin: 0;
    line-height: 1.5;
  }

  .primary-btn,
  .select-folder-btn,
  .setup-btn {
    background: #667eea;
    color: white;
    border: none;
    padding: 1rem 2rem;
    border-radius: 12px;
    font-size: 1.125rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }

  .primary-btn:hover,
  .select-folder-btn:hover,
  .setup-btn:hover:not(:disabled) {
    background: #5a67d8;
    transform: translateY(-2px);
    box-shadow: 0 10px 25px -5px rgba(102, 126, 234, 0.4);
  }

  .back-btn {
    background: none;
    border: none;
    color: #6b7280;
    font-size: 1rem;
    cursor: pointer;
    margin-bottom: 1rem;
    padding: 0.5rem;
    border-radius: 6px;
    transition: all 0.2s ease;
    align-self: flex-start;
  }

  .back-btn:hover {
    background: #f3f4f6;
    color: #374151;
  }

  .select-step h2,
  .confirm-step h2 {
    font-size: 2rem;
    font-weight: 700;
    color: #1f2937;
    margin: 0 0 0.5rem 0;
  }

  .select-step p {
    color: #6b7280;
    margin: 0 0 2rem 0;
    font-size: 1.125rem;
  }

  .recommendations {
    background: #f8faff;
    border: 1px solid #e0e7ff;
    border-radius: 12px;
    padding: 1.5rem;
    margin: 2rem 0;
    text-align: left;
  }

  .recommendations h3 {
    color: #1f2937;
    margin: 0 0 1rem 0;
    font-size: 1.125rem;
    font-weight: 600;
  }

  .recommendations ul {
    margin: 0;
    padding-left: 1.5rem;
    color: #4b5563;
  }

  .recommendations li {
    margin-bottom: 0.5rem;
    line-height: 1.5;
  }

  .folder-icon {
    font-size: 1.5rem;
  }

  .selected-path {
    background: #f8faff;
    border: 1px solid #e0e7ff;
    border-radius: 12px;
    padding: 1.5rem;
    margin: 1.5rem 0;
    text-align: left;
  }

  .selected-path h3 {
    color: #1f2937;
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .path-display {
    background: white;
    padding: 0.75rem;
    border-radius: 6px;
    border: 1px solid #d1d5db;
  }

  .path-text {
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 0.875rem;
    color: #374151;
    word-break: break-all;
  }

  .directory-status {
    margin: 1.5rem 0;
  }

  .status {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    padding: 1rem;
    border-radius: 8px;
    text-align: left;
  }

  .status.success {
    background: #d1fae5;
    border: 1px solid #10b981;
  }

  .status.warning {
    background: #fef3cd;
    border: 1px solid #f59e0b;
  }

  .status.error {
    background: #fee2e2;
    border: 1px solid #ef4444;
  }

  .status-icon {
    font-size: 1.25rem;
    flex-shrink: 0;
  }

  .status strong {
    display: block;
    margin-bottom: 0.25rem;
    font-weight: 600;
  }

  .status.success strong {
    color: #065f46;
  }

  .status.warning strong {
    color: #92400e;
  }

  .status.error strong {
    color: #991b1b;
  }

  .status p {
    margin: 0;
    font-size: 0.875rem;
    line-height: 1.4;
  }

  .status.success p {
    color: #047857;
  }

  .status.warning p {
    color: #b45309;
  }

  .status.error p {
    color: #dc2626;
  }

  .vault-structure {
    background: #f8faff;
    border: 1px solid #e0e7ff;
    border-radius: 12px;
    padding: 1.5rem;
    margin: 1.5rem 0;
    text-align: left;
  }

  .vault-structure h3 {
    color: #1f2937;
    margin: 0 0 1rem 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .structure-preview {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .folder-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: #374151;
  }

  .folder-item.nested {
    margin-left: 1.5rem;
  }

  .folder-name {
    font-weight: 500;
  }

  .folder-desc {
    color: #6b7280;
    font-style: italic;
  }

  .setup-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
  }

  .spinner {
    width: 1rem;
    height: 1rem;
    border: 2px solid transparent;
    border-top: 2px solid currentColor;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* Mobile responsiveness */
  @media (max-width: 640px) {
    .vault-setup {
      padding: 1rem;
    }

    .welcome-step,
    .select-step,
    .confirm-step {
      padding: 2rem 1.5rem;
    }

    .welcome-step h1 {
      font-size: 2rem;
    }

    .features {
      gap: 1rem;
    }

    .feature {
      flex-direction: column;
      text-align: center;
      gap: 0.5rem;
    }
  }

  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    .setup-container {
      background: #1f2937;
      color: #f9fafb;
    }

    .welcome-step h1,
    .select-step h2,
    .confirm-step h2,
    .feature h3,
    .recommendations h3,
    .selected-path h3,
    .vault-structure h3 {
      color: #f9fafb;
    }

    .subtitle,
    .select-step p {
      color: #d1d5db;
    }

    .feature p,
    .recommendations li {
      color: #9ca3af;
    }

    .back-btn {
      color: #d1d5db;
    }

    .back-btn:hover {
      background: #374151;
      color: #f3f4f6;
    }

    .recommendations,
    .selected-path,
    .vault-structure {
      background: #374151;
      border-color: #4b5563;
    }

    .path-display {
      background: #1f2937;
      border-color: #6b7280;
    }

    .path-text {
      color: #e5e7eb;
    }
  }
</style>
