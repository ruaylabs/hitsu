<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { app } from "$lib/stores/app.svelte";
  import StatusBar from "$lib/components/chrome/StatusBar.svelte";
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import ItemList from "$lib/components/list/ItemList.svelte";
  import ItemDetail from "$lib/components/detail/ItemDetail.svelte";
  import SettingsView from "$lib/components/settings/SettingsView.svelte";

  function onkeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === ",") {
      e.preventDefault();
      app.toggleSettings();
    }
  }

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
