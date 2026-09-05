<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "../ui/Icon.svelte";

  let {
    label,
    collapsed = false,
    ontoggle,
    onadd,
    addLabel = `Add ${label}`,
    children,
  }: {
    label: string;
    collapsed?: boolean;
    ontoggle?: () => void;
    onadd?: () => void;
    addLabel?: string;
    children: Snippet;
  } = $props();
</script>

<div class="sidebar-section">
  <div class="section-header">
    <span class="section-label">{label}</span>
    {#if onadd}
      <button class="section-action" type="button" aria-label={addLabel} onclick={onadd}>
        <Icon name="plus" size={13} />
      </button>
    {/if}
    {#if ontoggle}
      <button
        class="collapse-button"
        type="button"
        aria-label={collapsed ? `Expand ${label}` : `Collapse ${label}`}
        aria-expanded={!collapsed}
        onclick={ontoggle}
      >
        <Icon name={collapsed ? "chevron-right" : "chevron-down"} size={13} />
      </button>
    {/if}
  </div>
  {#if !collapsed}
    <div class="section-items">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .sidebar-section {
    margin-bottom: 8px;
  }

  .section-header {
    display: flex;
    align-items: center;
    padding: var(--space-half) var(--space-2) var(--space-1) var(--space-3);
  }

  .section-label {
    flex: 1;
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .section-action,
  .collapse-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--icon-button-size);
    height: var(--icon-button-size);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }

  .section-action:hover,
  .collapse-button:hover {
    color: var(--text-secondary);
    background: var(--surface-hover);
  }

  .section-items {
    display: flex;
    flex-direction: column;
    gap: var(--space-half);
  }
</style>
