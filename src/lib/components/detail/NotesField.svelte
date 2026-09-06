<script lang="ts">
  let { notes }: { notes: string } = $props();
</script>

<div class="notes-section">
  <span class="notes-label">Notes</span>
  <div class="notes-content">{notes}</div>
</div>

<style>
  .notes-section {
    margin-bottom: var(--space-5);
    padding-top: var(--space-4);
    border-top: 1px solid var(--divider);
  }

  .notes-label {
    display: block;
    font-size: var(--text-sm);
    color: var(--text-muted);
    margin-bottom: 6px;
  }

  .notes-content {
    user-select: text;
    font-size: var(--text-base);
    line-height: 1.7;
    color: var(--text-secondary);
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
