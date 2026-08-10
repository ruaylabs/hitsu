<script lang="ts">
  import { onMount } from "svelte";
  import type { FolderSummary, ItemType } from "$lib/bridge/types";
  import { ENTRY_TYPES } from "$lib/entryTypes";
  import { features } from "$lib/stores/features.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { errorMessage } from "$lib/utils/errorMessage";
  import { tagColor } from "$lib/utils/tagColor";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";
  import FormDialog from "../ui/FormDialog.svelte";
  import SidebarItem from "./SidebarItem.svelte";
  import SidebarSection from "./SidebarSection.svelte";

  let activeEntries = $derived(vault.entries.filter((e) => !e.trashed));
  let allCount = $derived(activeEntries.length);
  let favoritesCount = $derived(activeEntries.filter((e) => e.favorite).length);
  let recentCount = $derived(Math.min(activeEntries.length, 20));
  let trashCount = $derived(vault.entries.filter((e) => e.trashed).length);
  type FolderRow = FolderSummary & { depth: number };

  function flattenFolders(folders: FolderSummary[]): FolderRow[] {
    const rows: FolderRow[] = [];
    const seen = new Set<string>();
    const visit = (parentId: string | undefined, depth: number) => {
      for (const folder of folders
        .filter((candidate) => candidate.parentId === parentId)
        .sort((left, right) => left.name.localeCompare(right.name))) {
        if (seen.has(folder.id)) continue;
        seen.add(folder.id);
        rows.push({ ...folder, depth });
        visit(folder.id, depth + 1);
      }
    };
    visit(undefined, 0);
    for (const folder of folders) {
      if (!seen.has(folder.id)) rows.push({ ...folder, depth: 0 });
    }
    return rows;
  }

  let folderRows = $derived(flattenFolders(vault.folders));
  let folderDialog = $state<
    | { mode: "create"; parentId?: string; parentName?: string }
    | { mode: "rename"; folder: FolderSummary }
    | null
  >(null);
  let folderName = $state("");
  let folderError = $state("");
  let savingFolder = $state(false);

  // Tag rename dialog
  let tagRenameDialog = $state<{ oldName: string } | null>(null);
  let tagRenameInput = $state("");
  let tagRenameError = $state("");
  let savingTag = $state(false);
  let tagDeletePrompt = $state<string | null>(null);

  function openCreateFolder(parent?: FolderSummary) {
    folderName = "";
    folderError = "";
    folderDialog = {
      mode: "create",
      parentId: parent?.id,
      parentName: parent?.name,
    };
  }

  function openRenameFolder(folder: FolderSummary) {
    folderName = folder.name;
    folderError = "";
    folderDialog = { mode: "rename", folder };
  }

  async function saveFolder() {
    const action = folderDialog;
    const name = folderName.trim();
    if (!action || !name || savingFolder) return;
    savingFolder = true;
    folderError = "";
    try {
      if (action.mode === "create") {
        await vault.createFolder(action.parentId ?? null, name);
      } else {
        await vault.renameFolder(action.folder.id, name);
      }
      folderDialog = null;
    } catch (error) {
      folderError = errorMessage(error);
    } finally {
      savingFolder = false;
    }
  }

  function openRenameTag(tag: string) {
    tagRenameDialog = { oldName: tag };
    tagRenameInput = tag;
    tagRenameError = "";
  }

  async function saveTagRename() {
    const dialog = tagRenameDialog;
    const newName = tagRenameInput.trim();
    if (!dialog || !newName || savingTag || newName === dialog.oldName) return;
    savingTag = true;
    tagRenameError = "";
    try {
      await vault.renameTag(dialog.oldName, newName);
      tagRenameDialog = null;
    } catch (error) {
      tagRenameError = errorMessage(error);
    } finally {
      savingTag = false;
    }
  }

  async function confirmDeleteTag() {
    const tag = tagDeletePrompt;
    if (!tag) return;
    tagDeletePrompt = null;
    try {
      await vault.deleteTag(tag);
    } catch (error) {
      console.error("Failed to delete tag", error);
    }
  }

  function folderCount(folderId: string) {
    const folderIds = vault.folderIdsWithin(folderId);
    return activeEntries.filter((entry) => entry.folderId && folderIds.has(entry.folderId)).length;
  }

  let typeCounts = $derived.by(() => {
    const counts: Partial<Record<ItemType, number>> = {};
    for (const entry of activeEntries) {
      counts[entry.type] = (counts[entry.type] ?? 0) + 1;
    }
    return counts;
  });

  const TAGS_COLLAPSED_KEY = "hitsu:sidebar-tags-collapsed";

  let tags = $derived([...new Set(activeEntries.flatMap((e) => e.tags))].sort());
  let tagsCollapsed = $state(false);

  onMount(() => {
    try {
      tagsCollapsed = localStorage.getItem(TAGS_COLLAPSED_KEY) === "true";
    } catch {
      // Sidebar persistence is optional.
    }
  });

  function toggleTags() {
    tagsCollapsed = !tagsCollapsed;
    try {
      localStorage.setItem(TAGS_COLLAPSED_KEY, String(tagsCollapsed));
    } catch {
      // Sidebar persistence is optional.
    }
  }

  function selectFilter(filter: typeof selection.filter) {
    selection.requestNavigation(() => {
      selection.filter = filter;
    });
  }

  function isSelected(kind: "all" | "favorites" | "recent" | "trash"): boolean;
  function isSelected(kind: "type", type: ItemType): boolean;
  function isSelected(kind: "tag", tag: string): boolean;
  function isSelected(kind: "folder", folderId: string): boolean;
  function isSelected(kind: string, value?: string): boolean {
    const f = selection.filter;
    if (!value) return f.kind === kind;
    if (kind === "type") return f.kind === "type" && f.type === value;
    if (kind === "tag") return f.kind === "tag" && f.tag === value;
    if (kind === "folder") return f.kind === "folder" && f.folderId === value;
    return false;
  }
</script>

<aside class="sidebar">
  <SidebarSection label="Groups">
    <SidebarItem
      label="All items"
      icon="layout-list"
      count={allCount}
      selected={isSelected("all")}
      onclick={() => selectFilter({ kind: "all" })}
    />
    <SidebarItem
      label="Favorites"
      icon="star"
      count={favoritesCount}
      selected={isSelected("favorites")}
      onclick={() => selectFilter({ kind: "favorites" })}
    />
    <SidebarItem
      label="Recent"
      icon="clock"
      count={recentCount}
      selected={isSelected("recent")}
      onclick={() => selectFilter({ kind: "recent" })}
    />
    <SidebarItem
      label="Recycle Bin"
      icon="trash"
      count={trashCount}
      selected={isSelected("trash")}
      onclick={() => selectFilter({ kind: "trash" })}
    />
  </SidebarSection>

  {#if features.foldersEnabled}
    <SidebarSection label="Folders" onadd={() => openCreateFolder()} addLabel="Add root folder">
      {#each folderRows as folder (folder.id)}
        <SidebarItem
          label={folder.name}
          icon="folder"
          count={folderCount(folder.id)}
          indent={folder.depth}
          selected={isSelected("folder", folder.id)}
          onclick={() => selectFilter({ kind: "folder", folderId: folder.id })}
          onadd={() => openCreateFolder(folder)}
          onedit={() => openRenameFolder(folder)}
        />
      {/each}
    </SidebarSection>
  {/if}

  <SidebarSection label="Types">
    {#each ENTRY_TYPES as item (item.type)}
      <SidebarItem
        label={item.pluralLabel}
        icon={item.icon}
        count={typeCounts[item.type] ?? 0}
        selected={isSelected("type", item.type)}
        onclick={() => selectFilter({ kind: "type", type: item.type })}
      />
    {/each}
  </SidebarSection>

  {#if tags.length > 0}
    <SidebarSection label="Tags" collapsed={tagsCollapsed} ontoggle={toggleTags}>
      {#each tags as tag}
        <SidebarItem
          label={tag}
          tagColor={tagColor(tag)}
          onclick={() => selectFilter({ kind: "tag", tag })}
          selected={isSelected("tag", tag)}
          onedit={() => openRenameTag(tag)}
          ondelete={() => (tagDeletePrompt = tag)}
        />
      {/each}
    </SidebarSection>
  {/if}
</aside>

{#if folderDialog}
  <FormDialog
    title={folderDialog.mode === "rename"
      ? "Rename folder"
      : folderDialog.parentName
        ? `New folder in ${folderDialog.parentName}`
        : "New folder"}
    confirmLabel={folderDialog.mode === "rename" ? "Rename" : "Create"}
    busy={savingFolder}
    disabled={!folderName.trim()}
    oncancel={() => (folderDialog = null)}
    onconfirm={saveFolder}
  >
    <div class="folder-form">
      <label class="control-label" for="folder-name">Name</label>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        id="folder-name"
        class="control control--compact"
        bind:value={folderName}
        autocomplete="off"
        autofocus
      />
      {#if folderError}
        <p class="control-error">{folderError}</p>
      {/if}
    </div>
  </FormDialog>
{/if}

{#if tagRenameDialog}
  <FormDialog
    title="Rename tag"
    confirmLabel="Rename"
    busy={savingTag}
    disabled={!tagRenameInput.trim() || tagRenameInput.trim() === tagRenameDialog.oldName}
    oncancel={() => (tagRenameDialog = null)}
    onconfirm={saveTagRename}
  >
    <div class="folder-form">
      <label class="control-label" for="tag-rename">
        Rename <strong>{tagRenameDialog.oldName}</strong> across all entries
      </label>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        id="tag-rename"
        class="control control--compact"
        bind:value={tagRenameInput}
        autocomplete="off"
        autofocus
      />
      {#if tagRenameError}
        <p class="control-error">{tagRenameError}</p>
      {/if}
    </div>
  </FormDialog>
{/if}

{#if tagDeletePrompt}
  <ConfirmDialog
    title="Delete tag"
    message={`Remove ${tagDeletePrompt} from all entries? This cannot be undone.`}
    confirmLabel="Delete"
    danger
    oncancel={() => (tagDeletePrompt = null)}
    onconfirm={confirmDeleteTag}
  />
{/if}

<style>
  .sidebar {
    width: 100%;
    background: var(--surface-1);
    padding: 14px 8px;
    overflow-y: auto;
  }

  .folder-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
</style>
