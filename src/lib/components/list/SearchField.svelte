<script lang="ts">
  import { selection } from "$lib/stores/selection.svelte";
  import Icon from "../ui/Icon.svelte";

  let search = $state(selection.search);

  function onInput(e: Event) {
    const el = e.currentTarget as HTMLInputElement;
    search = el.value;
    selection.search = search;
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      search = "";
      selection.search = "";
    }
  }
</script>

<div class="search-wrapper">
  <div class="search-pill">
    <Icon name="search" size={13} />
    <input
      type="text"
      class="search-input"
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
        onclick={() => { search = ""; selection.search = ""; }}
        aria-label="Clear search"
      >
        <Icon name="x" size={12} />
      </button>
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
  }

  .search-input {
    flex: 1;
    font-size: 13px;
    color: var(--text-primary);
    min-width: 0;
  }

  .search-input::placeholder {
    color: var(--text-muted);
  }

  .search-clear {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border-radius: 3px;
    color: var(--text-muted);
  }

  .search-clear:hover {
    background: var(--border);
  }
</style>
