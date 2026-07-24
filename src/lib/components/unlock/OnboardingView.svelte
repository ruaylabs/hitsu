<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import * as prefsBridge from "$lib/bridge/prefs";
  import { nativeDialog } from "$lib/stores/nativeDialog.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import Icon from "../ui/Icon.svelte";
  import PasswordDialog from "../ui/PasswordDialog.svelte";

  let dialog: "open" | "create" | "unlock" | null = $state(null);
  let pendingPath = $state("");
  let busy = $state(false);
  let error = $state("");

  // The remembered vault (Settings persists it on every open), so landing
  // here — e.g. after closing the locked prompt — still offers a way back.
  let lastVault = $state<string | null>(null);
  let recentVaults = $state<string[]>([]);
  let rememberedVaults = $derived.by(() => {
    const paths = lastVault ? [lastVault, ...recentVaults] : recentVaults;
    return [...new Set(paths)].slice(0, 5);
  });

  function vaultName(path: string) {
    return path.split(/[\\/]/).pop() || path;
  }

  onMount(() => {
    prefsBridge
      .prefsGet()
      .then((prefs) => {
        lastVault = prefs.lastVault ?? null;
        recentVaults = prefs.recentVaults ?? [];
      })
      .catch((e) => console.error("Failed to load preferences", e));
  });

  function requestUnlock(path: string, mode: "open" | "unlock" = "unlock") {
    if (busy) return;
    error = "";
    pendingPath = path;
    dialog = mode;
  }

  async function handleOpen() {
    if (busy) return;
    error = "";
    try {
      const result = await nativeDialog.during(() =>
        open({
          multiple: false,
          filters: [{ name: "KeePass Database", extensions: ["kdbx"] }],
        }),
      );
      if (!result) return;
      requestUnlock(result, "open");
    } catch (e) {
      error = String(e);
    }
  }

  async function doOpen(password: string) {
    dialog = null;
    busy = true;
    error = "";
    try {
      await vault.open(pendingPath, password);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
    busy = false;
  }

  async function handleCreate() {
    if (busy) return;
    error = "";
    try {
      const result = await nativeDialog.during(() =>
        save({
          filters: [{ name: "KeePass Database", extensions: ["kdbx"] }],
          defaultPath: "vault.kdbx",
        }),
      );
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
      await vault.create(pendingPath, password);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
    busy = false;
  }
</script>

{#if dialog === "open"}
  <PasswordDialog
    title="Open vault"
    vaultPath={pendingPath}
    confirmLabel="Open"
    onconfirm={doOpen}
    oncancel={() => (dialog = null)}
  />
{:else if dialog === "unlock"}
  <PasswordDialog
    title="Unlock vault"
    vaultPath={pendingPath}
    confirmLabel="Unlock"
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
    <h1 class="onboarding-title">Hitsu</h1>
    <p class="onboarding-subtitle">Native desktop password manager</p>

    {#if error}
      <p class="onboarding-error">{error}</p>
    {/if}

    {#if rememberedVaults.length > 0}
      <div class="recent-vaults">
        <span class="recent-heading">Recent vaults</span>
        {#each rememberedVaults as path (path)}
          <button
            class="recent-vault-btn"
            onclick={() => requestUnlock(path)}
            disabled={busy}
            title={path}
          >
            <Icon name="database" size={15} />
            <span>{vaultName(path)}</span>
          </button>
        {/each}
      </div>
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
    width: min(380px, calc(100vw - 32px));
    max-height: calc(100vh - 32px);
    overflow-y: auto;
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

  .recent-vaults {
    display: flex;
    flex-direction: column;
    gap: 4px;
    width: 100%;
  }

  .recent-heading {
    margin-bottom: 2px;
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .recent-vault-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 9px;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    font-size: 12.5px;
    text-align: left;
  }

  .recent-vault-btn:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--surface-1);
  }

  .recent-vault-btn span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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

  .onboarding-btn span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
