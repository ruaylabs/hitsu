<script lang="ts">
  import { onDestroy } from "svelte";
  import type { SidebarFilter } from "$lib/bridge/types";
  import { ENTRY_TYPE_BY_TYPE } from "$lib/entryTypes";
  import { selection } from "$lib/stores/selection.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { keyboardShortcut } from "$lib/utils/keyboardShortcut";
  import Icon from "../ui/Icon.svelte";

  let {
    allowCreate = true,
    onCreate = () => {},
    onSearchStart = () => {},
    onSearchClear = () => {},
    searchLimited = false,
  }: {
    allowCreate?: boolean;
    onCreate?: () => void;
    onSearchStart?: () => void;
    onSearchClear?: () => void;
    searchLimited?: boolean;
  } = $props();
  let search = $state(selection.search);
  const searchShortcut = keyboardShortcut("F");

  function getScopeName(filter: SidebarFilter): string {
    if (filter.kind === "all") return "All items";
    if (filter.kind === "favorites") return "Favorites";
    if (filter.kind === "recent") return "Recent";
    if (filter.kind === "trash") return "Recycle Bin";
    if (filter.kind === "health") {
      return {
        weak: "Weak passwords",
        reused: "Reused passwords",
      }[filter.issue];
    }
    if (filter.kind === "type") return ENTRY_TYPE_BY_TYPE[filter.type].pluralLabel;
    if (filter.kind === "tag") return `#${filter.tag}`;
    return vault.folders.find((folder) => folder.id === filter.folderId)?.name ?? "Folder";
  }

  let scoped = $derived(selection.filter.kind !== "all");
  let scopeName = $derived(getScopeName(selection.filter));
  let placeholder = $derived(scoped ? `Search in ${scopeName}…` : "Search all items…");

  // The input echoes keystrokes immediately, but filtering the list is
  // deferred so fast typing doesn't re-filter on every keystroke.
  const SEARCH_DEBOUNCE_MS = 100;
  let debounce: ReturnType<typeof setTimeout> | undefined;

  function setSearch(value: string) {
    if (!search && value) onSearchStart();
    search = value;
    clearTimeout(debounce);
    debounce = setTimeout(() => {
      selection.search = value;
    }, SEARCH_DEBOUNCE_MS);
  }

  function clearSearch() {
    search = "";
    clearTimeout(debounce);
    selection.search = "";
    onSearchClear();
  }

  function clearScope() {
    selection.requestNavigation(() => {
      selection.filter = { kind: "all" };
    });
  }

  onDestroy(() => clearTimeout(debounce));

  function onInput(e: Event) {
    setSearch((e.currentTarget as HTMLInputElement).value);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      clearSearch();
    }
  }
</script>

<div class="search-wrapper">
  <div class="search-pill">
    <Icon name="search" size={13} />
    <input
      type="text"
      class="entry-search-input"
      {placeholder}
      aria-label={placeholder}
      autocomplete="off"
      autocorrect="off"
      autocapitalize="off"
      spellcheck="false"
      value={search}
      oninput={onInput}
      onkeydown={onKeydown}
    />
    {#if !search}
      <kbd class="search-shortcut" aria-hidden="true">{searchShortcut}</kbd>
    {:else}
      <button
        class="search-clear"
        onclick={clearSearch}
        aria-label="Clear search"
        title="Clear search"
      >
        <Icon name="x" size={12} />
      </button>
    {/if}
    {#if allowCreate}
      <div class="search-divider"></div>
      <button class="add-btn" onclick={onCreate} aria-label="Add entry" title="Add entry">
        <Icon name="plus" size={14} />
      </button>
    {/if}
  </div>
  {#if search && searchLimited}
    <div class="search-warning" role="status">
      More than 500 full-field matches were found; keep typing to narrow the search.
    </div>
  {/if}
  {#if scoped}
    <div class="scope-row" aria-label="Search scope">
      <span class="scope-chip">
        <Icon name="filter" size={12} />
        <span>{scopeName}</span>
        <button
          type="button"
          onclick={clearScope}
          aria-label={`Remove ${scopeName} search scope`}
          title="Search all items"
        >
          <Icon name="x" size={11} />
        </button>
      </span>
      {#if search}
        <button type="button" class="search-all" onclick={clearScope}>Search all items</button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .search-wrapper {
    position: relative;
    z-index: 1;
    padding: var(--space-3) var(--space-3);
    border-bottom: 0.5px solid var(--border);
  }

  .search-pill {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    background: var(--surface-1);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    isolation: isolate;
  }

  .entry-search-input {
    flex: 1;
    font-size: var(--text-base);
    color: var(--text-primary);
    min-width: 0;
  }

  .entry-search-input::placeholder {
    color: var(--text-muted);
  }

  .search-shortcut {
    flex-shrink: 0;
    padding: var(--space-half) var(--space-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-xs);
    color: var(--text-muted);
    font-family: var(--font-sans);
    font-size: var(--text-xs);
    line-height: var(--leading-normal);
    opacity: 0.75;
  }

  .search-clear {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    margin-block: -6px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .search-clear:hover {
    background: var(--surface-hover);
  }

  .search-divider {
    width: 1px;
    height: 18px;
    background: var(--border);
    flex-shrink: 0;
  }

  .add-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    margin-block: -6px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .add-btn:hover {
    background: var(--surface-hover);
    color: var(--text-secondary);
  }

  .search-warning {
    margin-top: 7px;
    color: var(--warning-text);
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
  }

  .scope-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    margin-top: 8px;
  }

  .scope-chip {
    display: inline-flex;
    min-width: 0;
    align-items: center;
    gap: var(--space-1);
    padding-left: var(--space-2);
    color: var(--text-secondary);
    background: var(--bg-accent);
    border-radius: 999px;
    font-size: var(--text-sm);
  }

  .scope-chip > span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .scope-chip button {
    display: inline-flex;
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    border-radius: 50%;
  }

  .scope-chip button:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .search-all {
    flex-shrink: 0;
    padding: var(--space-1) var(--space-2);
    color: var(--text-accent);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    font-weight: 500;
  }

  .search-all:hover {
    background: var(--surface-hover);
  }
</style>
