<script lang="ts">
  import Icon from "../ui/Icon.svelte";
</script>

<div class="empty-detail">
  <Icon name="list-details" size={26} />
  <p>Select an item to view its details</p>
  <div class="shortcut-hints" aria-label="Keyboard shortcuts">
    <span><kbd>↑</kbd><kbd>↓</kbd> Navigate</span>
    <span><kbd>⌘K</kbd> Search</span>
    <span><kbd>⌘N</kbd> New</span>
  </div>
</div>

<style>
  .empty-detail {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    gap: 12px;
    min-height: 200px;
    padding: 40px;
    text-align: center;
  }

  .empty-detail p {
    margin: 0;
  }

  .shortcut-hints {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px 14px;
    margin-top: 4px;
    color: var(--text-muted);
    font-size: 11px;
  }

  .shortcut-hints span {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }

  kbd {
    min-width: 19px;
    padding: 2px 4px;
    color: var(--text-secondary);
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: 4px;
    box-shadow: 0 1px 0 var(--border);
    font: inherit;
    line-height: 1.2;
  }
</style>
