<script lang="ts">
  import { vault } from "$lib/stores/vault.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import SearchField from "./SearchField.svelte";
  import ItemListRow from "./ItemListRow.svelte";
  import Icon from "../ui/Icon.svelte";

  let filtered = $derived.by(() => {
    let items = vault.entries;
    if (selection.filter === "favorites") {
      items = items.filter((e) => e.favorite);
    } else if (selection.filter === "trash") {
      items = [];
    } else if (selection.filter !== "all") {
      const typeFilter = selection.filter;
      const isTypeFilter = ["login", "note", "identity", "card"].includes(typeFilter);
      items = items.filter((e) =>
        isTypeFilter ? e.type === typeFilter : e.tags.includes(typeFilter),
      );
    }
    if (selection.search) {
      const q = selection.search.toLowerCase();
      items = items.filter(
        (e) =>
          e.title.toLowerCase().includes(q) ||
          e.subtitle.toLowerCase().includes(q) ||
          e.url?.toLowerCase().includes(q) ||
          e.username?.toLowerCase().includes(q) ||
          e.tags.some((t) => t.includes(q)),
      );
    }
    return items;
  });

  let hasSelection = $derived(
    filtered.length > 0 && filtered.some((e) => e.id === selection.selectedId),
  );

  // Auto-select first when filter/search changes and nothing is selected
  $effect(() => {
    if (filtered.length > 0 && !hasSelection) {
      selection.selectedId = filtered[0].id;
    }
  });
</script>

<div class="item-list">
  <SearchField />
  <div class="list-rows" role="listbox">
    {#each filtered as entry (entry.id)}
      <ItemListRow
        {entry}
        selected={entry.id === selection.selectedId}
        onclick={() => { selection.selectedId = entry.id; }}
      />
    {:else}
      <div class="empty-list">
        {#if selection.search}
          <Icon name="search-off" size={18} />
          <p>No items match "{selection.search}"</p>
        {:else}
          <Icon name="lock-open" size={18} />
          <p>No entries yet</p>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .item-list {
    width: var(--list-width);
    border-right: 0.5px solid var(--border);
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .list-rows {
    flex: 1;
    overflow-y: auto;
  }

  .empty-list {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 32px 16px;
    color: var(--text-muted);
    font-size: 13px;
    text-align: center;
  }
</style>
