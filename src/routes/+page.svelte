<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { app } from "$lib/stores/app.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { startIdleTimer, stopIdleTimer } from "$lib/stores/idle.svelte";
  import * as vaultBridge from "$lib/bridge/vault";
  import * as entriesBridge from "$lib/bridge/entries";
  import * as prefsBridge from "$lib/bridge/prefs";
  import StatusBar from "$lib/components/chrome/StatusBar.svelte";
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import ItemList from "$lib/components/list/ItemList.svelte";
  import ItemDetail from "$lib/components/detail/ItemDetail.svelte";
  import SettingsView from "$lib/components/settings/SettingsView.svelte";
  import PasswordDialog from "$lib/components/ui/PasswordDialog.svelte";
  import OnboardingView from "$lib/components/unlock/OnboardingView.svelte";

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
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
      // Trigger add — find the add button in status bar
      document.querySelector('[aria-label="Add entry"]')?.dispatchEvent(new MouseEvent("click"));
    }
    if ((e.metaKey || e.ctrlKey) && e.key === "Backspace") {
      e.preventDefault();
      // Delete selected entry via detail pane's delete
      const btn = document.querySelector('[aria-label="Delete"]');
      if (btn) (btn as HTMLButtonElement).click();
    }
    if ((e.metaKey || e.ctrlKey) && e.key === "f") {
      e.preventDefault();
      const input = document.querySelector(".search-input") as HTMLInputElement | null;
      if (input) input.focus();
    }
  }

  // Idle timeout loaded from prefs (default 5 min, 0 = never)
  let idleTimeoutMs = $state(5 * 60 * 1000);

  // Start idle/sleep lock monitors when the vault is unlocked, stop when locked
  $effect(() => {
    if (!vault.locked && vault.meta) {
      startIdleTimer(idleTimeoutMs);
      return stopIdleTimer;
    }
  });

  let startupDialog: "password" | null = $state(null);
  let startupPath = $state("");
  let startupError = $state("");
  let unlockError = $state("");
  let startupChecked = $state(false);

  onMount(() => {
    const unlisten = listen("menu://settings", () => {
      app.toggleSettings();
    });
    // Load preferences — both startup vault and security settings
    prefsBridge
      .prefsGet()
      .then((prefs) => {
        idleTimeoutMs = (prefs.idleLockMinutes ?? 5) * 60 * 1000;
        if (prefs.lastVault) {
          startupPath = prefs.lastVault;
          startupDialog = "password";
        }
        startupChecked = true;
      })
      .catch(() => {
        startupChecked = true;
      });
    return () => {
      unlisten.then((fn) => fn());
    };
  });

  async function onStartupPassword(password: string) {
    startupError = "";
    try {
      const meta = await vaultBridge.vaultOpen(startupPath, password);
      vault.setMeta(meta);

      const summaries = await entriesBridge.entriesList();
      const fullEntries = await Promise.all(summaries.map((s) => entriesBridge.entryGet(s.id)));
      vault.setEntries(fullEntries);
      startupDialog = null;
    } catch (e) {
      startupError = e instanceof Error ? e.message : String(e);
    }
  }

  async function onUnlock(password: string) {
    unlockError = "";
    try {
      const meta = await vaultBridge.vaultOpen(vault.meta!.path, password);
      vault.setMeta(meta);

      const summaries = await entriesBridge.entriesList();
      const fullEntries = await Promise.all(summaries.map((s) => entriesBridge.entryGet(s.id)));
      vault.setEntries(fullEntries);
      vault.unlock();
    } catch (e) {
      unlockError = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<svelte:window {onkeydown} />

{#if startupDialog === "password"}
  <PasswordDialog
    title="Unlock vault"
    confirmLabel="Unlock"
    errorMessage={startupError}
    transparentOverlay
    onconfirm={onStartupPassword}
    oncancel={() => {
      startupDialog = null;
      vault.setMeta(null);
    }}
  />
{/if}

{#if vault.locked && vault.meta}
  <PasswordDialog
    title="Locked"
    confirmLabel="Unlock"
    errorMessage={unlockError}
    transparentOverlay
    showCancel={false}
    onconfirm={onUnlock}
  />
{/if}

{#if startupDialog || (vault.locked && vault.meta)}
<!-- Password dialogs rendered above, nothing else to show -->
{:else if !startupChecked}
<!-- Waiting for startup check — show blank -->
{:else if app.view === "settings"}
  <SettingsView />
{:else if !vault.meta}
  <OnboardingView />
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
</style>
