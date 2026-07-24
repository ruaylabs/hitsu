<script lang="ts">
  import { tick } from "svelte";
  import * as entriesBridge from "$lib/bridge/entries";
  import { selection } from "$lib/stores/selection.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { entryHaystack } from "$lib/utils/search";
  import Icon from "../ui/Icon.svelte";
  import ItemListRow from "./ItemListRow.svelte";
  import SearchField from "./SearchField.svelte";

  let { onCreate = () => {} }: { onCreate?: () => void } = $props();

  let searchMatchIds = $state<string[] | null>(null);
  let searchRequest = 0;

  // Full-field search stays in Rust so notes and other field values do not
  // have to be copied into every list summary. Keep the existing summary
  // search as an immediate fallback while the backend result is in flight.
  $effect(() => {
    const query = selection.search.trim();
    void vault.entries;
    const request = ++searchRequest;
    searchMatchIds = null;
    if (!query) return;

    void entriesBridge
      .entriesSearch(query)
      .then((ids) => {
        if (request === searchRequest && selection.search.trim() === query) {
          searchMatchIds = ids;
        }
      })
      .catch(() => {
        // Keep the summary-field fallback when backend search is unavailable.
      });
  });

  let hasActiveEntries = $derived(vault.entries.some((entry) => !entry.trashed));
  let hasMatchesOutsideFilter = $derived.by(() => {
    if (!selection.search || selection.filter.kind === "all") return false;
    const query = selection.search.toLowerCase();
    const matches = searchMatchIds === null ? null : new Set(searchMatchIds);
    return vault.entries.some(
      (entry) =>
        !entry.trashed && (matches?.has(entry.id) === true || entryHaystack(entry).includes(query)),
    );
  });

  let filtered = $derived.by(() => {
    const f = selection.filter;
    let items = vault.entries.filter((e) => (f.kind === "trash" ? e.trashed : !e.trashed));
    if (f.kind === "favorites") {
      items = items.filter((e) => e.favorite);
    } else if (f.kind === "type") {
      items = items.filter((e) => e.type === f.type);
    } else if (f.kind === "tag") {
      items = items.filter((e) => e.tags.includes(f.tag));
    } else if (f.kind === "folder") {
      const folderIds = vault.folderIdsWithin(f.folderId);
      items = items.filter((entry) => entry.folderId && folderIds.has(entry.folderId));
    }
    if (selection.search) {
      if (searchMatchIds === null) {
        const q = selection.search.toLowerCase();
        items = items.filter((e) => entryHaystack(e).includes(q));
      } else {
        const q = selection.search.toLowerCase();
        const matches = new Set(searchMatchIds);
        items = items.filter((entry) => matches.has(entry.id) || entryHaystack(entry).includes(q));
      }
    }
    return items;
  });

  // Windowed rendering: only the rows intersecting the viewport (plus a
  // small overscan) exist in the DOM. Must match .list-row's fixed height.
  const ROW_HEIGHT = 50;
  const OVERSCAN = 5;

  let scrollEl = $state<HTMLDivElement | undefined>();
  let scrollTop = $state(0);
  let viewportHeight = $state(0);

  let startIndex = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN));
  let endIndex = $derived(
    Math.min(filtered.length, Math.ceil((scrollTop + viewportHeight) / ROW_HEIGHT) + OVERSCAN),
  );
  let visible = $derived(filtered.slice(startIndex, endIndex));

  // When the list shrinks or the viewport grows, the browser clamps the
  // container's scrollTop without necessarily firing a scroll event;
  // re-read it so the window doesn't point past the end of the list.
  $effect(() => {
    void filtered.length;
    void viewportHeight;
    if (scrollEl) scrollTop = scrollEl.scrollTop;
  });

  /** Scroll just far enough that row `index` is fully inside the viewport,
   * so it is rendered and focusable. Mirrors what the browser used to do
   * implicitly when focusing an off-screen row. */
  function scrollIndexIntoView(index: number) {
    if (!scrollEl) return;
    const top = index * ROW_HEIGHT;
    const bottom = top + ROW_HEIGHT;
    if (top < scrollEl.scrollTop) {
      scrollEl.scrollTop = top;
    } else if (bottom > scrollEl.scrollTop + scrollEl.clientHeight) {
      scrollEl.scrollTop = bottom - scrollEl.clientHeight;
    }
    scrollTop = scrollEl.scrollTop;
  }

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
      const nextIndex = next;
      selection.requestNavigation(() => {
        selection.select(nextEntry.id, "keyboard");
        // The target row may not be in the DOM yet (windowed rendering), so
        // scroll it into range first, then focus after navigation is allowed
        // (immediately or after the unsaved-changes dialog is resolved).
        scrollIndexIntoView(nextIndex);
        void tick().then(() => {
          document
            .querySelector<HTMLButtonElement>(`[data-entry-id="${nextEntry.id}"]`)
            ?.focus({ preventScroll: true });
        });
      });
    }
  }
</script>

<svelte:window onkeydown={onListKeydown} />

<div class="item-list">
  <SearchField allowCreate={selection.filter.kind !== "trash"} />
  <div
    class="list-rows"
    role="listbox"
    bind:this={scrollEl}
    bind:clientHeight={viewportHeight}
    onscroll={() => { scrollTop = scrollEl?.scrollTop ?? 0; }}
  >
    {#if filtered.length > 0}
      <div
        class="virtual-spacer"
        role="presentation"
        style="height: {filtered.length * ROW_HEIGHT}px"
      >
        <div
          class="virtual-window"
          role="presentation"
          style="transform: translateY({startIndex * ROW_HEIGHT}px)"
        >
          {#each visible as entry (entry.id)}
            <ItemListRow
              {entry}
              selected={entry.id === selection.selectedId}
              onclick={() => selection.requestNavigation(() => { selection.selectedId = entry.id; })}
            />
          {/each}
        </div>
      </div>
    {:else}
      <div class="empty-list">
        {#if selection.search}
          <Icon name="search-off" size={18} />
          <p>No items match "{selection.search}"</p>
          {#if hasMatchesOutsideFilter}
            <button
              type="button"
              class="empty-action"
              onclick={() => selection.requestNavigation(() => { selection.filter = { kind: "all" }; })}
            >
              Search all items
            </button>
          {/if}
        {:else if selection.filter.kind === "trash"}
          <Icon name="trash" size={18} />
          <p>Recycle Bin is empty</p>
        {:else if !hasActiveEntries}
          <Icon name="lock-open" size={18} />
          <p>No entries yet</p>
          <button type="button" class="empty-action" onclick={onCreate}>
            Create your first entry
          </button>
        {:else}
          <Icon name="filter-off" size={18} />
          <p>No entries in this view</p>
        {/if}
      </div>
    {/if}
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

  .virtual-spacer {
    position: relative;
    overflow: hidden;
  }

  .virtual-window {
    will-change: transform;
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

  .empty-list p {
    margin: 0;
  }

  .empty-action {
    padding: 5px 10px;
    color: var(--accent);
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 500;
  }

  .empty-action:hover {
    background: var(--accent-subtle);
  }

  .empty-action:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
