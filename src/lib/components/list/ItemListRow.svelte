<script lang="ts">
  import type { Entry } from "$lib/bridge/types";
  import EntryIcon from "./EntryIcon.svelte";
  import Icon from "../ui/Icon.svelte";

  let {
    entry,
    selected = false,
    onclick,
  }: {
    entry: Entry;
    selected?: boolean;
    onclick?: () => void;
  } = $props();

  let subtitle = $derived.by(() => {
    if (entry.type === "card" && entry.card?.number) {
      const num = entry.card.number;
      return `${num.slice(0, 4)} •••• ${num.slice(-4)}`;
    }
    return entry.subtitle;
  });
</script>

<button class="list-row" class:selected {onclick} role="option" aria-selected={selected}>
  <EntryIcon iconHint={entry.iconHint} type={entry.type} title={entry.title} size={30} />
  <div class="list-row-text">
    <div class="list-row-title">{entry.title}</div>
    <div class="list-row-subtitle">{subtitle}</div>
  </div>
  {#if entry.favorite}
    <span style="color: var(--warning); flex-shrink: 0; display: flex;">
      <Icon name="star" size={12} />
    </span>
  {/if}
</button>

<style>
  .list-row {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 10px 12px;
    border-bottom: 0.5px solid var(--border);
    width: 100%;
    text-align: left;
    transition: background 0.1s;
  }

  .list-row:hover {
    background: var(--border);
  }

  .list-row.selected {
    background: var(--bg-accent);
  }

  .list-row.selected .list-row-title {
    color: var(--text-accent);
  }

  .list-row-text {
    flex: 1;
    min-width: 0;
  }

  .list-row-title {
    font-size: 13.5px;
    font-weight: 500;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .list-row-subtitle {
    font-size: 11.5px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
