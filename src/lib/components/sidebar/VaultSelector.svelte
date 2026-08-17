<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import * as prefsBridge from "$lib/bridge/prefs";
  import { nativeDialog } from "$lib/stores/nativeDialog.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { errorMessage } from "$lib/utils/errorMessage";
  import Icon from "../ui/Icon.svelte";

  const MAX_RECENT = 4;

  let root: HTMLElement | undefined = $state();
  let menuOpen = $state(false);
  let recentVaults = $state<string[]>([]);

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
      const result = await nativeDialog.during(() =>
        open({
          multiple: false,
          filters: [{ name: "KeePass Database", extensions: ["kdbx"] }],
        }),
      );
      if (result) switchTo(result);
    } catch (e) {
      toast.error(errorMessage(e));
    }
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
    <Icon name="database" size={15} />
    <span class="vault-name">{currentName}</span>
    <Icon name="chevron-down" size={13} />
  </button>

  {#if menuOpen}
    <div class="vault-menu" role="menu" aria-label="Vault actions">
      <button type="button" class="menu-item" role="menuitem" onclick={lockVault}>
        <Icon name="lock" size={15} />
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
            <Icon name="database" size={15} />
            <span>{vaultName(path)}</span>
          </button>
        {/each}
      {/if}

      <div class="menu-separator" role="separator"></div>
      <button type="button" class="menu-item" role="menuitem" onclick={openOtherVault}>
        <Icon name="folder-open" size={15} />
        <span>Open other vault…</span>
      </button>
    </div>
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
    gap: 7px;
    width: 100%;
    padding: 7px 8px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
    text-align: left;
    transition: background var(--transition-fast);
  }

  .vault-button:hover,
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
    padding: 4px;
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-dialog);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 12.5px;
    text-align: left;
  }

  .menu-item span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .menu-item:hover {
    background: var(--bg-accent);
    color: var(--text-primary);
  }

  .menu-separator {
    height: 0.5px;
    margin: 4px 6px;
    background: var(--border);
  }

  .menu-heading {
    padding: 4px 8px 2px;
    color: var(--text-muted);
    font-size: 11.5px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
</style>
