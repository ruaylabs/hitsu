<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { app } from "$lib/stores/app.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { security } from "$lib/stores/security.svelte";
  import { startIdleTimer, stopIdleTimer } from "$lib/stores/idle.svelte";
  import * as vaultBridge from "$lib/bridge/vault";
  import * as prefsBridge from "$lib/bridge/prefs";
  import type { ItemType } from "$lib/bridge/types";
  import * as entriesBridge from "$lib/bridge/entries";
  import { toSummary } from "$lib/bridge/entries";
  import StatusBar from "$lib/components/chrome/StatusBar.svelte";
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import ItemList from "$lib/components/list/ItemList.svelte";
  import ItemDetail from "$lib/components/detail/ItemDetail.svelte";
  import SettingsView from "$lib/components/settings/SettingsView.svelte";
  import CommandPalette from "$lib/components/ui/CommandPalette.svelte";

  let showCommandPalette = $state(false);

  async function onCreateEntry(type: string) {
    showCommandPalette = false;
    try {
      const entry = await entriesBridge.entryCreate(type, { title: `New ${type}` });
      vault.setEntries([...vault.entries, toSummary(entry)]);
      selection.filter = { kind: "type", type: type as ItemType };
      selection.selectedId = entry.id;
      vault.setCreatingId(entry.id);
      vault.setEditingId(entry.id);
    } catch (e) {
      console.error("Failed to create entry", e);
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (showCommandPalette) {
        showCommandPalette = false;
        return;
      }
      if (app.view === "settings") {
        app.view = "main";
        return;
      }
    }
    if ((e.metaKey || e.ctrlKey) && e.key === ",") {
      e.preventDefault();
      app.toggleSettings();
    }
    if ((e.metaKey || e.ctrlKey) && e.key === "n") {
      e.preventDefault();
      showCommandPalette = true;
    }
    if ((e.metaKey || e.ctrlKey) && e.key === "Backspace") {
      e.preventDefault();
      // Delete selected entry via detail pane's delete
      const btn = document.querySelector('[aria-label="Delete"]');
      if (btn) (btn as HTMLButtonElement).click();
    }
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === "f") {
      e.preventDefault();
      // Toggle the Favorites sidebar filter.
      selection.filter =
        selection.filter.kind === "favorites" ? { kind: "all" } : { kind: "favorites" };
      return;
    }
    if ((e.metaKey || e.ctrlKey) && e.key === "f") {
      e.preventDefault();
      const input = document.querySelector(".search-input") as HTMLInputElement | null;
      if (input) input.focus();
    }
  }

  // Start idle/sleep lock monitors while the vault is unlocked
  $effect(() => {
    if (!vault.locked && vault.meta) {
      startIdleTimer(security.idleLockMs);
      return stopIdleTimer;
    }
  });

  let showKdfUpgrade = $state(false);
  let kdfUpgradeDismissedVaults = $state<string[]>([]);

  // Load persisted KDF-dismissals so a deliberate "Later" survives restarts.
  onMount(async () => {
    try {
      const prefs = await prefsBridge.prefsGet();
      kdfUpgradeDismissedVaults = prefs.kdfUpgradeDismissedVaults ?? [];
    } catch {
      // Non-fatal — worst case the upgrade prompt shows again.
    }
  });

  $effect(() => {
    const path = vault.meta?.path;
    const dismissed = path ? kdfUpgradeDismissedVaults.includes(path) : false;
    if (vault.meta?.kdfNeedsUpgrade && !vault.locked && !dismissed) {
      showKdfUpgrade = true;
    }
  });

  onMount(() => {
    const unlisten = listen("menu://settings", () => {
      app.toggleSettings();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  });
</script>

<svelte:window {onkeydown} />

{#if app.view === "settings"}
  <SettingsView />
{:else}
  <div class="app-window">
    <div class="main-grid">
      <Sidebar />
      <ItemList />
      <ItemDetail />
    </div>
    <StatusBar />
  </div>
{/if}

{#if showCommandPalette}
  <CommandPalette onSelect={onCreateEntry} onClose={() => (showCommandPalette = false)} />
{/if}

{#if showKdfUpgrade}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="kdf-overlay" onclick={() => (showKdfUpgrade = false)}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="kdf-dialog" onclick={(e) => e.stopPropagation()}>
      <h2>Upgrade vault security?</h2>
      <p>
        This vault uses a weak KDF configuration (less than 64 MiB memory). Upgrade to Argon2id with
        64 MiB for better protection against brute-force attacks?
      </p>
      <div class="kdf-actions">
        <button
          class="btn"
          onclick={async () => {
            showKdfUpgrade = false;
            const path = vault.meta?.path;
            if (path) {
              kdfUpgradeDismissedVaults = [...kdfUpgradeDismissedVaults, path];
              try {
                await prefsBridge.prefsSetKdfDismissed(path, true);
              } catch (e) {
                console.error("Failed to persist KDF dismissal", e);
              }
            }
          }}
        >
          Later
        </button>
        <button
          class="btn btn-primary"
          onclick={async () => {
            try {
              await vaultBridge.vaultUpgradeKdf();
              showKdfUpgrade = false;
            } catch (e) {
              console.error("KDF upgrade failed", e);
            }
          }}
        >
          Upgrade
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .app-window {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--surface-2);
  }

  .main-grid {
    display: grid;
    grid-template-columns: var(--sidebar-width) var(--list-width) minmax(0, 1fr);
    flex: 1;
    min-height: 480px;
    overflow: hidden;
  }

  .kdf-overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
  }

  .kdf-dialog {
    background: var(--surface-0);
    border-radius: 12px;
    padding: 24px;
    width: 400px;
    max-width: 90vw;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }

  .kdf-dialog h2 {
    margin: 0 0 12px 0;
    font-size: 1.2rem;
  }

  .kdf-dialog p {
    margin: 0 0 20px 0;
    color: var(--text-2);
    line-height: 1.5;
  }

  .kdf-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .kdf-actions .btn {
    padding: 8px 20px;
    border-radius: 8px;
    border: none;
    cursor: pointer;
    font-size: 0.9rem;
  }

  .kdf-actions .btn-primary {
    background: var(--accent);
    color: white;
  }
</style>
