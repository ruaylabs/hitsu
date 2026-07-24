<script lang="ts">
  import type { EntrySummary } from "$lib/bridge/types";
  import Icon from "../ui/Icon.svelte";
  import EntryIcon from "./EntryIcon.svelte";

  let {
    entry,
    selected = false,
    onclick,
    oncontextmenu,
  }: {
    entry: EntrySummary;
    selected?: boolean;
    onclick?: () => void;
    oncontextmenu?: (event: MouseEvent) => void;
  } = $props();
</script>

<button
  class="list-row"
  class:selected
  {onclick}
  {oncontextmenu}
  role="option"
  aria-selected={selected}
  tabindex={selected ? 0 : -1}
  data-entry-id={entry.id}
>
  <EntryIcon iconHint={entry.iconHint} type={entry.type} title={entry.title} size={30} />
  <div class="list-row-text">
    <div class="list-row-title">{entry.title}</div>
    <div class="list-row-subtitle">{entry.subtitle}</div>
  </div>
  {#if entry.hasTotp || entry.hasAttachments || entry.favorite}
    <span class="row-indicators">
      {#if entry.hasTotp}
        <span class="row-indicator" aria-label="Has TOTP" title="Has TOTP">
          <Icon name="clock-code" size={12} />
        </span>
      {/if}
      {#if entry.hasAttachments}
        <span class="row-indicator" aria-label="Has attachments" title="Has attachments">
          <Icon name="paperclip" size={12} />
        </span>
      {/if}
      {#if entry.favorite}
        <span class="row-indicator favorite-indicator" aria-label="Favorite" title="Favorite">
          <Icon name="star" size={12} />
        </span>
      {/if}
    </span>
  {/if}
</button>

<style>
  .list-row {
    display: flex;
    align-items: center;
    gap: 11px;
    /* Fixed height: the virtualized list (ItemList) positions rows at
       ROW_HEIGHT intervals, so this must stay in sync with that constant. */
    height: 50px;
    padding: 0 12px;
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

  .row-indicators {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 4px;
    flex-shrink: 0;
    color: var(--text-muted);
  }

  .row-indicator {
    display: inline-flex;
  }

  .favorite-indicator {
    color: var(--warning);
  }
</style>
