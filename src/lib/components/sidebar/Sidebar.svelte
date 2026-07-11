<script lang="ts">
  import type { ItemType } from "$lib/bridge/types";
  import { selection } from "$lib/stores/selection.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import SidebarItem from "./SidebarItem.svelte";
  import SidebarSection from "./SidebarSection.svelte";

  let allCount = $derived(vault.entries.length);
  let favoritesCount = $derived(vault.entries.filter((e) => e.favorite).length);
  let loginCount = $derived(vault.entries.filter((e) => e.type === "login").length);
  let noteCount = $derived(vault.entries.filter((e) => e.type === "note").length);
  let identityCount = $derived(vault.entries.filter((e) => e.type === "identity").length);
  let cardCount = $derived(vault.entries.filter((e) => e.type === "card").length);

  let tags = $derived([...new Set(vault.entries.flatMap((e) => e.tags))].sort());

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
  </SidebarSection>

  <SidebarSection label="Types">
    <SidebarItem
      label="Logins"
      icon="key"
      count={loginCount}
      selected={isSelected("type", "login")}
      onclick={() => selectFilter({ kind: "type", type: "login" })}
    />
    <SidebarItem
      label="Notes"
      icon="notes"
      count={noteCount}
      selected={isSelected("type", "note")}
      onclick={() => selectFilter({ kind: "type", type: "note" })}
    />
    <SidebarItem
      label="Identities"
      icon="user"
      count={identityCount}
      selected={isSelected("type", "identity")}
      onclick={() => selectFilter({ kind: "type", type: "identity" })}
    />
    <SidebarItem
      label="Cards"
      icon="credit-card"
      count={cardCount}
      selected={isSelected("type", "card")}
      onclick={() => selectFilter({ kind: "type", type: "card" })}
    />
  </SidebarSection>

  {#if tags.length > 0}
    <SidebarSection label="Tags">
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
    width: var(--sidebar-width);
    background: var(--surface-1);
    border-right: 0.5px solid var(--border);
    padding: 14px 8px;
    overflow-y: auto;
  }
</style>
