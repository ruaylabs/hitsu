<script lang="ts">
  import { vault } from "$lib/stores/vault.svelte";

  let {
    initialTags,
    onupdate,
  }: {
    initialTags: string[];
    onupdate?: (tags: string[]) => void;
  } = $props();

  let inputValue = $state("");
  let currentTags = $state<string[]>([]);
  let showSuggestions = $state(false);
  let selectedIndex = $state(0);
  let initialized = false;

  $effect(() => {
    // Set initial value once (effect runs once on mount)
    if (!initialized) {
      inputValue = initialTags.join(", ");
      currentTags = [...initialTags];
      initialized = true;
    }
  });

  let allTags = $derived([...new Set(vault.entries.flatMap((e) => e.tags))].sort());

  let currentToken = $derived.by(() => {
    const cursorPos = inputValue.length;
    const before = inputValue.slice(0, cursorPos);
    const lastComma = before.lastIndexOf(",");
    return before.slice(lastComma + 1).trimStart();
  });

  let suggestions = $derived.by(() => {
    if (!currentToken) return [] as string[];
    const t = currentToken.toLowerCase();
    const exclude = new Set(currentTags);
    return allTags.filter((tag) => tag.toLowerCase().includes(t) && !exclude.has(tag)).slice(0, 6);
  });

  let visible = $derived(showSuggestions && suggestions.length > 0);

  function commitTags() {
    const parsed = inputValue
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);
    currentTags = parsed;
    onupdate?.(parsed);
  }

  function selectSuggestion(tag: string) {
    const parts = inputValue.split(",");
    parts[parts.length - 1] = parts[parts.length - 1]!.replace(currentToken, tag);
    inputValue = parts.join(", ");
    if (!inputValue.endsWith(", ")) {
      const lastPart = parts[parts.length - 1]!;
      if (lastPart.endsWith(tag) && !lastPart.includes(",", lastPart.indexOf(tag))) {
        inputValue += ", ";
      }
    }
    showSuggestions = false;
    commitTags();
  }

  function onKeydown(e: KeyboardEvent) {
    if (!visible) {
      if (e.key === "Escape") {
        showSuggestions = false;
      }
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, suggestions.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (suggestions[selectedIndex]) {
        selectSuggestion(suggestions[selectedIndex]!);
      }
    } else if (e.key === "Escape") {
      showSuggestions = false;
    }
  }

  function oninput(e: Event) {
    const el = e.currentTarget as HTMLInputElement;
    inputValue = el.value;
    showSuggestions = true;
    selectedIndex = 0;
    commitTags();
  }
</script>

<div class="tag-input-wrapper">
  <input
    class="tag-input"
    type="text"
    placeholder="comma, separated"
    value={inputValue}
    {oninput}
    onkeydown={onKeydown}
    onfocus={() => (showSuggestions = true)}
    onblur={() => setTimeout(() => (showSuggestions = false), 150)}
  />
  {#if visible}
    <ul class="suggestions" role="listbox">
      {#each suggestions as tag, i}
        <li
          class="suggestion-item"
          class:selected={i === selectedIndex}
          role="option"
          aria-selected={i === selectedIndex}
          onclick={() => selectSuggestion(tag)}
          onkeydown={(e) => { if (e.key === "Enter") selectSuggestion(tag); }}
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

  .tag-input {
    width: 100%;
    padding: 6px 8px;
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 13.5px;
    color: var(--text-primary);
  }

  .tag-input:focus {
    border-color: var(--accent);
    outline: none;
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
