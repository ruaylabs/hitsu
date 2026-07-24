<script lang="ts">
  import { tick } from "svelte";
  import * as entriesBridge from "$lib/bridge/entries";
  import type { EntrySummary } from "$lib/bridge/types";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { entryDeletion } from "$lib/stores/entryDeletion.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { openHttpUrl } from "$lib/utils/openHttpUrl";
  import { entryHaystack } from "$lib/utils/search";
  import Icon from "../ui/Icon.svelte";
  import ItemListRow from "./ItemListRow.svelte";
  import SearchField from "./SearchField.svelte";

  let { onCreate = () => {} }: { onCreate?: () => void } = $props();

  function modifiedTime(entry: EntrySummary): number {
    const timestamp = Date.parse(entry.modifiedAt ?? "");
    return Number.isNaN(timestamp) ? 0 : timestamp;
  }

  function compareModified(left: EntrySummary, right: EntrySummary): number {
    return modifiedTime(right) - modifiedTime(left);
  }

  interface RowContextMenu {
    entry: EntrySummary;
    x: number;
    y: number;
  }

  let contextMenu = $state<RowContextMenu | null>(null);
  let contextMenuEl = $state<HTMLDivElement | undefined>();
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
    } else if (f.kind === "recent") {
      items = [...items].sort(compareModified).slice(0, 20);
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

  function reportActionError(error: unknown) {
    toast.error(error instanceof Error ? error.message : String(error));
  }

  async function copyUsername(entry: EntrySummary) {
    if (!entry.username) return;
    try {
      await clipboard.copyPlain(entry.username);
      toast.success("Username copied");
    } catch (error) {
      reportActionError(error);
    }
  }

  async function copySecret(entry: EntrySummary, field: "password" | "totp", label: string) {
    try {
      await clipboard.copySecretField(entry.id, field);
      toast.success(`${label} copied`);
    } catch (error) {
      reportActionError(error);
    }
  }

  function openContextMenu(event: MouseEvent, entry: EntrySummary) {
    event.preventDefault();
    const x = Math.max(8, Math.min(event.clientX, window.innerWidth - 208));
    const y = Math.max(8, Math.min(event.clientY, window.innerHeight - 258));
    selection.requestNavigation(() => {
      selection.selectedId = entry.id;
      contextMenu = { entry, x, y };
      void tick().then(() => {
        contextMenuEl?.querySelector<HTMLButtonElement>("button:not(:disabled)")?.focus();
      });
    });
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function runContextAction(action: () => void) {
    action();
    closeContextMenu();
  }

  function onContextMenuKeydown(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Escape" || event.key === "Tab") {
      closeContextMenu();
      return;
    }
    const items = Array.from(
      contextMenuEl?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? [],
    );
    if (items.length === 0) return;
    const current = items.indexOf(document.activeElement as HTMLButtonElement);
    let next = current;
    if (event.key === "ArrowDown") next = (current + 1) % items.length;
    else if (event.key === "ArrowUp") next = (current - 1 + items.length) % items.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = items.length - 1;
    else return;
    event.preventDefault();
    items[next]?.focus();
  }

  function onWindowClick(event: MouseEvent) {
    if (contextMenu && !(event.target as Element | null)?.closest(".row-context-menu")) {
      closeContextMenu();
    }
  }

  async function onListKeydown(e: KeyboardEvent) {
    if (isTextEditable(document.activeElement)) return;

    if ((e.metaKey || e.ctrlKey) && !e.altKey && e.key.toLowerCase() === "c") {
      const entry = vault.entries.find((item) => item.id === selection.selectedId);
      if (!entry || entry.trashed) return;
      if (e.shiftKey) {
        if (!entry.hasPassword) return;
        e.preventDefault();
        void copySecret(entry, "password", "Password");
      } else {
        if (!entry.username) return;
        e.preventDefault();
        void copyUsername(entry);
      }
      return;
    }

    if (e.metaKey || e.ctrlKey || e.altKey) return;
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

<svelte:window onkeydown={onListKeydown} onclick={onWindowClick} onblur={closeContextMenu} />

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
              oncontextmenu={(event) => openContextMenu(event, entry)}
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

{#if contextMenu}
  {@const menuEntry = contextMenu.entry}
  <div
    class="row-context-menu"
    role="menu"
    tabindex="-1"
    aria-label="Actions for {menuEntry.title}"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    bind:this={contextMenuEl}
    onkeydown={onContextMenuKeydown}
  >
    <button
      type="button"
      role="menuitem"
      disabled={!menuEntry.username}
      onclick={() => runContextAction(() => void copyUsername(menuEntry))}
    >
      <Icon name="user" size={14} />Copy username
    </button>
    <button
      type="button"
      role="menuitem"
      disabled={!menuEntry.hasPassword}
      onclick={() => runContextAction(() => void copySecret(menuEntry, "password", "Password"))}
    >
      <Icon name="key" size={14} />Copy password
    </button>
    <button
      type="button"
      role="menuitem"
      disabled={!menuEntry.hasTotp}
      onclick={() => runContextAction(() => void copySecret(menuEntry, "totp", "TOTP"))}
    >
      <Icon name="clock-code" size={14} />Copy TOTP
    </button>
    <button
      type="button"
      role="menuitem"
      disabled={!menuEntry.url}
      onclick={() => runContextAction(() => openHttpUrl(menuEntry.url ?? ""))}
    >
      <Icon name="external-link" size={14} />Open URL
    </button>
    <div class="context-separator" role="separator"></div>
    <button
      type="button"
      role="menuitem"
      disabled={menuEntry.trashed}
      onclick={() => runContextAction(() => vault.setEditingId(menuEntry.id))}
    >
      <Icon name="pencil" size={14} />Edit
    </button>
    <button
      type="button"
      role="menuitem"
      class="danger-menu-item"
      onclick={() => runContextAction(() => entryDeletion.request(menuEntry.id, menuEntry.title))}
    >
      <Icon name="trash" size={14} />Delete
    </button>
  </div>
{/if}

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
    background: var(--bg-accent);
  }

  .empty-action:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .row-context-menu {
    position: fixed;
    z-index: 1000;
    width: 200px;
    padding: 5px;
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-dialog);
  }

  .row-context-menu button {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 7px 9px;
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    font-size: 12.5px;
    text-align: left;
  }

  .row-context-menu button:hover:not(:disabled),
  .row-context-menu button:focus-visible {
    background: var(--border);
    outline: none;
  }

  .row-context-menu button:disabled {
    color: var(--text-muted);
    opacity: 0.5;
  }

  .row-context-menu .danger-menu-item {
    color: var(--danger);
  }

  .context-separator {
    height: 0.5px;
    margin: 4px 6px;
    background: var(--border);
  }
</style>
