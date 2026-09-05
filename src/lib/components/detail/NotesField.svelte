<script lang="ts">
  let { notes }: { notes: string } = $props();
</script>

<div class="notes-section">
  <span class="notes-label">Notes</span>
  <div class="notes-content">{notes}</div>
</div>

<style>
  .notes-section {
    margin-bottom: 16px;
  }

  .notes-label {
    display: block;
    font-size: var(--text-sm);
    color: var(--text-muted);
    margin-bottom: 6px;
  }

  .notes-content {
    padding: var(--space-3) var(--space-3);
    background: var(--surface-1);
    border-radius: var(--radius);
    font-size: var(--text-base);
    line-height: 1.55;
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
