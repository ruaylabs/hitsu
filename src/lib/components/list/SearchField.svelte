<script lang="ts">
  import { onDestroy } from "svelte";
  import * as entriesBridge from "$lib/bridge/entries";
  import { toSummary } from "$lib/bridge/entries";
  import type { ItemType } from "$lib/bridge/types";
  import { selection } from "$lib/stores/selection.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import Icon from "../ui/Icon.svelte";

  let { allowCreate = true }: { allowCreate?: boolean } = $props();
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

  function requestCreateEntry(type: string) {
    selection.requestNavigation(() => {
      void createEntry(type);
    });
  }

  async function createEntry(type: string) {
    try {
      const entry = await entriesBridge.entryCreate(type, { title: `New ${type}` });
      vault.setEntries([...vault.entries, toSummary(entry)]);
      selection.filter = { kind: "type", type: type as ItemType };
      selection.selectedId = entry.id;
      // Mark as a brand-new entry so ItemDetail auto-discards it if the
      // user navigates away (or cancels) without saving. Must match the
      // command-palette / ⌘N path, which sets both ids.
      vault.setCreatingId(entry.id);
      vault.setEditingId(entry.id);
    } catch (e) {
      console.error("Failed to create entry", e);
    }
  }

  let showTypePicker = $state(false);

  $effect(() => {
    if (!showTypePicker) return;
    const close = () => {
      showTypePicker = false;
    };
    document.addEventListener("click", close);
    return () => document.removeEventListener("click", close);
  });
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
      <div class="search-actions">
        {#if showTypePicker}
          <div class="type-picker" role="menu">
            <button
              class="type-item"
              onclick={() => { showTypePicker = false; requestCreateEntry("login"); }}
              role="menuitem"
            >
              <Icon name="key" size={12} />
              Login
            </button>
            <button
              class="type-item"
              onclick={() => { showTypePicker = false; requestCreateEntry("password"); }}
              role="menuitem"
            >
              <Icon name="lock" size={12} />
              Password
            </button>
            <button
              class="type-item"
              onclick={() => { showTypePicker = false; requestCreateEntry("note"); }}
              role="menuitem"
            >
              <Icon name="notes" size={12} />
              Note
            </button>
            <button
              class="type-item"
              onclick={() => { showTypePicker = false; requestCreateEntry("identity"); }}
              role="menuitem"
            >
              <Icon name="user" size={12} />
              Identity
            </button>
            <button
              class="type-item"
              onclick={() => { showTypePicker = false; requestCreateEntry("card"); }}
              role="menuitem"
            >
              <Icon name="credit-card" size={12} />
              Card
            </button>
          </div>
        {/if}
        <button
          class="add-btn"
          onclick={(e) => { e.stopPropagation(); showTypePicker = !showTypePicker; }}
          aria-label="Add entry"
          title="Add entry"
        >
          <Icon name="plus" size={14} />
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .search-wrapper {
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

  .search-actions {
    position: relative;
    display: flex;
    align-items: center;
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

  .type-picker {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    display: flex;
    flex-direction: column;
    padding: 4px;
    z-index: 100;
  }

  .type-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    color: var(--text-primary);
    text-align: left;
    white-space: nowrap;
  }

  .type-item:hover {
    background: var(--bg-accent);
  }
</style>
