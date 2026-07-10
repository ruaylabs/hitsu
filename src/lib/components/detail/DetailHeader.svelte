<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import type { Entry } from "$lib/bridge/types";
  import EntryIcon from "../list/EntryIcon.svelte";
  import Icon from "../ui/Icon.svelte";

  function openEntryUrl(rawUrl: string): void {
    const url = rawUrl.includes("://") ? rawUrl : `https://${rawUrl}`;
    try {
      const parsed = new URL(url);
      if (parsed.protocol === "http:" || parsed.protocol === "https:") {
        openUrl(url);
      } else {
        console.warn("Blocked opening URL with disallowed scheme:", parsed.protocol);
      }
    } catch {
      console.warn("Blocked opening invalid URL:", url);
    }
  }

  let {
    entry,
    onFavorite,
    onEdit,
    onTotpSetup,
    showTotpSetup,
  }: {
    entry: Entry;
    onFavorite?: () => void;
    onEdit?: () => void;
    onTotpSetup?: () => void;
    showTotpSetup?: boolean;
  } = $props();
</script>

<div class="detail-header">
  <div class="detail-header-left">
    <EntryIcon iconHint={entry.iconHint} type={entry.type} title={entry.title} size={48} />
    <div class="detail-header-text">
      <h1 class="detail-title">{entry.title}</h1>
      {#if entry.url}
        <button class="detail-url" onclick={() => openEntryUrl(entry.url!)} title={entry.url}>
          {entry.url}
        </button>
      {/if}
    </div>
  </div>
  <div class="detail-header-actions">
    {#if showTotpSetup}
      <button
        class="action-btn"
        onclick={onTotpSetup}
        aria-label="Setup TOTP"
        title="Setup TOTP from seed"
      >
        <Icon name="key" size={14} />
      </button>
    {/if}
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
