<script lang="ts">
  import { untrack } from "svelte";
  import * as entriesBridge from "$lib/bridge/entries";
  import type { Entry } from "$lib/bridge/types";
  import { toast } from "$lib/stores/toast.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { errorMessage } from "$lib/utils/errorMessage";
  import Button from "../ui/Button.svelte";
  import Dialog from "../ui/Dialog.svelte";
  import Icon from "../ui/Icon.svelte";

  let {
    entry,
    onclose,
    onmove,
  }: {
    entry: Entry;
    onclose: () => void;
    onmove: (updated: Entry) => void;
  } = $props();

  let moveFolderId = $state(untrack(() => entry.folderId ?? ""));
  let movingEntry = $state(false);
  let moveError = $state("");
  let addingMoveFolder = $state(false);
  let newMoveFolderName = $state("");
  let creatingMoveFolder = $state(false);
  let newMoveFolderError = $state("");

  function folderPath(folderId: string): string {
    const names: string[] = [];
    const seen = new Set<string>();
    let current = vault.folders.find((f) => f.id === folderId);
    while (current && !seen.has(current.id)) {
      seen.add(current.id);
      names.unshift(current.name);
      current = current.parentId
        ? vault.folders.find((f) => f.id === current?.parentId)
        : undefined;
    }
    return names.join(" / ");
  }

  let moveFolders = $derived(
    [...vault.folders].sort((left, right) =>
      folderPath(left.id).localeCompare(folderPath(right.id)),
    ),
  );

  async function createMoveFolder() {
    const name = newMoveFolderName.trim();
    if (!name || creatingMoveFolder) return;
    creatingMoveFolder = true;
    newMoveFolderError = "";
    try {
      const folder = await vault.createFolder(moveFolderId || null, name);
      moveFolderId = folder.id;
      addingMoveFolder = false;
      newMoveFolderName = "";
    } catch (error) {
      newMoveFolderError = errorMessage(error);
    } finally {
      creatingMoveFolder = false;
    }
  }

  async function moveEntry() {
    if (movingEntry) return;
    movingEntry = true;
    moveError = "";
    try {
      const updated = await entriesBridge.entryMove(entry.id, moveFolderId || null);
      onmove(updated);
      toast.success("Entry moved");
    } catch (error) {
      moveError = errorMessage(error);
    } finally {
      movingEntry = false;
    }
  }
</script>

<Dialog
  title="Move entry"
  {onclose}
  onconfirm={addingMoveFolder ? createMoveFolder : moveEntry}
  size="sm"
>
  <div class="move-entry-content">
    <div class="move-destination-heading">
      <label for="move-folder">Destination</label>
      <button
        type="button"
        class="new-folder-button"
        onclick={() => {
          addingMoveFolder = !addingMoveFolder;
          newMoveFolderName = "";
          newMoveFolderError = "";
        }}
      >
        <Icon name="folder-plus" size={13} />
        New folder
      </button>
    </div>
    <select
      id="move-folder"
      class="control control--compact control--select"
      bind:value={moveFolderId}
    >
      <option value="">Vault root</option>
      {#each moveFolders as folder (folder.id)}
        <option value={folder.id}>{folderPath(folder.id)}</option>
      {/each}
    </select>
    {#if addingMoveFolder}
      <div class="new-folder-form">
        <label class="control-label" for="new-move-folder">
          Create in {moveFolderId ? folderPath(moveFolderId) : "Vault root"}
        </label>
        <div class="new-folder-row">
          <!-- svelte-ignore a11y_autofocus -->
          <input
            id="new-move-folder"
            class="control control--compact"
            bind:value={newMoveFolderName}
            placeholder="Folder name"
            autocomplete="off"
            autofocus
            onkeydown={(event) => {
              if (event.key === "Enter") {
                event.stopPropagation();
                void createMoveFolder();
              }
            }}
          />
          <Button
            variant="outline"
            onclick={createMoveFolder}
            disabled={creatingMoveFolder || !newMoveFolderName.trim()}
          >
            {creatingMoveFolder ? "Creating…" : "Create"}
          </Button>
        </div>
        {#if newMoveFolderError}
          <p class="control-error">{newMoveFolderError}</p>
        {/if}
      </div>
    {/if}
    {#if moveError}
      <p class="save-error">{moveError}</p>
    {/if}
  </div>

  {#snippet footer()}
    <Button onclick={onclose}>Cancel</Button>
    <Button
      variant="primary"
      onclick={moveEntry}
      disabled={movingEntry || moveFolderId === (entry.folderId ?? "")}
    >
      {movingEntry ? "Moving…" : "Move"}
    </Button>
  {/snippet}
</Dialog>

<style>
  .move-entry-content {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .move-destination-heading {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .move-destination-heading label {
    font-weight: 600;
    font-size: 13px;
    color: var(--text-primary);
  }

  .new-folder-button {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-accent);
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 4px;
  }

  .new-folder-button:hover {
    background: var(--border);
  }

  .new-folder-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .control-label {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .new-folder-row {
    display: flex;
    gap: 8px;
  }

  .new-folder-row .control {
    flex: 1;
  }

  .new-folder-row :global(button) {
    white-space: nowrap;
  }

  .control-error,
  .save-error {
    margin: 0;
    font-size: 12px;
    color: var(--danger);
  }
</style>
