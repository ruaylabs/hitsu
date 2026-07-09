<script lang="ts">
  import Icon from "./Icon.svelte";

  let { onclose }: { onclose: () => void } = $props();

  const isMac = navigator.platform.toLowerCase().includes("mac");
  const mod = isMac ? "⌘" : "Ctrl";

  interface Shortcut {
    keys: string;
    description: string;
  }

  const shortcuts: Shortcut[] = [
    { keys: `${mod}N`, description: "New entry" },
    { keys: `${mod}⌫`, description: "Delete selected entry" },
    { keys: `${mod}F`, description: "Focus search" },
    { keys: `${mod}⇧F`, description: "Toggle favorites filter" },
    { keys: `${mod},`, description: "Toggle settings" },
    { keys: "Esc", description: "Close dialog / exit settings" },
    { keys: "↑ ↓", description: "Navigate items" },
    { keys: "Home / End", description: "Jump to first / last item" },
    { keys: `${mod}S`, description: "Save entry (edit mode)" },
    { keys: "Esc", description: "Cancel edit (edit mode)" },
  ];

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="dialog-overlay" onclick={onclose} role="dialog" aria-label="Keyboard shortcuts">
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="dialog-pane" onclick={(e) => e.stopPropagation()}>
    <header class="dialog-header">
      <h2 class="dialog-title">Keyboard shortcuts</h2>
      <button class="dialog-close" onclick={onclose} aria-label="Close" title="Close">
        <Icon name="x" size={16} />
      </button>
    </header>

    <div class="dialog-body">
      <div class="shortcuts-list">
        {#each shortcuts as shortcut}
          <div class="shortcut-row">
            <kbd class="shortcut-keys">{shortcut.keys}</kbd>
            <span class="shortcut-desc">{shortcut.description}</span>
          </div>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .dialog-pane {
    width: 380px;
    max-width: 90vw;
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-card);
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 0.5px solid var(--border);
  }

  .dialog-title {
    font-size: 15px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .dialog-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }

  .dialog-close:hover {
    background: var(--border);
  }

  .dialog-body {
    padding: 12px 0;
    max-height: 360px;
    overflow-y: auto;
  }

  .shortcuts-list {
    display: flex;
    flex-direction: column;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 20px;
    gap: 16px;
  }

  .shortcut-keys {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    min-width: 28px;
    justify-content: center;
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .shortcut-desc {
    font-size: 13px;
    color: var(--text-secondary);
    text-align: right;
  }
</style>
