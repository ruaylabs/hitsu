<script lang="ts">
  import * as entriesBridge from "$lib/bridge/entries";
  import type { EntrySummary } from "$lib/bridge/types";
  import EntryIcon from "$lib/components/list/EntryIcon.svelte";
  import { entryHaystack } from "$lib/utils/search";
  import Dialog from "./Dialog.svelte";
  import Icon from "./Icon.svelte";

  let {
    entries,
    onSelect,
    onClose,
  }: {
    entries: EntrySummary[];
    onSelect: (entry: EntrySummary) => void;
    onClose: () => void;
  } = $props();

  let search = $state("");
  let selectedIndex = $state(0);
  let searchMatchIds = $state<string[] | null>(null);
  let searchRequest = 0;

  $effect(() => {
    const query = search.trim();
    const request = ++searchRequest;
    searchMatchIds = null;
    if (!query) return;

    const timeout = setTimeout(() => {
      void entriesBridge
        .entriesSearch(query)
        .then((ids) => {
          if (request === searchRequest && search.trim() === query) searchMatchIds = ids;
        })
        .catch(() => {
          // Keep the summary-field fallback when backend search is unavailable.
        });
    }, 100);

    return () => {
      clearTimeout(timeout);
      if (searchRequest === request) searchRequest++;
    };
  });

  let filtered = $derived.by(() => {
    const query = search.trim().toLowerCase();
    if (!query) return entries;
    if (searchMatchIds === null) {
      return entries.filter((entry) => entryHaystack(entry).includes(query));
    }
    const matches = new Set(searchMatchIds);
    return entries.filter((entry) => matches.has(entry.id) || entryHaystack(entry).includes(query));
  });

  $effect(() => {
    filtered;
    selectedIndex = 0;
  });

  function keepSelectedVisible() {
    requestAnimationFrame(() => {
      const selected = document.querySelector<HTMLElement>(".entry-palette-item.selected");
      selected?.scrollIntoView?.({ block: "nearest" });
    });
  }

  function onKeydown(event: KeyboardEvent) {
    const ctrlNext = event.ctrlKey && !event.metaKey && event.key.toLowerCase() === "n";
    const ctrlPrevious = event.ctrlKey && !event.metaKey && event.key.toLowerCase() === "p";

    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      onClose();
    } else if (event.key === "ArrowDown" || ctrlNext) {
      event.preventDefault();
      event.stopPropagation();
      selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
      keepSelectedVisible();
    } else if (event.key === "ArrowUp" || ctrlPrevious) {
      event.preventDefault();
      event.stopPropagation();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      keepSelectedVisible();
    } else if (event.key === "Enter") {
      event.preventDefault();
      event.stopPropagation();
      const selected = filtered[selectedIndex];
      if (selected) onSelect(selected);
    }
  }
</script>

<Dialog
  title="Search entries"
  onclose={onClose}
  showHeader={false}
  placement="top"
  topOffset="96px"
  width="480px"
  maxWidth="calc(100vw - 32px)"
  maxHeight="min(440px, calc(100vh - 128px))"
  bodyPadding="none"
  bodyOverflow="hidden"
  bodyFill={true}
  onkeydown={onKeydown}
>
  <div class="entry-palette-content">
    <div class="entry-palette-search">
      <Icon name="search" size={16} />
      <!-- svelte-ignore a11y_autofocus -->
      <input
        class="entry-palette-input"
        type="text"
        placeholder="Search entries…"
        aria-label="Search entries"
        autofocus
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        bind:value={search}
      />
      <kbd>esc</kbd>
    </div>

    <div class="entry-palette-items" role="listbox" aria-label="Entries">
      {#each filtered as entry, index (entry.id)}
        <button
          class="entry-palette-item"
          class:selected={index === selectedIndex}
          onclick={() => onSelect(entry)}
          onmouseenter={() => (selectedIndex = index)}
          role="option"
          aria-selected={index === selectedIndex}
        >
          <EntryIcon iconHint={entry.iconHint} type={entry.type} title={entry.title} size={30} />
          <span class="entry-palette-text">
            <span class="entry-palette-title">{entry.title}</span>
            <span class="entry-palette-subtitle">{entry.subtitle}</span>
          </span>
          {#if entry.trashed}
            <span class="entry-palette-badge">Recycle Bin</span>
          {/if}
        </button>
      {:else}
        <div class="entry-palette-empty">
          <Icon name="search-off" size={18} />
          <span>No matching entries</span>
        </div>
      {/each}
    </div>
  </div>
</Dialog>

<style>
  .entry-palette-content {
    display: flex;
    min-height: 0;
    flex: 1;
    flex-direction: column;
    overflow: hidden;
  }

  .entry-palette-search {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 13px 14px;
    color: var(--text-muted);
    border-bottom: 0.5px solid var(--border);
  }

  .entry-palette-input {
    min-width: 0;
    flex: 1;
    color: var(--text-primary);
    background: transparent;
    border: none;
    font-size: 14px;
  }

  .entry-palette-input:focus {
    outline: none;
  }

  .entry-palette-input::placeholder {
    color: var(--text-muted);
  }

  kbd {
    padding: 1px 5px;
    color: var(--text-muted);
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 10px;
  }

  .entry-palette-items {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
  }

  .entry-palette-item {
    display: flex;
    width: 100%;
    align-items: center;
    gap: 10px;
    padding: 7px 10px;
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    text-align: left;
  }

  .entry-palette-item:hover,
  .entry-palette-item.selected {
    color: var(--text-accent);
    background: var(--bg-accent);
  }

  .entry-palette-text {
    display: flex;
    min-width: 0;
    flex: 1;
    flex-direction: column;
    gap: 2px;
  }

  .entry-palette-title,
  .entry-palette-subtitle {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .entry-palette-title {
    font-size: 13.5px;
    font-weight: 500;
  }

  .entry-palette-subtitle {
    color: var(--text-muted);
    font-size: 11.5px;
  }

  .entry-palette-badge {
    flex-shrink: 0;
    padding: 2px 6px;
    color: var(--text-muted);
    background: var(--surface-1);
    border-radius: 999px;
    font-size: 10px;
  }

  .entry-palette-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 28px 20px;
    color: var(--text-muted);
    font-size: 13px;
  }
</style>
