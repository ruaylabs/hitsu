<script lang="ts">
  import Icon from "../ui/Icon.svelte";

  let {
    label,
    icon,
    count,
    selected = false,
    tagColor,
    indent = 0,
    onclick,
    onadd,
    onedit,
    ondelete,
  }: {
    label: string;
    icon?: string;
    count?: number;
    selected?: boolean;
    tagColor?: string;
    indent?: number;
    onclick?: () => void;
    onadd?: () => void;
    onedit?: () => void;
    ondelete?: () => void;
  } = $props();
</script>

<div class="sidebar-row" class:has-actions={Boolean(onadd || onedit || ondelete)}>
  <button
    class="sidebar-item"
    class:selected
    style:padding-left={`${10 + indent * 16}px`}
    {onclick}
    aria-current={selected ? "true" : undefined}
  >
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
  {#if onadd || onedit || ondelete}
    <div class="sidebar-actions">
      {#if onadd}
        <button type="button" aria-label={`Add folder inside ${label}`} onclick={onadd}>
          <Icon name="plus" size={13} />
        </button>
      {/if}
      {#if onedit}
        <button type="button" aria-label={`Rename ${label}`} onclick={onedit}>
          <Icon name="pencil" size={12} />
        </button>
      {/if}
      {#if ondelete}
        <button
          type="button"
          class="action-delete"
          aria-label={`Delete ${label}`}
          onclick={ondelete}
        >
          <Icon name="trash" size={12} />
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .sidebar-row {
    position: relative;
  }

  .sidebar-item {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 32px;
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

  .sidebar-row.has-actions:hover .sidebar-count {
    opacity: 0;
  }

  .sidebar-item.selected .sidebar-count {
    color: var(--text-accent);
  }

  .sidebar-actions {
    position: absolute;
    top: 50%;
    right: 4px;
    display: flex;
    gap: 1px;
    padding-left: 10px;
    opacity: 0;
    transform: translateY(-50%);
    pointer-events: none;
  }

  .sidebar-row:hover .sidebar-actions,
  .sidebar-row:focus-within .sidebar-actions {
    opacity: 1;
    pointer-events: auto;
  }

  .sidebar-actions button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }

  .sidebar-actions button:hover {
    color: var(--text-primary);
    background: var(--border);
  }

  .sidebar-actions .action-delete:hover {
    color: var(--danger);
    background: var(--danger-bg);
  }

  .tag-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }
</style>
