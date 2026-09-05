<script lang="ts">
  import type { EntrySummary } from "$lib/bridge/types";
  import EntryIcon from "$lib/components/list/EntryIcon.svelte";
  import { createEntrySearch } from "$lib/utils/entrySearch.svelte";
  import { paletteKeydown } from "$lib/utils/paletteKeydown";
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

  const entrySearch = createEntrySearch(() => search, { debounceMs: 100 });

  // Only ~10 rows are ever visible; render a bounded slice and let the query
  // narrow it, as the backend does with SEARCH_RESULT_LIMIT.
  const MAX_RESULTS = 50;

  let matches = $derived.by(() => {
    if (!search.trim()) return entries;
    return entries.filter((entry) => entrySearch.matches(entry));
  });

  let filtered = $derived(matches.slice(0, MAX_RESULTS));

  let truncatedLabel = $derived(
    matches.length > filtered.length
      ? `Showing ${filtered.length} of ${matches.length} — keep typing to narrow`
      : "",
  );

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
    paletteKeydown(event, {
      items: filtered,
      selectedIndex,
      onSelectedIndexChange: (index) => (selectedIndex = index),
      onSelect,
      onClose,
      onNavigate: keepSelectedVisible,
    });
  }
</script>

<Dialog
  title="Search entries"
  onclose={onClose}
  showHeader={false}
  placement="top"
  topOffset="96px"
  width="480px"
  maxHeight="min(440px, calc(100vh - 128px))"
  bodyPadding="none"
  bodyOverflow="hidden"
  bodyFill={true}
  onkeydown={onKeydown}
>
  <div class="entry-palette-content">
    <div class="entry-palette-search">
      <Icon name="search" size={14} />
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
          <EntryIcon
            iconHint={entry.iconHint}
            type={entry.type}
            title={entry.title}
            size={30}
            hasCustomIcon={entry.hasCustomIcon}
            entryId={entry.id}
          />
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

    {#if truncatedLabel}
      <div class="entry-palette-footer">{truncatedLabel}</div>
    {/if}
    {#if search && entrySearch.truncated}
      <div class="entry-palette-warning" role="status">
        More than 500 full-field matches were found; keep typing to narrow the search.
      </div>
    {/if}
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
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    color: var(--text-muted);
    border-bottom: 0.5px solid var(--border);
  }

  .entry-palette-input {
    min-width: 0;
    flex: 1;
    color: var(--text-primary);
    font-size: var(--text-base);
  }

  .entry-palette-input::placeholder {
    color: var(--text-muted);
  }

  kbd {
    padding: var(--space-half) var(--space-1);
    color: var(--text-muted);
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-xs);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .entry-palette-items {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-1);
  }

  .entry-palette-item {
    display: flex;
    width: 100%;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    color: var(--text-primary);
    border-radius: var(--radius-sm);
    text-align: left;
  }

  .entry-palette-item:hover {
    background: var(--surface-hover);
  }

  .entry-palette-item.selected {
    color: var(--text-accent);
    background: var(--bg-accent);
  }

  .entry-palette-text {
    display: flex;
    min-width: 0;
    flex: 1;
    flex-direction: column;
    gap: var(--space-half);
  }

  .entry-palette-title,
  .entry-palette-subtitle {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .entry-palette-title {
    font-size: var(--text-base);
    font-weight: 500;
  }

  .entry-palette-subtitle {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .entry-palette-badge {
    flex-shrink: 0;
    padding: var(--space-half) var(--space-2);
    color: var(--text-muted);
    background: var(--surface-1);
    border-radius: 999px;
    font-size: var(--text-xs);
  }

  .entry-palette-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: calc(var(--space-6) + var(--space-1)) var(--space-5);
    color: var(--text-muted);
    font-size: var(--text-base);
  }

  .entry-palette-footer {
    flex-shrink: 0;
    padding: var(--space-2) var(--space-4);
    color: var(--text-muted);
    border-top: 0.5px solid var(--border);
    font-size: var(--text-sm);
  }

  .entry-palette-warning {
    flex-shrink: 0;
    padding: var(--space-2) var(--space-4);
    color: var(--warning-text);
    border-top: 0.5px solid var(--border);
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
  }
</style>
