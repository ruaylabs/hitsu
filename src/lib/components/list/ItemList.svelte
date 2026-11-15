<script lang="ts">
  import { LogicalPosition } from "@tauri-apps/api/dpi";
  import { Menu } from "@tauri-apps/api/menu";
  import { onDestroy, tick } from "svelte";
  import * as entriesBridge from "$lib/bridge/entries";
  import type { EntrySummary } from "$lib/bridge/types";
  import { ENTRY_TYPE_BY_TYPE } from "$lib/entryTypes";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { entryDeletion } from "$lib/stores/entryDeletion.svelte";
  import { health } from "$lib/stores/health.svelte";
  import { recycleBin } from "$lib/stores/recycleBin.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { runCopyFeedback } from "$lib/utils/copyFeedback.svelte";
  import { createEntrySearch } from "$lib/utils/entrySearch.svelte";
  import { errorMessage } from "$lib/utils/errorMessage";
  import { openHttpUrl } from "$lib/utils/openHttpUrl";
  import Button from "../ui/Button.svelte";
  import Icon from "../ui/Icon.svelte";
  import ItemListRow from "./ItemListRow.svelte";
  import SearchField from "./SearchField.svelte";

  let { onCreate = () => {} }: { onCreate?: () => void } = $props();

  type SortMode = "vault" | "title" | "modified";
  const SORT_MODES: SortMode[] = ["vault", "title", "modified"];
  const SORT_MODE_KEY = "hitsu:list-sort";
  const HEALTH_EMPTY_MESSAGES = {
    weak: "No weak passwords",
    reused: "No reused passwords",
  } as const;

  function loadSortMode(): SortMode {
    try {
      const saved = localStorage.getItem(SORT_MODE_KEY) as SortMode | null;
      return saved && SORT_MODES.includes(saved) ? saved : "vault";
    } catch {
      return "vault";
    }
  }

  let sortMode = $state<SortMode>(loadSortMode());

  $effect(() => {
    try {
      localStorage.setItem(SORT_MODE_KEY, sortMode);
    } catch {
      // Sort persistence is optional.
    }
  });

  function modifiedTime(entry: EntrySummary): number {
    const timestamp = Date.parse(entry.modifiedAt ?? "");
    return Number.isNaN(timestamp) ? 0 : timestamp;
  }

  function compareModified(left: EntrySummary, right: EntrySummary): number {
    return modifiedTime(right) - modifiedTime(left);
  }

  let contextMenu: Menu | null = null;
  let contextMenuGeneration = 0;

  onDestroy(() => {
    contextMenuGeneration++;
    void contextMenu?.close().catch(reportActionError);
  });
  const entrySearch = createEntrySearch(() => selection.search, {
    refreshOn: () => {
      void vault.entries;
    },
  });

  let hasActiveEntries = $derived(vault.entries.some((entry) => !entry.trashed));

  let filtered = $derived.by(() => {
    const f = selection.filter;
    let items = vault.entries.filter((e) => (f.kind === "trash" ? e.trashed : !e.trashed));
    if (f.kind === "favorites") {
      items = items.filter((e) => e.favorite);
    } else if (f.kind === "recent") {
      items = [...items].sort(compareModified).slice(0, 20);
    } else if (f.kind === "health") {
      const ids = new Set(health.ids(f.issue));
      items = items.filter((entry) => ids.has(entry.id));
    } else if (f.kind === "type") {
      items = items.filter((e) => e.type === f.type);
    } else if (f.kind === "tag") {
      items = items.filter((e) => e.tags.includes(f.tag));
    } else if (f.kind === "folder") {
      const folderIds = vault.folderIdsWithin(f.folderId);
      items = items.filter((entry) => entry.folderId && folderIds.has(entry.folderId));
    }
    if (selection.search) {
      items = items.filter((entry) => entrySearch.matches(entry));
    }
    if (sortMode === "title") {
      items = [...items].sort((left, right) =>
        left.title.localeCompare(right.title, undefined, { sensitivity: "base" }),
      );
    } else if (sortMode === "modified") {
      items = [...items].sort(compareModified);
    }
    return items;
  });

  // Windowed rendering: only the rows intersecting the viewport (plus a
  // small overscan) exist in the DOM. Must match --row-lg, which .list-row
  // renders at — rowHeight.test.ts fails if the two drift apart.
  const ROW_HEIGHT = 58;
  const OVERSCAN = 5;

  let scrollEl = $state<HTMLDivElement | undefined>();
  let scrollTop = $state(0);
  let viewportHeight = $state(0);
  let searchContext: {
    selectedId: string | null;
    scrollTop: number;
    filter: typeof selection.filter;
  } | null = null;

  function rememberSearchContext() {
    if (searchContext) return;
    searchContext = {
      selectedId: selection.selectedId,
      scrollTop: scrollEl?.scrollTop ?? 0,
      filter: selection.filter,
    };
  }

  function restoreSearchContext() {
    const context = searchContext;
    if (!context) return;
    selection.requestNavigation(() => {
      searchContext = null;
      if (context.selectedId && filtered.some((entry) => entry.id === context.selectedId)) {
        selection.selectedId = context.selectedId;
      }
      if (selection.filter !== context.filter) return;
      void tick().then(() => {
        if (!scrollEl) return;
        scrollEl.scrollTop = context.scrollTop;
        scrollTop = context.scrollTop;
      });
    });
  }

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

  // Keep the detail selection in sync with the current filter/search.
  $effect(() => {
    if (hasSelection) return;
    const nextId = filtered[0]?.id ?? null;
    if (selection.selectedId === nextId) return;
    selection.requestNavigation(() => {
      selection.selectedId = nextId;
    });
  });

  // Keyboard navigation: ↑/↓ move selection, Home/End jump to ends.
  // Roving tabindex keeps the selected row Tab-reachable; arrow keys move
  // both selection and DOM focus within the listbox.
  //
  // Bound at the window level (not the listbox) so navigation works even
  // before the user clicks a row — on launch the first entry is auto-selected
  // but not focused, so a div-scoped handler would never fire. We bail out
  // when focus is in a text-editable element so search/caret editing is
  // unaffected, and when a modifier key is held (letting Cmd/Ctrl+F
  // etc. through).
  function isTextEditable(el: Element | null): boolean {
    if (!(el instanceof HTMLElement)) return false;
    const tag = el.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
    return el.isContentEditable;
  }

  function reportActionError(error: unknown) {
    toast.error(errorMessage(error));
  }

  async function copyUsername(entry: EntrySummary) {
    if (!entry.username) return;
    await runCopyFeedback(() => clipboard.copyPlain(entry.username!), "Username copied");
  }

  async function copySecret(entry: EntrySummary, field: "password" | "totp", label: string) {
    await runCopyFeedback(() => clipboard.copySecretField(entry.id, field), `${label} copied`);
  }

  async function toggleFavorite(entry: EntrySummary) {
    try {
      const updated = await entriesBridge.entryUpdate(entry.id, { favorite: !entry.favorite });
      vault.setEntries(
        vault.entries.map((summary) =>
          summary.id === updated.id ? entriesBridge.toSummary(updated) : summary,
        ),
      );
      void health.refresh();
    } catch (error) {
      reportActionError(error);
    }
  }

  async function showContextMenu(entry: EntrySummary, position: LogicalPosition) {
    const generation = ++contextMenuGeneration;
    const previous = contextMenu;
    contextMenu = null;
    try {
      await previous?.close();
      if (generation !== contextMenuGeneration) return;
      const action = (run: () => void) => () => {
        if (generation === contextMenuGeneration) run();
      };
      const menu = await Menu.new({
        items: [
          {
            id: "copy-username",
            text: "Copy username",
            enabled: !!entry.username,
            action: action(() => void copyUsername(entry)),
          },
          {
            id: "copy-password",
            text: "Copy password",
            enabled: !!entry.hasPassword,
            action: action(() => void copySecret(entry, "password", "Password")),
          },
          {
            id: "copy-totp",
            text: "Copy TOTP",
            enabled: !!entry.hasTotp,
            action: action(() => void copySecret(entry, "totp", "TOTP")),
          },
          {
            id: "open-url",
            text: "Open URL",
            enabled: !!entry.url,
            action: action(() => openHttpUrl(entry.url ?? "")),
          },
          { item: "Separator" },
          {
            id: "edit",
            text: "Edit",
            enabled: !entry.trashed,
            action: action(() => vault.setEditingId(entry.id)),
          },
          {
            id: "favorite",
            text: entry.favorite ? "Remove from favorites" : "Add to favorites",
            enabled: !entry.trashed,
            action: action(() => void toggleFavorite(entry)),
          },
          {
            id: "delete",
            text: "Delete",
            action: action(() => entryDeletion.request(entry.id, entry.title)),
          },
        ],
      });
      if (generation !== contextMenuGeneration) {
        await menu.close();
        return;
      }
      // Keep the resource alive until the next open or unmount: on Linux,
      // popup() can resolve before the user has dismissed the native menu.
      contextMenu = menu;
      await menu.popup(position);
    } catch (error) {
      reportActionError(error);
    }
  }

  function openContextMenu(event: MouseEvent, entry: EntrySummary) {
    event.preventDefault();
    const position = new LogicalPosition(event.clientX, event.clientY);
    selection.requestNavigation(() => {
      selection.selectedId = entry.id;
      void showContextMenu(entry, position);
    });
  }

  async function onListKeydown(e: KeyboardEvent) {
    const fromSearch = document.activeElement?.matches(".entry-search-input") ?? false;
    if (e.isComposing) return;
    if (isTextEditable(document.activeElement)) {
      if (!fromSearch || (e.key !== "ArrowDown" && e.key !== "ArrowUp")) return;
      if (e.shiftKey) return;
    }

    if ((e.metaKey || e.ctrlKey) && !e.altKey && e.key.toLowerCase() === "c") {
      // Detail values are selectable; native Copy takes precedence over the
      // selected-entry shortcut when the user has highlighted text.
      if (!e.shiftKey && window.getSelection()?.toString()) return;
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
        if (fromSearch) return;
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
  <SearchField
    allowCreate={selection.filter.kind !== "trash"}
    {onCreate}
    searchLimited={entrySearch.truncated}
    onSearchStart={rememberSearchContext}
    onSearchClear={restoreSearchContext}
  />
  <div class="sort-bar">
    <span class="item-count">{filtered.length} {filtered.length === 1 ? "item" : "items"}</span>
    <div class="sort-control">
      <Icon name="arrows-sort" size={14} />
      <label for="entry-sort">Sort</label>
      <select id="entry-sort" bind:value={sortMode} aria-label="Sort entries">
        <option value="vault">Vault order</option>
        <option value="title">Title A–Z</option>
        <option value="modified">Recently modified</option>
      </select>
      <Icon name="chevron-down" size={14} />
    </div>
    {#if selection.filter.kind === "trash"}
      <Button
        type="button"
        variant="ghost-danger"
        size="xs"
        class="empty-trash-button"
        disabled={recycleBin.count === 0 || recycleBin.emptying}
        onclick={() => recycleBin.requestEmpty()}
      >
        <Icon name="trash-x" size={14} />
        {recycleBin.emptying ? "Emptying…" : "Empty…"}
      </Button>
    {/if}
  </div>
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
        {:else if selection.filter.kind === "trash"}
          <Icon name="trash" size={18} />
          <p>Recycle Bin is empty</p>
        {:else if selection.filter.kind === "health"}
          <Icon name="shield-check" size={18} />
          <p>{HEALTH_EMPTY_MESSAGES[selection.filter.issue]}</p>
        {:else if !hasActiveEntries}
          <Icon name="lock-open" size={18} />
          <p>No entries yet</p>
          <button type="button" class="empty-action" onclick={onCreate}>
            Create your first entry
          </button>
        {:else if selection.filter.kind === "type"}
          <Icon name="filter-off" size={18} />
          <p>
            No {ENTRY_TYPE_BY_TYPE[selection.filter.type].pluralLabel.toLowerCase()} in this view
          </p>
          <button type="button" class="empty-action" onclick={onCreate}>
            Create a {ENTRY_TYPE_BY_TYPE[selection.filter.type].label.toLowerCase()} entry
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
    background: var(--surface-2);
  }

  .sort-bar {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    min-height: 34px;
    padding: 0 var(--space-4);
    color: var(--text-muted);
    background: var(--surface-2);
    border-bottom: var(--hairline) solid var(--border);
    font-size: var(--text-xs);
  }

  .item-count {
    flex-shrink: 0;
  }

  .sort-control {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    min-width: 0;
    margin-left: auto;
  }

  .sort-bar select {
    min-width: 0;
    max-width: 112px;
    color: var(--text-secondary);
    font-size: inherit;
    cursor: default;
    appearance: none;
  }

  /* Tighter than `xs` — the sort bar is only 30px tall. */
  .sort-bar :global(.empty-trash-button) {
    flex-shrink: 0;
    padding: var(--space-1);
    border-radius: var(--radius-xs);
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
    gap: var(--space-2);
    padding: calc(var(--space-6) + var(--space-2)) var(--space-4);
    color: var(--text-muted);
    font-size: var(--text-base);
    text-align: center;
  }

  .empty-action {
    padding: var(--space-1) var(--space-3);
    color: var(--accent);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    font-weight: 500;
  }

  .empty-action:hover {
    background: var(--surface-hover);
  }

  .empty-action:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
