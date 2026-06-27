<script lang="ts">
  import StatusBar from "$lib/components/chrome/StatusBar.svelte";
  import Sidebar from "$lib/components/sidebar/Sidebar.svelte";
  import ItemList from "$lib/components/list/ItemList.svelte";
  import ItemDetail from "$lib/components/detail/ItemDetail.svelte";
</script>

<div class="app-window">
  <div class="main-grid">
    <Sidebar />
    <ItemList />
    <ItemDetail />
  </div>
  <StatusBar />
</div>

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
