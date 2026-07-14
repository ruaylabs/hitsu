<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import * as entriesBridge from "$lib/bridge/entries";
  import { toSummary } from "$lib/bridge/entries";
  import * as prefsBridge from "$lib/bridge/prefs";
  import type { ItemType } from "$lib/bridge/types";
  import * as vaultBridge from "$lib/bridge/vault";
  import StatusBar from "$lib/components/chrome/StatusBar.svelte";
  import ItemDetail from "$lib/components/detail/ItemDetail.svelte";
  import ItemList from "$lib/components/list/ItemList.svelte";
  import SettingsView from "$lib/components/settings/SettingsView.svelte";
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import CommandPalette from "$lib/components/ui/CommandPalette.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import ShortcutsDialog from "$lib/components/ui/ShortcutsDialog.svelte";
  import { app } from "$lib/stores/app.svelte";
  import { entryDeletion } from "$lib/stores/entryDeletion.svelte";
  import { startIdleTimer, stopIdleTimer } from "$lib/stores/idle.svelte";
  import { security } from "$lib/stores/security.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { vault } from "$lib/stores/vault.svelte";

  let showCommandPalette = $state(false);
  let showShortcuts = $state(false);

  const SIDEBAR_DEFAULT = 168;
  const SIDEBAR_MIN = 140;
  const LIST_DEFAULT = 224;
  const LIST_MIN = 190;
  const DETAIL_MIN = 360;
  const RESIZER_WIDTH = 1;
  const PANE_WIDTHS_KEY = "kagi:pane-widths";

  let sidebarWidth = $state(SIDEBAR_DEFAULT);
  let listWidth = $state(LIST_DEFAULT);
  let viewportWidth = $state(1100);

  function fitPaneWidths() {
    viewportWidth = window.innerWidth;
    const available = viewportWidth - DETAIL_MIN - RESIZER_WIDTH * 2;
    sidebarWidth = Math.max(SIDEBAR_MIN, Math.min(sidebarWidth, available - LIST_MIN));
    listWidth = Math.max(LIST_MIN, Math.min(listWidth, available - sidebarWidth));
  }

  function savePaneWidths() {
    try {
      localStorage.setItem(PANE_WIDTHS_KEY, JSON.stringify({ sidebarWidth, listWidth }));
    } catch {
      // Layout persistence is optional.
    }
  }

  function resizePane(pane: "sidebar" | "list", delta: number, startWidth: number) {
    const available = window.innerWidth - DETAIL_MIN - RESIZER_WIDTH * 2;
    if (pane === "sidebar") {
      sidebarWidth = Math.max(SIDEBAR_MIN, Math.min(startWidth + delta, available - LIST_MIN));
    } else {
      listWidth = Math.max(LIST_MIN, Math.min(startWidth + delta, available - sidebarWidth));
    }
  }

  function beginPaneResize(pane: "sidebar" | "list", event: PointerEvent) {
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = pane === "sidebar" ? sidebarWidth : listWidth;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";

    const move = (moveEvent: PointerEvent) => {
      resizePane(pane, moveEvent.clientX - startX, startWidth);
    };
    const stop = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", stop);
      window.removeEventListener("pointercancel", stop);
      window.removeEventListener("blur", stop);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      savePaneWidths();
    };

    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", stop);
    window.addEventListener("pointercancel", stop);
    window.addEventListener("blur", stop);
  }

  function onResizerKeydown(pane: "sidebar" | "list", event: KeyboardEvent) {
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    event.preventDefault();
    resizePane(
      pane,
      event.key === "ArrowLeft" ? -10 : 10,
      pane === "sidebar" ? sidebarWidth : listWidth,
    );
    savePaneWidths();
  }

  function resetPaneWidth(pane: "sidebar" | "list") {
    if (pane === "sidebar") sidebarWidth = SIDEBAR_DEFAULT;
    else listWidth = LIST_DEFAULT;
    fitPaneWidths();
    savePaneWidths();
  }

  function deleteSelected() {
    if (selection.selectedId) entryDeletion.request(selection.selectedId);
  }

  function onCreateEntry(type: ItemType) {
    selection.requestNavigation(() => {
      void createEntry(type);
    });
  }

  async function createEntry(type: ItemType) {
    showCommandPalette = false;
    try {
      const entry = await entriesBridge.entryCreate(type, { title: `New ${type}` });
      vault.setEntries([...vault.entries, toSummary(entry)]);
      selection.filter = { kind: "type", type };
      selection.selectedId = entry.id;
      vault.setCreatingId(entry.id);
      vault.setEditingId(entry.id);
    } catch (e) {
      console.error("Failed to create entry", e);
      toast.error(e instanceof Error ? e.message : String(e));
    }
  }

  function onkeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement | null;
    const isEditable =
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLSelectElement ||
      target?.isContentEditable;

    if (e.key === "?" && !e.metaKey && !e.ctrlKey && !e.altKey && !isEditable) {
      e.preventDefault();
      showShortcuts = true;
      return;
    }

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
      selection.requestNavigation(() => app.toggleSettings());
    }
    if ((e.metaKey || e.ctrlKey) && e.key === "n") {
      e.preventDefault();
      showCommandPalette = true;
    }
    if ((e.metaKey || e.ctrlKey) && e.key === "Backspace") {
      e.preventDefault();
      deleteSelected();
    }
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === "f") {
      e.preventDefault();
      // Toggle the Favorites sidebar filter.
      selection.requestNavigation(() => {
        selection.filter =
          selection.filter.kind === "favorites" ? { kind: "all" } : { kind: "favorites" };
      });
      return;
    }
    if ((e.metaKey || e.ctrlKey) && e.key === "f") {
      e.preventDefault();
      const input = document.querySelector(".entry-search-input") as HTMLInputElement | null;
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

  onMount(() => {
    try {
      const saved = JSON.parse(localStorage.getItem(PANE_WIDTHS_KEY) ?? "null");
      if (Number.isFinite(saved?.sidebarWidth)) sidebarWidth = saved.sidebarWidth;
      if (Number.isFinite(saved?.listWidth)) listWidth = saved.listWidth;
    } catch {
      // Ignore invalid persisted layout values.
    }
    fitPaneWidths();
    window.addEventListener("resize", fitPaneWidths);
    return () => window.removeEventListener("resize", fitPaneWidths);
  });

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
    const listeners = [
      listen("menu://settings", () => {
        selection.requestNavigation(() => app.toggleSettings());
      }),
      listen("vault://session-locked", () => {
        vault.sessionLocked();
      }),
    ];
    return () => {
      for (const unlisten of listeners) unlisten.then((fn) => fn());
    };
  });
</script>

<svelte:window {onkeydown} />

{#if app.view === "settings"}
  <SettingsView />
{:else}
  <div class="app-window">
    <div
      class="main-grid"
      style={`--sidebar-current: ${sidebarWidth}px; --list-current: ${listWidth}px;`}
    >
      <Sidebar />
      <button
        type="button"
        class="pane-resizer"
        role="slider"
        aria-label="Sidebar width"
        aria-orientation="horizontal"
        aria-valuemin={SIDEBAR_MIN}
        aria-valuemax={viewportWidth - DETAIL_MIN - RESIZER_WIDTH * 2 - LIST_MIN}
        aria-valuenow={Math.round(sidebarWidth)}
        onpointerdown={(event) => beginPaneResize("sidebar", event)}
        onkeydown={(event) => onResizerKeydown("sidebar", event)}
        ondblclick={() => resetPaneWidth("sidebar")}
      ></button>
      <ItemList />
      <button
        type="button"
        class="pane-resizer"
        role="slider"
        aria-label="Item list width"
        aria-orientation="horizontal"
        aria-valuemin={LIST_MIN}
        aria-valuemax={viewportWidth - DETAIL_MIN - RESIZER_WIDTH * 2 - sidebarWidth}
        aria-valuenow={Math.round(listWidth)}
        onpointerdown={(event) => beginPaneResize("list", event)}
        onkeydown={(event) => onResizerKeydown("list", event)}
        ondblclick={() => resetPaneWidth("list")}
      ></button>
      <ItemDetail />
    </div>
    <StatusBar
      onHelpClick={() => (showShortcuts = true)}
      onSettingsClick={() => selection.requestNavigation(() => app.toggleSettings())}
    />
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
              toast.success("Vault security upgraded");
            } catch (e) {
              console.error("KDF upgrade failed", e);
              toast.error(e instanceof Error ? e.message : String(e));
            }
          }}
        >
          Upgrade
        </button>
      </div>
    </div>
  </div>
{/if}

{#if showShortcuts}
  <ShortcutsDialog onclose={() => (showShortcuts = false)} />
{/if}

{#if entryDeletion.pending}
  <ConfirmDialog
    title={entryDeletion.pending.permanent ? "Delete permanently?" : "Move to Recycle Bin?"}
    message={entryDeletion.pending.permanent
      ? `Permanently delete "${entryDeletion.pending.title}"? This cannot be undone.`
      : `Move "${entryDeletion.pending.title}" to the Recycle Bin?`}
    confirmLabel={entryDeletion.pending.permanent ? "Delete permanently" : "Move to Bin"}
    danger={true}
    onconfirm={() => entryDeletion.confirm()}
    oncancel={() => entryDeletion.cancel()}
  />
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
    grid-template-columns:
      var(--sidebar-current, var(--sidebar-width)) var(--resizer-width)
      var(--list-current, var(--list-width)) var(--resizer-width) minmax(0, 1fr);
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .pane-resizer {
    position: relative;
    width: var(--resizer-width);
    cursor: col-resize;
    background: var(--surface-1);
    outline: none;
  }

  .pane-resizer::before {
    content: "";
    position: absolute;
    inset: 0 -4px;
  }

  .pane-resizer::after {
    content: "";
    position: absolute;
    inset: 0;
    background: var(--border);
    transition: background 0.1s;
  }

  .pane-resizer:hover::after,
  .pane-resizer:focus-visible::after {
    background: var(--accent);
  }

  .kdf-overlay {
    position: fixed;
    inset: 0;
    z-index: var(--z-blocking-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--backdrop-strong);
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
    color: var(--text-secondary);
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
