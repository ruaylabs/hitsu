<script lang="ts">
  import { vault } from "$lib/stores/vault.svelte";

  let itemCount = $derived(vault.entries.length);
</script>

<footer class="statusbar">
  <div class="statusbar-left">
    <span class="sync-dot"></span>
    <span>Changes saved</span>
  </div>
  <div class="statusbar-right">
    <span>{itemCount} items</span>
  </div>
</footer>

<style>
  .statusbar {
    height: var(--statusbar-height);
    padding: 0 14px;
    background: var(--surface-1);
    border-top: 0.5px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .statusbar-left {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .sync-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--success);
    flex-shrink: 0;
  }

  .statusbar-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }
</style>
