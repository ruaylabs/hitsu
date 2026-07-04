<script lang="ts">
  import { vault } from "$lib/stores/vault.svelte";

  let {
    initialTags,
    onupdate,
  }: {
    initialTags: string[];
    onupdate?: (tags: string[]) => void;
  } = $props();

  let currentTags = $state<string[]>([]);
  let inputValue = $state("");
  let showSuggestions = $state(false);
  let selectedIndex = $state(0);
  let inputEl = $state<HTMLInputElement | null>(null);
  let initialized = false;

  $effect(() => {
    // Capture the initial tags once on mount. `initialTags` is the parent's
    // current value at creation time, not a reactive source we want to track.
    if (!initialized) {
      currentTags = [...initialTags];
      initialized = true;
    }
  });

  let allTags = $derived([...new Set(vault.entries.flatMap((e) => e.tags))].sort());

  let currentToken = $derived(inputValue.trim());

  let suggestions = $derived.by(() => {
    if (!currentToken) return [] as string[];
    const t = currentToken.toLowerCase();
    const exclude = new Set(currentTags);
    return allTags.filter((tag) => tag.toLowerCase().includes(t) && !exclude.has(tag)).slice(0, 6);
  });

  let visible = $derived(showSuggestions && suggestions.length > 0);

  function emit(tags: string[]) {
    currentTags = tags;
    onupdate?.(tags);
  }

  function commitInput() {
    const value = inputValue.trim();
    if (!value) return;
    // Split on comma so pasting "a, b, c" adds three chips at once.
    const parts = value
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);
    const next = [...currentTags];
    for (const p of parts) {
      if (!next.includes(p)) next.push(p);
    }
    emit(next);
    inputValue = "";
    showSuggestions = false;
  }

  function addTag(tag: string) {
    if (!tag || currentTags.includes(tag)) return;
    emit([...currentTags, tag]);
    inputValue = "";
    showSuggestions = false;
    inputEl?.focus();
  }

  function removeTag(tag: string) {
    emit(currentTags.filter((t) => t !== tag));
  }

  function selectSuggestion(tag: string) {
    addTag(tag);
  }

  function onKeydown(e: KeyboardEvent) {
    if (visible) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        selectedIndex = Math.min(selectedIndex + 1, suggestions.length - 1);
        return;
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        selectedIndex = Math.max(selectedIndex - 1, 0);
        return;
      }
      if (e.key === "Enter") {
        e.preventDefault();
        if (suggestions[selectedIndex]) {
          selectSuggestion(suggestions[selectedIndex]!);
        } else {
          commitInput();
        }
        return;
      }
      if (e.key === "Escape") {
        showSuggestions = false;
        return;
      }
      if (e.key === "Tab" && suggestions[selectedIndex]) {
        // Accept the top suggestion on Tab to keep typing flow.
        e.preventDefault();
        selectSuggestion(suggestions[selectedIndex]!);
        return;
      }
    }

    // Backspace on an empty input removes the last chip.
    if (e.key === "Backspace" && inputValue === "" && currentTags.length > 0) {
      e.preventDefault();
      emit(currentTags.slice(0, -1));
    }
  }

  function onInput(e: Event) {
    const el = e.currentTarget as HTMLInputElement;
    inputValue = el.value;
    showSuggestions = true;
    selectedIndex = 0;
    // Commit immediately when the token ends with a comma.
    if (inputValue.includes(",")) {
      commitInput();
    }
  }
</script>

<div class="tag-input-wrapper">
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
  <div class="chip-area" onclick={() => inputEl?.focus()} role="group" aria-label="Tags">
    {#each currentTags as tag (tag)}
      <span class="chip">
        {tag}
        <button
          type="button"
          class="chip-remove"
          aria-label="Remove {tag}"
          title="Remove"
          onclick={() => removeTag(tag)}
        >
          <i class="ti ti-x" style="font-size: 11px"></i>
        </button>
      </span>
    {/each}
    <input
      bind:this={inputEl}
      class="tag-input"
      type="text"
      placeholder={currentTags.length === 0 ? "Add tags…" : ""}
      autocomplete="off"
      autocorrect="off"
      autocapitalize="off"
      spellcheck="false"
      value={inputValue}
      oninput={onInput}
      onkeydown={onKeydown}
      onfocus={() => (showSuggestions = true)}
      onblur={() => setTimeout(() => (showSuggestions = false), 150)}
    />
  </div>
  {#if visible}
    <ul class="suggestions" role="listbox">
      {#each suggestions as tag, i (tag)}
        <li
          class="suggestion-item"
          class:selected={i === selectedIndex}
          role="option"
          aria-selected={i === selectedIndex}
          onmousedown={(e) => { e.preventDefault(); selectSuggestion(tag); }}
          onmouseenter={() => (selectedIndex = i)}
        >
          {tag}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .tag-input-wrapper {
    position: relative;
  }

  .chip-area {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
    min-height: 30px;
    padding: 4px 6px;
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: text;
  }

  /* Make the click area keyboard-accessible: it's a label-like container
     that focuses the input. */
  .chip-area:focus-within {
    border-color: var(--accent);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 2px 4px 2px 8px;
    background: var(--surface-2);
    border: 0.5px solid var(--border-strong);
    border-radius: 10px;
    font-size: 12px;
    color: var(--text-primary);
  }

  .chip-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    color: var(--text-muted);
  }

  .chip-remove:hover {
    background: var(--border);
    color: var(--danger);
  }

  .tag-input {
    flex: 1;
    min-width: 60px;
    background: transparent;
    border: none;
    font-size: 13.5px;
    color: var(--text-primary);
    padding: 2px 4px;
  }

  .tag-input:focus {
    outline: none;
  }

  .tag-input::placeholder {
    color: var(--text-muted);
  }

  .suggestions {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 2px;
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    list-style: none;
    padding: 4px;
    z-index: 10;
  }

  .suggestion-item {
    padding: 5px 10px;
    border-radius: 4px;
    font-size: 13px;
    color: var(--text-primary);
    cursor: pointer;
  }

  .suggestion-item:hover,
  .suggestion-item.selected {
    background: var(--bg-accent);
    color: var(--text-accent);
  }
</style>
