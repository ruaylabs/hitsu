<script lang="ts">
  import { onMount } from "svelte";
  import * as prefsBridge from "$lib/bridge/prefs";
  import { selection } from "$lib/stores/selection.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { errorMessage } from "$lib/utils/errorMessage";
  import { pickVaultToCreate, pickVaultToOpen } from "$lib/utils/vaultFilePicker";
  import Icon from "../ui/Icon.svelte";
  import PasswordDialog from "../ui/PasswordDialog.svelte";

  const MAX_RECENT = 4;

  let root: HTMLElement | undefined = $state();
  let menuOpen = $state(false);
  let recentVaults = $state<string[]>([]);
  let createDialog = $state(false);
  let createPath = $state("");

  let currentPath = $derived(vault.meta?.path ?? "");
  let currentName = $derived(vault.meta?.name || vaultName(currentPath));
  let otherVaults = $derived(
    recentVaults.filter((path) => path !== currentPath).slice(0, MAX_RECENT),
  );

  function vaultName(path: string) {
    return path.split(/[\\/]/).pop() || path;
  }

  onMount(() => {
    prefsBridge
      .prefsGet()
      .then((prefs) => (recentVaults = prefs.recentVaults ?? []))
      .catch((e) => console.error("Failed to load recent vaults", e));
  });

  function closeMenu() {
    menuOpen = false;
  }

  function switchTo(path: string) {
    closeMenu();
    // Locking drops the decrypted vault, so run it through the navigation
    // guard: an in-progress edit gets the save/discard prompt first.
    selection.requestNavigation(() => void vault.switchTo(path));
  }

  function lockVault() {
    closeMenu();
    selection.requestNavigation(() => void vault.lock());
  }

  async function openOtherVault() {
    closeMenu();
    try {
      const result = await pickVaultToOpen();
      if (result) switchTo(result);
    } catch (e) {
      toast.error(errorMessage(e));
    }
  }

  function createVault() {
    closeMenu();
    // Resolve in-progress edits before choosing a path and asking for a new
    // master password. This leaves doCreate free to await the complete
    // operation while PasswordDialog displays its pending and error states.
    selection.requestNavigation(() => void chooseCreatePath());
  }

  async function chooseCreatePath() {
    try {
      const result = await pickVaultToCreate();
      if (!result) return;
      createPath = result;
      createDialog = true;
    } catch (e) {
      toast.error(errorMessage(e));
    }
  }

  async function doCreate(password: string) {
    // vault_create replaces the open backend vault atomically. Locking first
    // would briefly render the old vault's unlock dialog while the new vault
    // is being created.
    await vault.create(createPath, password);
    createDialog = false;
  }

  function onWindowPointerdown(event: PointerEvent) {
    if (!menuOpen) return;
    if (root?.contains(event.target as Node)) return;
    closeMenu();
  }

  function onWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && menuOpen) closeMenu();
  }
</script>

<svelte:window onpointerdown={onWindowPointerdown} onkeydown={onWindowKeydown} />

<div class="vault-selector" bind:this={root}>
  <button
    type="button"
    class="vault-button"
    aria-haspopup="menu"
    aria-expanded={menuOpen}
    title={currentPath}
    onclick={() => (menuOpen = !menuOpen)}
  >
    <Icon name="database" size={14} />
    <span class="vault-name">{currentName}</span>
    <Icon name="chevron-down" size={14} />
  </button>

  {#if menuOpen}
    <div class="vault-menu" role="menu" aria-label="Vault actions">
      <button type="button" class="menu-item" role="menuitem" onclick={lockVault}>
        <Icon name="lock" size={14} />
        <span>Lock vault</span>
      </button>

      {#if otherVaults.length > 0}
        <div class="menu-separator" role="separator"></div>
        <div class="menu-heading">Switch vault</div>
        {#each otherVaults as path (path)}
          <button
            type="button"
            class="menu-item"
            role="menuitem"
            title={path}
            onclick={() => switchTo(path)}
          >
            <Icon name="database" size={14} />
            <span>{vaultName(path)}</span>
          </button>
        {/each}
      {/if}

      <div class="menu-separator" role="separator"></div>
      <button type="button" class="menu-item" role="menuitem" onclick={openOtherVault}>
        <Icon name="folder-open" size={14} />
        <span>Open other vault…</span>
      </button>
      <button type="button" class="menu-item" role="menuitem" onclick={createVault}>
        <Icon name="plus" size={14} />
        <span>Create new vault…</span>
      </button>
    </div>
  {/if}

  {#if createDialog}
    <PasswordDialog
      title="Create new vault"
      vaultPath={createPath}
      confirmLabel="Create"
      confirm={true}
      showStrength={true}
      showRecoveryWarning={true}
      minStrength={1}
      onconfirm={doCreate}
      oncancel={() => (createDialog = false)}
    />
  {/if}
</div>

<style>
  .vault-selector {
    position: relative;
    margin: 0 2px 12px;
  }

  .vault-button {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-2);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: var(--text-base);
    font-weight: 500;
    text-align: left;
    transition: background var(--transition-fast);
  }

  .vault-button:hover {
    background: var(--surface-hover);
  }

  .vault-button[aria-expanded="true"] {
    background: var(--bg-accent);
    color: var(--text-primary);
  }

  .vault-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .vault-menu {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    z-index: var(--z-view-overlay);
    min-width: 200px;
    padding: var(--space-1);
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-popover);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-2);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: var(--text-base);
    text-align: left;
  }

  .menu-item span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .menu-item:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .menu-separator {
    height: 0.5px;
    margin: 4px 6px;
    background: var(--border);
  }

  .menu-heading {
    padding: var(--space-1) var(--space-2) var(--space-half);
    color: var(--text-muted);
    font-size: var(--text-sm);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
</style>
