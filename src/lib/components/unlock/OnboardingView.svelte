<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import * as prefsBridge from "$lib/bridge/prefs";
  import * as vaultBridge from "$lib/bridge/vault";
  import { vault } from "$lib/stores/vault.svelte";
  import Icon from "../ui/Icon.svelte";
  import PasswordDialog from "../ui/PasswordDialog.svelte";

  let dialog: "open" | "create" | null = $state(null);
  let pendingPath = $state("");
  let busy = $state(false);
  let error = $state("");

  async function handleOpen() {
    if (busy) return;
    error = "";
    try {
      const result = await open({
        multiple: false,
        filters: [{ name: "KeePass Database", extensions: ["kdbx"] }],
      });
      if (!result) return;
      pendingPath = result;
      dialog = "open";
    } catch (e) {
      error = String(e);
    }
  }

  async function doOpen(password: string) {
    dialog = null;
    busy = true;
    error = "";
    try {
      const meta = await vaultBridge.vaultOpen(pendingPath, password);
      vault.openVault(meta);

      prefsBridge.prefsSetLastVault(pendingPath);
    } catch (e) {
      error = String(e);
    }
    busy = false;
  }

  async function handleCreate() {
    if (busy) return;
    error = "";
    try {
      const result = await save({
        filters: [{ name: "KeePass Database", extensions: ["kdbx"] }],
        defaultPath: "vault.kdbx",
      });
      if (!result) return;
      pendingPath = result;
      dialog = "create";
    } catch (e) {
      error = String(e);
    }
  }

  async function doCreate(password: string) {
    dialog = null;
    busy = true;
    error = "";
    try {
      const meta = await vaultBridge.vaultCreate(pendingPath, password, "");
      vault.openVault(meta);

      prefsBridge.prefsSetLastVault(pendingPath);
    } catch (e) {
      error = String(e);
    }
    busy = false;
  }
</script>

{#if dialog === "open"}
  <PasswordDialog
    title="Open vault"
    confirmLabel="Open"
    onconfirm={doOpen}
    oncancel={() => (dialog = null)}
  />
{:else if dialog === "create"}
  <PasswordDialog
    title="Create new vault"
    confirmLabel="Create"
    confirm={true}
    showStrength={true}
    minStrength={1}
    onconfirm={doCreate}
    oncancel={() => (dialog = null)}
  />
{/if}

<div class="onboarding">
  <div class="onboarding-card">
    <div class="onboarding-logo">
      <Icon name="key" size={32} />
    </div>
    <h1 class="onboarding-title">Kagi</h1>
    <p class="onboarding-subtitle">Native desktop password manager</p>

    {#if error}
      <p class="onboarding-error">{error}</p>
    {/if}

    <div class="onboarding-actions">
      <button class="onboarding-btn" onclick={handleOpen} disabled={busy}>
        <Icon name="folder-open" size={18} />
        <span>Open existing vault…</span>
      </button>
      <button class="onboarding-btn" onclick={handleCreate} disabled={busy}>
        <Icon name="plus" size={18} />
        <span>Create new vault…</span>
      </button>
    </div>
  </div>
</div>

<style>
  .onboarding {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-0);
  }

  .onboarding-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 40px 48px;
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-card);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
    max-width: 340px;
  }

  .onboarding-logo {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 56px;
    height: 56px;
    border-radius: 14px;
    background: var(--accent);
    color: #fff;
  }

  .onboarding-title {
    font-size: 22px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .onboarding-subtitle {
    font-size: 13.5px;
    color: var(--text-muted);
    text-align: center;
    margin-bottom: 8px;
  }

  .onboarding-error {
    font-size: 12px;
    color: var(--danger);
    text-align: center;
  }

  .onboarding-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
  }

  .onboarding-btn {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 12px 18px;
    border: 0.5px solid var(--border-strong);
    border-radius: var(--radius);
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    background: var(--surface-1);
    transition: background 0.1s;
  }

  .onboarding-btn:hover:not(:disabled) {
    background: var(--bg-accent);
    border-color: var(--accent);
    color: var(--text-accent);
  }

  .onboarding-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
