<script lang="ts">
  import * as entriesBridge from "$lib/bridge/entries";
  import type { AttachmentMeta } from "$lib/bridge/types";
  import { toast } from "$lib/stores/toast.svelte";
  import { formatFileSize } from "$lib/utils/format";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";
  import Icon from "../ui/Icon.svelte";

  let {
    entryId,
    attachments,
    onchange,
  }: {
    entryId: string;
    attachments: AttachmentMeta[];
    onchange?: () => void;
  } = $props();

  let fileInput: HTMLInputElement | undefined = $state();

  async function download(att: AttachmentMeta) {
    try {
      const bytes = await entriesBridge.entryAttachmentSave(entryId, att.name);
      if (bytes === null) return; // user cancelled the Rust-owned native dialog
      toast.success(`Saved ${att.name} (${formatFileSize(bytes)})`);
    } catch (e) {
      toast.error(`Failed to save ${att.name}: ${e}`);
    }
  }

  function pickFile() {
    fileInput?.click();
  }

  async function onFilePicked(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    try {
      // Read file as base64
      const b64 = await fileToBase64(file);
      // Strip data URL prefix if present
      const raw = b64.includes(",") ? b64.split(",")[1] : b64;
      await entriesBridge.entryAttachmentAdd(entryId, file.name, raw);
      toast.success(`Added ${file.name}`);
      onchange?.();
    } catch (e) {
      toast.error(`Failed to add attachment: ${e}`);
    } finally {
      // Reset so the same file can be picked again
      input.value = "";
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
      toast.error(`Failed to remove ${att.name}: ${e}`);
    }
  }

  function fileToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result as string);
      reader.onerror = () => reject(reader.error);
      reader.readAsDataURL(file);
    });
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

  <input type="file" bind:this={fileInput} onchange={onFilePicked} style="display: none" />

  {#if attachments.length > 0}
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
            onclick={() => download(att)}
            aria-label="Download {att.name}"
            title="Download {att.name}"
          >
            <Icon name="download" size={15} />
          </button>
          <button
            class="attachment-remove"
            onclick={() => requestRemove(att)}
            aria-label="Remove {att.name}"
            title="Remove {att.name}"
          >
            <Icon name="trash" size={14} />
          </button>
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
    font-size: 11px;
    color: var(--text-muted);
  }

  .add-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-secondary);
    background: none;
    border: none;
    cursor: pointer;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    transition: background 0.1s;
  }

  .add-btn:hover {
    background: var(--border);
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

  .attachment-download,
  .attachment-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    background: none;
    border: none;
    cursor: pointer;
  }

  .attachment-download:hover {
    background: var(--border);
  }

  .attachment-remove:hover {
    background: var(--danger-bg);
    color: var(--danger);
  }

  .attachments-empty {
    font-size: 12px;
    color: var(--text-muted);
    font-style: italic;
  }
</style>
