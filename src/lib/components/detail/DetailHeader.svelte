<script lang="ts">
  import type { Entry } from "$lib/bridge/types";
  import EntryIcon from "../list/EntryIcon.svelte";
  import IconButton from "../ui/IconButton.svelte";

  let {
    entry,
    onFavorite,
    onEdit,
    onMove,
    onTotpSetup,
    onDownloadFavicon,
    showMove = false,
    showTotpSetup,
    showDownloadFavicon = false,
    downloadingFavicon = false,
    readOnly = false,
  }: {
    entry: Entry;
    onFavorite?: () => void;
    onEdit?: () => void;
    onMove?: () => void;
    onTotpSetup?: () => void;
    onDownloadFavicon?: () => void;
    showMove?: boolean;
    showTotpSetup?: boolean;
    showDownloadFavicon?: boolean;
    downloadingFavicon?: boolean;
    readOnly?: boolean;
  } = $props();
</script>

<div class="detail-header">
  <div class="detail-header-left">
    <EntryIcon
      iconHint={entry.iconHint}
      type={entry.type}
      title={entry.title}
      size={48}
      hasCustomIcon={entry.hasCustomIcon}
      customIconData={entry.customIconData}
      entryId={entry.id}
    />
    <div class="detail-header-text">
      <h1 class="detail-title" title={entry.title}>{entry.title}</h1>
    </div>
  </div>
  {#if !readOnly}
    <div class="detail-header-actions">
      {#if showDownloadFavicon}
        <IconButton
          icon="photo-down"
          iconSize={14}
          variant="outline"
          disabled={downloadingFavicon}
          onclick={onDownloadFavicon}
          aria-label="Download favicon"
          title="Download favicon"
        />
      {/if}
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

  .detail-header-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }
</style>
