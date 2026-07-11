<script lang="ts">
  import Dialog from "./Dialog.svelte";

  let { onclose }: { onclose: () => void } = $props();

  const isMac = navigator.platform.toLowerCase().includes("mac");
  const mod = isMac ? "⌘" : "Ctrl";

  interface Shortcut {
    keys: string;
    description: string;
  }

  const shortcuts: Shortcut[] = [
    { keys: "?", description: "Open keyboard shortcuts" },
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
</script>

<Dialog
  title="Keyboard shortcuts"
  {onclose}
  size="md"
  bodyPadding="none"
  bodyOverflow="auto"
  bodyMaxHeight="360px"
>
  <div class="shortcuts-list">
    {#each shortcuts as shortcut}
      <div class="shortcut-row">
        <kbd class="shortcut-keys">{shortcut.keys}</kbd>
        <span class="shortcut-desc">{shortcut.description}</span>
      </div>
    {/each}
  </div>
</Dialog>

<style>
  .shortcuts-list {
    display: flex;
    flex-direction: column;
    padding: 12px 0;
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
