<script lang="ts">
  import { onMount } from "svelte";
  import type { ItemType } from "$lib/bridge/types";
  import { ENTRY_TYPES } from "$lib/entryTypes";
  import { selection } from "$lib/stores/selection.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import SidebarItem from "./SidebarItem.svelte";
  import SidebarSection from "./SidebarSection.svelte";

  let activeEntries = $derived(vault.entries.filter((e) => !e.trashed));
  let allCount = $derived(activeEntries.length);
  let favoritesCount = $derived(activeEntries.filter((e) => e.favorite).length);
  let trashCount = $derived(vault.entries.filter((e) => e.trashed).length);
  let typeCounts = $derived.by(() => {
    const counts: Partial<Record<ItemType, number>> = {};
    for (const entry of activeEntries) {
      counts[entry.type] = (counts[entry.type] ?? 0) + 1;
    }
    return counts;
  });

  const TAGS_COLLAPSED_KEY = "kagi:sidebar-tags-collapsed";

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

  function isSelected(kind: "all" | "favorites" | "trash"): boolean;
  function isSelected(kind: "type", type: ItemType): boolean;
  function isSelected(kind: "tag", tag: string): boolean;
  function isSelected(kind: string, value?: string): boolean {
    const f = selection.filter;
    if (!value) return f.kind === kind;
    if (kind === "type") return f.kind === "type" && f.type === value;
    if (kind === "tag") return f.kind === "tag" && f.tag === value;
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
      label="Recycle Bin"
      icon="trash"
      count={trashCount}
      selected={isSelected("trash")}
      onclick={() => selectFilter({ kind: "trash" })}
    />
  </SidebarSection>

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

<style>
  .sidebar {
    width: 100%;
    background: var(--surface-1);
    padding: 14px 8px;
    overflow-y: auto;
  }
</style>
