<script lang="ts">
  import type { Entry } from "$lib/bridge/types";
  import EntryIcon from "../list/EntryIcon.svelte";
  import Icon from "../ui/Icon.svelte";

  let {
    entry,
    onFavorite,
    onEdit,
  }: {
    entry: Entry;
    onFavorite?: () => void;
    onEdit?: () => void;
  } = $props();
</script>

<div class="detail-header">
  <div class="detail-header-left">
    <EntryIcon iconHint={entry.iconHint} type={entry.type} title={entry.title} size={48} />
    <div class="detail-header-text">
      <h1 class="detail-title">{entry.title}</h1>
      {#if entry.url}
        <a
          class="detail-url"
          href={entry.url.includes("://") ? entry.url : `https://${entry.url}`}
          target="_blank"
          rel="noreferrer"
        >
          {entry.url}
        </a>
      {/if}
    </div>
  </div>
  <div class="detail-header-actions">
    <button
      class="action-btn"
      class:favorited={entry.favorite}
      onclick={onFavorite}
      aria-label={entry.favorite ? "Unfavorite" : "Favorite"}
      title={entry.favorite ? "Unfavorite" : "Favorite"}
    >
      <Icon name="star" size={14} />
    </button>
    <button class="action-btn" onclick={onEdit} aria-label="Edit entry" title="Edit entry">
      <Icon name="pencil" size={14} />
    </button>
  </div>
</div>

<style>
  .detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 20px;
    gap: 12px;
  }

  .detail-header-left {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
  }

  .detail-header-text {
    min-width: 0;
  }

  .detail-title {
    font-size: 18px;
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.2;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .detail-url {
    display: block;
    font-size: 12.5px;
    color: var(--text-accent);
    margin-top: 3px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .detail-url:hover {
    text-decoration: underline;
  }

  .detail-header-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: 0.5px solid var(--border-strong);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition: background 0.1s;
  }

  .action-btn:hover {
    background: var(--border);
  }

  .action-btn.favorited {
    color: var(--warning);
  }
</style>
