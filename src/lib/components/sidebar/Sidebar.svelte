<script lang="ts">
  import { vault } from "$lib/stores/vault.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import SidebarSection from "./SidebarSection.svelte";
  import SidebarItem from "./SidebarItem.svelte";
  import type { ItemType } from "$lib/bridge/types";

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
    selection.filter = filter;
  }
</script>

<aside class="sidebar">
  <SidebarSection label="Groups">
    <SidebarItem
      label="All items"
      icon="layout-list"
      count={allCount}
      selected={selection.filter === "all"}
      onclick={() => selectFilter("all")}
    />
    <SidebarItem
      label="Favorites"
      icon="star"
      count={favoritesCount}
      selected={selection.filter === "favorites"}
      onclick={() => selectFilter("favorites")}
    />
  </SidebarSection>

  <SidebarSection label="Types">
    <SidebarItem
      label="Logins"
      icon="key"
      count={loginCount}
      selected={selection.filter === "login"}
      onclick={() => selectFilter("login") as void}
    />
    <SidebarItem
      label="Notes"
      icon="notes"
      count={noteCount}
      selected={selection.filter === "note"}
      onclick={() => selectFilter("note") as void}
    />
    <SidebarItem
      label="Identities"
      icon="user"
      count={identityCount}
      selected={selection.filter === "identity"}
      onclick={() => selectFilter("identity") as void}
    />
    <SidebarItem
      label="Cards"
      icon="credit-card"
      count={cardCount}
      selected={selection.filter === "card"}
      onclick={() => selectFilter("card") as void}
    />
  </SidebarSection>

  {#if tags.length > 0}
    <SidebarSection label="Tags">
      {#each tags as tag}
        <SidebarItem
          label={tag}
          tagColor={tagColor(tag)}
          onclick={() => selectFilter(tag as ItemType)}
          selected={selection.filter === tag}
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
