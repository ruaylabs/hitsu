<script lang="ts">
  import type { Entry } from "$lib/bridge/types";
  import { openHttpUrl } from "$lib/utils/openHttpUrl";
  import EntryIcon from "../list/EntryIcon.svelte";
  import IconButton from "../ui/IconButton.svelte";

  let {
    entry,
    onFavorite,
    onEdit,
    onMove,
    onTotpSetup,
    showMove = false,
    showTotpSetup,
    readOnly = false,
  }: {
    entry: Entry;
    onFavorite?: () => void;
    onEdit?: () => void;
    onMove?: () => void;
    onTotpSetup?: () => void;
    showMove?: boolean;
    showTotpSetup?: boolean;
    readOnly?: boolean;
  } = $props();
</script>

<div class="detail-header">
  <div class="detail-header-left">
    <EntryIcon iconHint={entry.iconHint} type={entry.type} title={entry.title} size={48} />
    <div class="detail-header-text">
      <h1 class="detail-title">{entry.title}</h1>
      {#if entry.url}
        <button class="detail-url" onclick={() => openHttpUrl(entry.url!)} title={entry.url}>
          {entry.url}
        </button>
      {/if}
    </div>
  </div>
  {#if !readOnly}
    <div class="detail-header-actions">
      {#if showTotpSetup}
        <IconButton
          icon="key"
          iconSize={14}
          variant="outline"
          onclick={onTotpSetup}
          aria-label="Setup TOTP"
          title="Setup TOTP from seed"
        />
      {/if}
      {#if showMove}
        <IconButton
          icon="folder-share"
          iconSize={14}
          variant="outline"
          onclick={onMove}
          aria-label="Move entry"
          title="Move entry"
        />
      {/if}
      <IconButton
        icon="star"
        iconSize={14}
        variant="outline"
        active={entry.favorite}
        onclick={onFavorite}
        aria-label={entry.favorite ? "Unfavorite" : "Favorite"}
        title={entry.favorite ? "Unfavorite" : "Favorite"}
      />
      <IconButton
        icon="pencil"
        iconSize={14}
        variant="outline"
        onclick={onEdit}
        aria-label="Edit entry"
        title="Edit entry"
      />
    </div>
  {/if}
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
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    text-align: left;
    width: 100%;
  }

  .detail-url:hover {
    text-decoration: underline;
  }

  .detail-header-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }
</style>
