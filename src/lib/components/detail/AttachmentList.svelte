<script lang="ts">
  import * as entriesBridge from "$lib/bridge/entries";
  import type { AttachmentMeta } from "$lib/bridge/types";
  import { nativeDialog } from "$lib/stores/nativeDialog.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { errorMessage } from "$lib/utils/errorMessage";
  import { formatFileSize } from "$lib/utils/format";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";
  import Icon from "../ui/Icon.svelte";
  import IconButton from "../ui/IconButton.svelte";

  let {
    entryId,
    attachments,
    onchange,
  }: {
    entryId: string;
    attachments: AttachmentMeta[];
    onchange?: () => void;
  } = $props();

  async function download(att: AttachmentMeta) {
    try {
      const bytes = await nativeDialog.during(() =>
        entriesBridge.entryAttachmentSave(entryId, att.name),
      );
      if (bytes === null) return; // user cancelled the Rust-owned native dialog
      toast.success(`Saved ${att.name} (${formatFileSize(bytes)})`);
    } catch (e) {
      toast.error(`Failed to save ${att.name}: ${errorMessage(e)}`);
    }
  }

  async function pickFile() {
    try {
      const attachment = await nativeDialog.during(() => entriesBridge.entryAttachmentAdd(entryId));
      if (attachment === null) return; // user cancelled the Rust-owned native dialog
      toast.success(`Added ${attachment.name}`);
      onchange?.();
    } catch (e) {
      toast.error(`Failed to add attachment: ${errorMessage(e)}`);
    }
  }

  let pendingRemoval = $state<AttachmentMeta | null>(null);

  function requestRemove(att: AttachmentMeta) {
    pendingRemoval = att;
  }

  function cancelRemove() {
    pendingRemoval = null;
  }

  async function confirmRemove() {
    const att = pendingRemoval;
    if (!att) return;
    pendingRemoval = null;
    try {
      await entriesBridge.entryAttachmentRemove(entryId, att.name);
      toast.success(`Removed ${att.name}`);
      onchange?.();
    } catch (e) {
      toast.error(`Failed to remove ${att.name}: ${errorMessage(e)}`);
    }
  }
</script>

<div class="attachments-section">
  <div class="attachments-header">
    <span class="attachments-label">Attachments</span>
    <button class="add-btn" onclick={pickFile} aria-label="Add attachment" title="Add attachment">
      <Icon name="plus" size={14} />
      <span>Add</span>
    </button>
  </div>

  {#if attachments.length > 0}
    <div class="attachments-list">
      {#each attachments as att (att.id)}
        <div class="attachment-row">
          <Icon name="file" size={18} />
          <div class="attachment-info">
            <span class="attachment-name">{att.name}</span>
            <span class="attachment-size">{formatFileSize(att.sizeBytes)}</span>
          </div>
          <IconButton
            icon="download"
            onclick={() => download(att)}
            aria-label="Download {att.name}"
            title="Download {att.name}"
          />
          <IconButton
            icon="trash"
            iconSize={14}
            variant="danger"
            onclick={() => requestRemove(att)}
            aria-label="Remove {att.name}"
            title="Remove {att.name}"
          />
        </div>
      {/each}
    </div>
  {:else}
    <p class="attachments-empty">No attachments</p>
  {/if}
</div>

{#if pendingRemoval}
  <ConfirmDialog
    title="Remove attachment?"
    message={`Are you sure you want to remove “${pendingRemoval.name}”?`}
    confirmLabel="Remove"
    danger={true}
    onconfirm={confirmRemove}
    oncancel={cancelRemove}
  />
{/if}

<style>
  .attachments-section {
    margin-bottom: 16px;
  }

  .attachments-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }

  .attachments-label {
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .add-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-1);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
  }

  .add-btn:hover {
    background: var(--border);
  }

  .attachments-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .attachment-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
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
    font-size: var(--text-base);
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .attachment-size {
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .attachments-empty {
    font-size: var(--text-sm);
    color: var(--text-muted);
    font-style: italic;
  }
</style>
