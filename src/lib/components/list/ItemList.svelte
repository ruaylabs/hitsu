<script lang="ts">
  import { tick } from "svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import Icon from "../ui/Icon.svelte";
  import ItemListRow from "./ItemListRow.svelte";
  import SearchField from "./SearchField.svelte";

  let filtered = $derived.by(() => {
    let items = vault.entries;
    const f = selection.filter;
    if (f.kind === "favorites") {
      items = items.filter((e) => e.favorite);
    } else if (f.kind === "trash") {
      items = [];
    } else if (f.kind === "type") {
      items = items.filter((e) => e.type === f.type);
    } else if (f.kind === "tag") {
      items = items.filter((e) => e.tags.includes(f.tag));
    }
    if (selection.search) {
      const q = selection.search.toLowerCase();
      items = items.filter(
        (e) =>
          e.title.toLowerCase().includes(q) ||
          e.subtitle.toLowerCase().includes(q) ||
          e.url?.toLowerCase().includes(q) ||
          e.username?.toLowerCase().includes(q) ||
          e.tags.some((t) => t.toLowerCase().includes(q)),
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
      selection.requestNavigation(() => {
        selection.selectedId = filtered[0].id;
      });
    }
  });

  // Keyboard navigation: ↑/↓ move selection, Home/End jump to ends.
  // Roving tabindex keeps the selected row Tab-reachable; arrow keys move
  // both selection and DOM focus within the listbox.
  //
  // Bound at the window level (not the listbox) so navigation works even
  // before the user clicks a row — on launch the first entry is auto-selected
  // but not focused, so a div-scoped handler would never fire. We bail out
  // when focus is in a text-editable element so search/caret editing is
  // unaffected, and when a modifier key is held (letting ⌘F etc. through).
  function isTextEditable(el: Element | null): boolean {
    if (!(el instanceof HTMLElement)) return false;
    const tag = el.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
    return el.isContentEditable;
  }

  async function onListKeydown(e: KeyboardEvent) {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    if (isTextEditable(document.activeElement)) return;
    if (filtered.length === 0) return;
    const current = filtered.findIndex((item) => item.id === selection.selectedId);
    let next = current;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      next = current < 0 ? 0 : Math.min(current + 1, filtered.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      next = current < 0 ? 0 : Math.max(current - 1, 0);
    } else if (e.key === "Home") {
      e.preventDefault();
      next = 0;
    } else if (e.key === "End") {
      e.preventDefault();
      next = filtered.length - 1;
    } else {
      return;
    }

    if (next !== current && filtered[next]) {
      const nextEntry = filtered[next];
      selection.requestNavigation(() => {
        selection.selectedId = nextEntry.id;
        // Focus only after navigation is allowed (immediately or after the
        // unsaved-changes dialog is resolved).
        void tick().then(() => {
          document.querySelector<HTMLButtonElement>(`[data-entry-id="${nextEntry.id}"]`)?.focus();
        });
      });
    }
  }
</script>

<svelte:window onkeydown={onListKeydown} />

<div class="item-list">
  <SearchField />
  <div class="list-rows" role="listbox">
    {#each filtered as entry (entry.id)}
      <ItemListRow
        {entry}
        selected={entry.id === selection.selectedId}
        onclick={() => selection.requestNavigation(() => { selection.selectedId = entry.id; })}
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
    width: 100%;
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
