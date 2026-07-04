<script lang="ts">
  import type { AttachmentMeta } from "$lib/bridge/types";
  import { formatFileSize } from "$lib/utils/format";
  import Icon from "../ui/Icon.svelte";

  let { attachments }: { attachments: AttachmentMeta[] } = $props();
</script>

{#if attachments.length > 0}
  <div class="attachments-section">
    <span class="attachments-label">Attachments</span>
    <div class="attachments-list">
      {#each attachments as att (att.id)}
        <div class="attachment-row">
          <Icon name="file" size={18} />
          <div class="attachment-info">
            <span class="attachment-name">{att.name}</span>
            <span class="attachment-size">{formatFileSize(att.sizeBytes)}</span>
          </div>
          <button
            class="attachment-download"
            aria-label="Download {att.name}"
            title="Download {att.name}"
          >
            <Icon name="download" size={15} />
          </button>
        </div>
      {/each}
    </div>
  </div>
{/if}

<style>
  .attachments-section {
    margin-bottom: 16px;
  }

  .attachments-label {
    display: block;
    font-size: 11px;
    color: var(--text-muted);
    margin-bottom: 6px;
  }

  .attachments-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .attachment-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    background: var(--surface-1);
    border-radius: var(--radius);
  }

  .attachment-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .attachment-name {
    font-size: 13px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .attachment-size {
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .attachment-download {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }

  .attachment-download:hover {
    background: var(--border);
  }
</style>
