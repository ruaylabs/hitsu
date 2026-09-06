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
    folderName,
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
    folderName?: string;
  } = $props();
</script>

{#if folderName}
  <div class="detail-toolbar">
    <div class="breadcrumb">
      <span title={folderName}>{folderName}</span>
    </div>
  </div>
{/if}
<div class="detail-header">
  <div class="detail-header-left">
    <EntryIcon
      iconHint={entry.iconHint}
      type={entry.type}
      title={entry.title}
      size={49}
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
        iconSize={18}
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
  .detail-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: var(--space-2);
    padding-bottom: var(--space-4);
    margin-bottom: calc(var(--space-6) + var(--space-1));
    border-bottom: 1px solid var(--divider);
  }

  .breadcrumb {
    display: flex;
    gap: var(--space-2);
    min-width: 0;
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  .breadcrumb span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .detail-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: calc(var(--space-6) + var(--space-1));
    gap: var(--space-3);
  }

  .detail-header-left {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    min-width: 0;
  }

  .detail-header-text {
    min-width: 0;
  }

  .detail-title {
    font-size: 24px;
    font-weight: 600;
    letter-spacing: -0.7px;
    color: var(--text-primary);
    line-height: var(--leading-tight);
    overflow-wrap: anywhere;
    user-select: text;
  }

  .detail-header-actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }
</style>
