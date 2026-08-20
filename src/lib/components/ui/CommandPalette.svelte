<script lang="ts">
  import type { ItemType } from "$lib/bridge/types";
  import { ENTRY_TYPES } from "$lib/entryTypes";
  import { paletteKeydown } from "$lib/utils/paletteKeydown";
  import Dialog from "./Dialog.svelte";
  import Icon from "./Icon.svelte";

  let {
    onSelect,
    onClose,
  }: {
    onSelect: (type: ItemType) => void;
    onClose: () => void;
  } = $props();

  let search = $state("");
  let selectedIndex = $state(0);

  let normalizedSearch = $derived(search.trim().toLowerCase());
  let filtered = $derived(
    normalizedSearch
      ? ENTRY_TYPES.filter((item) =>
          `${item.label} ${item.description}`.toLowerCase().includes(normalizedSearch),
        )
      : ENTRY_TYPES,
  );

  // Reset selection when filter changes
  $effect(() => {
    filtered;
    selectedIndex = 0;
  });

  function onKeydown(event: KeyboardEvent) {
    paletteKeydown(event, {
      items: filtered,
      selectedIndex,
      onSelectedIndexChange: (index) => (selectedIndex = index),
      onSelect: (item) => onSelect(item.type),
      onClose,
    });
  }
</script>

<Dialog
  title="Create entry"
  onclose={onClose}
  showHeader={false}
  placement="top"
  topOffset="120px"
  width="400px"
  maxWidth="calc(100vw - 32px)"
  maxHeight="min(480px, calc(100vh - 152px))"
  bodyPadding="none"
  bodyOverflow="hidden"
  bodyFill={true}
  onkeydown={onKeydown}
>
  <div class="palette-content">
    <div class="palette-search">
      <Icon name="search" size={14} />
      <!-- svelte-ignore a11y_autofocus -->
      <input
        class="search-input"
        type="text"
        placeholder="Search entry types…"
        autofocus
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        bind:value={search}
      />
    </div>

    <div class="palette-items">
      {#each filtered as item, i (item.type)}
        <button
          class="palette-item"
          class:selected={i === selectedIndex}
          onclick={() => onSelect(item.type)}
          onmouseenter={() => (selectedIndex = i)}
          role="option"
          aria-selected={i === selectedIndex}
          aria-label={item.label}
          aria-describedby={`entry-type-${item.type}-description`}
        >
          <Icon name={item.icon} size={16} />
          <span class="palette-item-text">
            <span class="palette-item-label">{item.label}</span>
            <span id={`entry-type-${item.type}-description`} class="palette-item-description">
              {item.description}
            </span>
          </span>
        </button>
      {:else}
        <div class="palette-empty">No matching types</div>
      {/each}
    </div>
  </div>
</Dialog>

<style>
  .palette-content {
    display: flex;
    min-height: 0;
    flex: 1;
    flex-direction: column;
    overflow: hidden;
  }

  .palette-search {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
    border-bottom: 0.5px solid var(--border);
    color: var(--text-muted);
  }

  .search-input {
    flex: 1;
    font-size: 14px;
    color: var(--text-primary);
  }

  .search-input::placeholder {
    color: var(--text-muted);
  }

  .palette-items {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
  }

  .palette-item {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    width: 100%;
    padding: 9px 12px;
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    text-align: left;
  }

  .palette-item:hover,
  .palette-item.selected {
    background: var(--bg-accent);
    color: var(--text-accent);
  }

  .palette-item-text {
    display: flex;
    min-width: 0;
    flex: 1;
    flex-direction: column;
    gap: 2px;
  }

  .palette-item-label {
    font-size: 13.5px;
    font-weight: 500;
  }

  .palette-item-description {
    color: var(--text-muted);
    font-size: var(--text-sm);
    line-height: 1.35;
  }

  .palette-item:hover .palette-item-description,
  .palette-item.selected .palette-item-description {
    color: var(--text-accent);
  }

  .palette-empty {
    padding: 20px;
    text-align: center;
    font-size: 13px;
    color: var(--text-muted);
  }
</style>
