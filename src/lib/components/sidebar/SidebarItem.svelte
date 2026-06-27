<script lang="ts">
  import Icon from "../ui/Icon.svelte";

  let {
    label,
    icon,
    count,
    selected = false,
    tagColor,
    onclick,
  }: {
    label: string;
    icon?: string;
    count?: number;
    selected?: boolean;
    tagColor?: string;
    onclick?: () => void;
  } = $props();
</script>

<button class="sidebar-item" class:selected {onclick} role="tab" aria-selected={selected}>
  {#if tagColor}
    <span class="tag-dot" style="background: {tagColor}"></span>
  {:else if icon}
    <Icon name={icon} size={14} />
  {/if}
  <span class="sidebar-label">{label}</span>
  {#if count !== undefined}
    <span class="sidebar-count">{count}</span>
  {/if}
</button>

<style>
  .sidebar-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    width: 100%;
    text-align: left;
    font-size: 13px;
    color: var(--text-primary);
    transition: background 0.1s;
  }

  .sidebar-item:hover {
    background: var(--border);
  }

  .sidebar-item.selected {
    background: var(--bg-accent);
    color: var(--text-accent);
    font-weight: 500;
  }

  .sidebar-label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sidebar-count {
    font-size: 12px;
    color: var(--text-muted);
  }

  .sidebar-item.selected .sidebar-count {
    color: var(--text-accent);
  }

  .tag-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
</style>
