<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { app } from "$lib/stores/app.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import * as vaultBridge from "$lib/bridge/vault";
  import * as entriesBridge from "$lib/bridge/entries";
  import Icon from "../ui/Icon.svelte";
  import PasswordDialog from "../ui/PasswordDialog.svelte";

  let dialog:
    | { kind: "open" }
    | { kind: "create" }
    | { kind: "change-password" }
    | { kind: "new-password" }
    | null = $state(null);

  let statusMsg = $state("");

  async function handleOpen() {
    try {
      const result = await open({
        multiple: false,
        filters: [{ name: "KeePass Database", extensions: ["kdbx"] }],
      });
      if (!result) return;
      selectedPath = result;
      dialog = { kind: "open" };
    } catch (e) {
      statusMsg = String(e);
    }
  }

  let selectedPath = $state("");

  async function doOpen(password: string) {
    dialog = null;
    try {
      const meta = await vaultBridge.vaultOpen(selectedPath, password);
      vault.setMeta(meta);

      const summaries = await entriesBridge.entriesList();
      const fullEntries = await Promise.all(summaries.map((s) => entriesBridge.entryGet(s.id)));
      vault.setEntries(fullEntries);

      app.view = "main";
    } catch (e) {
      statusMsg = String(e);
    }
  }

  async function handleCreate(password: string) {
    dialog = null;
    try {
      const result = await save({
        filters: [{ name: "KeePass Database", extensions: ["kdbx"] }],
        defaultPath: "vault.kdbx",
      });
      if (!result) return;

      const meta = await vaultBridge.vaultCreate(result, password, "");
      vault.setMeta(meta);
      vault.setEntries([]);
      app.view = "main";
    } catch (e) {
      statusMsg = String(e);
    }
  }

  let pendingOldPw = $state("");

  async function handleChangePassword(oldPassword: string) {
    pendingOldPw = oldPassword;
    dialog = { kind: "new-password" };
  }

  async function handleSetNewPassword(newPassword: string) {
    dialog = null;
    if (newPassword.length < 4) {
      statusMsg = "New password must be at least 4 characters";
      return;
    }
    try {
      await vaultBridge.vaultChangePassword(pendingOldPw, newPassword);
      statusMsg = "Password changed successfully";
    } catch (e) {
      statusMsg = String(e);
    }
  }
</script>

<div class="settings-overlay" role="dialog" aria-label="Settings">
  {#if dialog}
    {#if dialog.kind === "open"}
      <PasswordDialog
        title="Open vault"
        confirmLabel="Open"
        onconfirm={doOpen}
        oncancel={() => (dialog = null)}
      />
    {:else if dialog.kind === "create"}
      <PasswordDialog
        title="Create new vault"
        confirmLabel="Create"
        onconfirm={handleCreate}
        oncancel={() => (dialog = null)}
      />
    {:else if dialog.kind === "change-password"}
      <PasswordDialog
        title="Current master password"
        confirmLabel="Next"
        onconfirm={handleChangePassword}
        oncancel={() => (dialog = null)}
      />
    {:else if dialog.kind === "new-password"}
      <PasswordDialog
        title="New master password"
        confirmLabel="Change"
        onconfirm={handleSetNewPassword}
        oncancel={() => (dialog = null)}
      />
    {/if}
  {/if}

  <div class="settings-pane">
    <header class="settings-header">
      <h1 class="settings-title">Settings</h1>
      <button class="close-btn" onclick={() => (app.view = "main")} aria-label="Close settings">
        <Icon name="x" size={16} />
      </button>
    </header>

    <div class="settings-body">
      <section class="settings-section">
        <h2 class="section-heading">Vault</h2>

        <div class="vault-info">
          <span class="vault-label">Current vault</span>
          {#if vault.meta}
            <span class="vault-path">{vault.meta.name} — {vault.meta.itemCount} items</span>
            <span class="vault-path-sub">{vault.meta.path}</span>
          {:else}
            <span class="vault-none">No vault open</span>
          {/if}
        </div>

        <div class="settings-actions">
          <button class="settings-btn" onclick={handleOpen}>
            <Icon name="folder-open" size={14} />
            Open vault…
          </button>
          <button class="settings-btn" onclick={() => (dialog = { kind: "create" })}>
            <Icon name="plus" size={14} />
            Create new vault…
          </button>
          {#if vault.meta}
            <button class="settings-btn" onclick={() => (dialog = { kind: "change-password" })}>
              <Icon name="exchange" size={14} />
              Change master password…
            </button>
          {/if}
        </div>

        {#if statusMsg}
          <span class="status-msg">{statusMsg}</span>
        {/if}
      </section>

      <section class="settings-section">
        <h2 class="section-heading">Recent vaults</h2>
        <p class="empty-text">No recent vaults.</p>
      </section>

      <section class="settings-section">
        <h2 class="section-heading">Security</h2>
        <div class="setting-row">
          <span class="setting-label">Lock on idle</span>
          <span class="setting-value">5 minutes</span>
        </div>
        <div class="setting-row">
          <span class="setting-label">Clipboard clear</span>
          <span class="setting-value">15 seconds</span>
        </div>
      </section>
    </div>

    <footer class="settings-footer">
      <span class="version">Kagi 0.1.0</span>
    </footer>
  </div>
</div>

<style>
  .settings-overlay {
    position: absolute;
    inset: 0;
    background: var(--surface-0);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .settings-pane {
    width: 520px;
    max-height: 80vh;
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-card);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 0.5px solid var(--border);
  }

  .settings-title {
    font-size: 18px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }

  .close-btn:hover {
    background: var(--border);
  }

  .settings-body {
    padding: 20px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .section-heading {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .vault-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 12px;
    background: var(--surface-1);
    border-radius: var(--radius);
  }

  .vault-label {
    font-size: 11px;
    color: var(--text-muted);
  }

  .vault-path {
    font-size: 13.5px;
    color: var(--text-primary);
  }

  .vault-path-sub {
    font-size: 11.5px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .vault-none {
    font-size: 13.5px;
    color: var(--text-muted);
    font-style: italic;
  }

  .settings-actions {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .settings-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 13px;
    color: var(--text-primary);
    background: var(--surface-1);
    transition: background 0.1s;
  }

  .settings-btn:hover {
    background: var(--border);
  }

  .status-msg {
    font-size: 12px;
    color: var(--danger);
    padding: 4px 0;
  }

  .empty-text {
    font-size: 13px;
    color: var(--text-muted);
    font-style: italic;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0;
  }

  .setting-label {
    font-size: 13px;
    color: var(--text-primary);
  }

  .setting-value {
    font-size: 13px;
    color: var(--text-muted);
  }

  .settings-footer {
    padding: 10px 20px;
    border-top: 0.5px solid var(--border);
    display: flex;
    justify-content: center;
  }

  .version {
    font-size: 11.5px;
    color: var(--text-muted);
  }
</style>
