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
    padding: 2px 6px 4px 10px;
  }

  .section-label {
    flex: 1;
    font-size: 11px;
    color: var(--text-muted);
  }

  .section-action,
  .collapse-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }

  .section-action:hover,
  .collapse-button:hover {
    color: var(--text-secondary);
    background: var(--border);
  }

  .section-items {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
</style>
