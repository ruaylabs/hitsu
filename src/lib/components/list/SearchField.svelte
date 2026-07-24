<script lang="ts">
  import { onDestroy } from "svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import Icon from "../ui/Icon.svelte";

  let {
    allowCreate = true,
    onCreate = () => {},
  }: {
    allowCreate?: boolean;
    onCreate?: () => void;
  } = $props();
  let search = $state(selection.search);

  // The input echoes keystrokes immediately, but filtering the list is
  // deferred so fast typing doesn't re-filter on every keystroke.
  const SEARCH_DEBOUNCE_MS = 100;
  let debounce: ReturnType<typeof setTimeout> | undefined;

  function setSearch(value: string) {
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
      placeholder="Search..."
      autocomplete="off"
      autocorrect="off"
      autocapitalize="off"
      spellcheck="false"
      value={search}
      oninput={onInput}
      onkeydown={onKeydown}
    />
    {#if search}
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
</div>

<style>
  .search-wrapper {
    position: relative;
    z-index: 1;
    padding: 10px 12px;
    border-bottom: 0.5px solid var(--border);
  }

  .search-pill {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--surface-1);
    padding: 7px 10px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    isolation: isolate;
  }

  .entry-search-input {
    flex: 1;
    font-size: 13px;
    color: var(--text-primary);
    min-width: 0;
  }

  .entry-search-input::placeholder {
    color: var(--text-muted);
  }

  .search-clear {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 3px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .search-clear:hover {
    background: var(--border);
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
    width: 20px;
    height: 20px;
    border-radius: 3px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .add-btn:hover {
    background: var(--border);
    color: var(--text-secondary);
  }
</style>
