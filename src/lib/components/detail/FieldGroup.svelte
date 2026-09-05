<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

<div class="field-group">
  {@render children()}
</div>

<style>
  /* Dividers are the parent showing through a 1px gap between rows, so the
     colour has to carry the line on its own — hence --divider, not --border. */
  .field-group {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--divider);
    border-radius: var(--radius);
    overflow: hidden;
    margin-bottom: 16px;
  }
</style>
