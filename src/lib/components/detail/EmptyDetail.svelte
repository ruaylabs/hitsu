<script lang="ts">
  import { keyboardShortcut } from "$lib/utils/keyboardShortcut";
  import Icon from "../ui/Icon.svelte";
</script>

<div class="empty-detail">
  <Icon name="list-details" size={26} />
  <p>Select an item to view its details</p>
  <div class="shortcut-hints" aria-label="Keyboard shortcuts">
    <span><kbd>↑</kbd><kbd>↓</kbd> Navigate</span>
    <span><kbd>{keyboardShortcut("K")}</kbd> Search</span>
    <span><kbd>{keyboardShortcut("N")}</kbd> New</span>
  </div>
</div>

<style>
  .empty-detail {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    gap: var(--space-3);
    min-height: 200px;
    padding: calc(var(--space-5) + var(--space-5));
    text-align: center;
  }

  .shortcut-hints {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: var(--space-2) var(--space-4);
    margin-top: 4px;
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .shortcut-hints span {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  kbd {
    min-width: 19px;
    padding: var(--space-half) var(--space-1);
    color: var(--text-secondary);
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-xs);
    box-shadow: 0 1px 0 var(--border);
    font: inherit;
    line-height: var(--leading-tight);
  }
</style>
