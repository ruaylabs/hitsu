<script lang="ts">
  import { onMount } from "svelte";
  import type { FolderSummary, ItemType } from "$lib/bridge/types";
  import { ENTRY_TYPES } from "$lib/entryTypes";
  import { features } from "$lib/stores/features.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import Button from "../ui/Button.svelte";
  import Dialog from "../ui/Dialog.svelte";
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
      folderError = error instanceof Error ? error.message : String(error);
    } finally {
      savingFolder = false;
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

  const tagColors: Record<string, string> = {
    work: "var(--tag-work)",
    personal: "var(--tag-personal)",
    critical: "var(--tag-critical)",
    homelab: "var(--tag-homelab)",
  };

  function tagColor(tag: string): string {
    return tagColors[tag] || "var(--text-muted)";
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
        />
      {/each}
    </SidebarSection>
  {/if}
</aside>

{#if folderDialog}
  <Dialog
    title={folderDialog.mode === "rename"
      ? "Rename folder"
      : folderDialog.parentName
        ? `New folder in ${folderDialog.parentName}`
        : "New folder"}
    size="sm"
    onclose={() => (folderDialog = null)}
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

    {#snippet footer()}
      <Button onclick={() => (folderDialog = null)}>Cancel</Button>
      <Button variant="primary" onclick={saveFolder} disabled={savingFolder || !folderName.trim()}>
        {savingFolder ? "Saving…" : folderDialog?.mode === "rename" ? "Rename" : "Create"}
      </Button>
    {/snippet}
  </Dialog>
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
