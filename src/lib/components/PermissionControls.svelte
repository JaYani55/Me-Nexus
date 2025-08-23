<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface Permissions {
    share_with_ai: boolean;
    share_with_cloud: boolean;
    read_only: boolean;
    expires_at: string | null;
  }

  let {
    objectId,
    permissions = $bindable(),
    disabled = false,
    size = "normal"
  } = $props<{
    objectId: number;
    permissions: Permissions;
    disabled?: boolean;
    size?: "small" | "normal" | "large";
  }>();

  let isUpdating = $state(false);
  let lastUpdateError = $state<string | null>(null);

  async function updatePermission() {
    if (isUpdating) return;
    
    try {
      isUpdating = true;
      lastUpdateError = null;
      
      await invoke("update_todo_permissions", {
        objectId,
        permissions
      });
      
      console.log("Permissions updated successfully for object:", objectId);
    } catch (error) {
      console.error("Failed to update permissions:", error);
      lastUpdateError = error as string;
    } finally {
      isUpdating = false;
    }
  }
</script>

<div class="permission-controls {size}" class:disabled>
  <div class="permissions-grid">
    <label class="permission-item" title="Allow AI services to read and process this item">
      <span class="permission-icon">🧠</span>
      <span class="permission-label">AI Access</span>
      <input
        type="checkbox"
        bind:checked={permissions.share_with_ai}
        onchange={updatePermission}
        {disabled}
        class="permission-checkbox"
      />
      <span class="checkmark"></span>
    </label>

    <label class="permission-item" title="Allow this item to be synced to your cloud account">
      <span class="permission-icon">☁️</span>
      <span class="permission-label">Cloud Sync</span>
      <input
        type="checkbox"
        bind:checked={permissions.share_with_cloud}
        onchange={updatePermission}
        {disabled}
        class="permission-checkbox"
      />
      <span class="checkmark"></span>
    </label>

    <label class="permission-item" title="Make this item read-only">
      <span class="permission-icon">🔒</span>
      <span class="permission-label">Read Only</span>
      <input
        type="checkbox"
        bind:checked={permissions.read_only}
        onchange={updatePermission}
        {disabled}
        class="permission-checkbox"
      />
      <span class="checkmark"></span>
    </label>
  </div>

  {#if isUpdating}
    <div class="update-indicator">
      <span class="spinner"></span>
      <span class="update-text">Updating...</span>
    </div>
  {/if}

  {#if lastUpdateError}
    <div class="error-message">
      <span class="error-icon">⚠️</span>
      <span class="error-text">{lastUpdateError}</span>
    </div>
  {/if}
</div>

<style>
  .permission-controls {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .permission-controls.disabled {
    opacity: 0.5;
    pointer-events: none;
  }

  .permissions-grid {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  .permission-item {
    display: flex;
    align-items: center;
    cursor: pointer;
    user-select: none;
    position: relative;
    padding: 8px 12px;
    border-radius: 8px;
    border: 1px solid #e5e7eb;
    background: #ffffff;
    transition: all 0.2s ease;
    min-width: 120px;
  }

  .permission-item:hover {
    border-color: #d1d5db;
    background: #f9fafb;
  }

  .permission-item:has(.permission-checkbox:checked) {
    border-color: #93c5fd;
    background: #eff6ff;
  }

  .permission-icon {
    margin-right: 8px;
    font-size: 16px;
  }

  .permission-label {
    flex: 1;
    font-weight: 500;
    color: #374151;
    font-size: 14px;
  }

  .permission-checkbox {
    position: absolute;
    opacity: 0;
    cursor: pointer;
  }

  .checkmark {
    width: 20px;
    height: 20px;
    border-radius: 4px;
    border: 2px solid #d1d5db;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    position: relative;
  }

  .permission-checkbox:checked + .checkmark {
    background: #3b82f6;
    border-color: #3b82f6;
    color: white;
  }

  .permission-checkbox:checked + .checkmark::after {
    content: "✓";
    font-weight: bold;
    font-size: 12px;
  }

  .permission-checkbox:focus + .checkmark {
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .update-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #6b7280;
    font-size: 12px;
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid #d1d5db;
    border-top: 2px solid #3b82f6;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  .update-text {
    font-size: 12px;
  }

  .error-message {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .error-icon {
    color: #ef4444;
  }

  .error-text {
    color: #dc2626;
    font-size: 12px;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  /* Size variations */
  .permission-controls.small .permission-item {
    padding: 6px 10px;
    min-width: 100px;
  }

  .permission-controls.small .permission-label {
    font-size: 12px;
  }

  .permission-controls.small .permission-icon {
    font-size: 14px;
  }

  .permission-controls.small .checkmark {
    width: 16px;
    height: 16px;
  }

  .permission-controls.large .permission-item {
    padding: 12px 16px;
    min-width: 140px;
  }

  .permission-controls.large .permission-label {
    font-size: 16px;
  }

  .permission-controls.large .permission-icon {
    font-size: 18px;
  }

  .permission-controls.large .checkmark {
    width: 24px;
    height: 24px;
  }

  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    .permission-item {
      border-color: #4b5563;
      background: #1f2937;
    }

    .permission-item:hover {
      border-color: #6b7280;
      background: #374151;
    }

    .permission-item:has(.permission-checkbox:checked) {
      border-color: #60a5fa;
      background: #1e3a8a;
    }

    .permission-label {
      color: #e5e7eb;
    }

    .checkmark {
      border-color: #6b7280;
    }

    .permission-checkbox:checked + .checkmark {
      background: #2563eb;
      border-color: #2563eb;
    }

    .update-indicator {
      color: #9ca3af;
    }

    .spinner {
      border-color: #4b5563;
      border-top-color: #2563eb;
    }
  }
</style>
