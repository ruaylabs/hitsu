<script lang="ts">
  import type { ItemType } from "$lib/bridge/types";
  import { ENTRY_TYPES } from "$lib/entryTypes";
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

  let filtered = $derived(
    search
      ? ENTRY_TYPES.filter((item) => item.label.toLowerCase().includes(search.toLowerCase()))
      : ENTRY_TYPES,
  );

  // Reset selection when filter changes
  $effect(() => {
    filtered;
    selectedIndex = 0;
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (filtered[selectedIndex]) {
        onSelect(filtered[selectedIndex].type);
      }
    }
  }
</script>

<div
  class="palette-overlay"
  onclick={onClose}
  role="dialog"
  aria-label="Command palette"
  tabindex="-1"
>
  <div
    class="palette-pane"
    onclick={(e) => e.stopPropagation()}
    onkeydown={onKeydown}
    role="listbox"
    aria-label="Entry types"
  >
    <div class="palette-search">
      <Icon name="search" size={14} />
      <!-- svelte-ignore a11y_autofocus -->
      <input
        class="search-input"
        type="text"
        placeholder="Type to filter…"
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
        >
          <Icon name={item.icon} size={15} />
          <span>{item.label}</span>
        </button>
      {:else}
        <div class="palette-empty">No matching types</div>
      {/each}
    </div>
  </div>
</div>

<style>
  .palette-overlay {
    position: fixed;
    inset: 0;
    background: var(--backdrop);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 120px;
    z-index: var(--z-dialog);
  }

  .palette-pane {
    width: 320px;
    max-height: 360px;
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-card);
    box-shadow: var(--shadow-dialog);
    display: flex;
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
    background: transparent;
    border: none;
    font-size: 14px;
    color: var(--text-primary);
  }

  .search-input:focus {
    outline: none;
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
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 13.5px;
    color: var(--text-primary);
    text-align: left;
  }

  .palette-item:hover,
  .palette-item.selected {
    background: var(--bg-accent);
    color: var(--text-accent);
  }

  .palette-empty {
    padding: 20px;
    text-align: center;
    font-size: 13px;
    color: var(--text-muted);
  }
</style>
