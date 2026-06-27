<script lang="ts">
  import type { Snippet } from "svelte";

  let { label, children }: { label: string; children: Snippet } = $props();
</script>

<div class="sidebar-section">
  <span class="section-label">{label}</span>
  <div class="section-items">
    {@render children()}
  </div>
</div>

<style>
  .sidebar-section {
    margin-bottom: 8px;
  }

  .section-label {
    display: block;
    font-size: 11px;
    color: var(--text-muted);
    padding: 2px 10px 6px;
  }

  .section-items {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
</style>
