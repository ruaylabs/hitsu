<script lang="ts">
  import { timeAgo } from "$lib/utils/format";
  import Icon from "../ui/Icon.svelte";

  let {
    modifiedAt,
    createdAt,
    historyCount,
    onclick,
  }: {
    modifiedAt: string;
    createdAt: string;
    historyCount: number;
    onclick?: () => void;
  } = $props();

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  }
</script>

<div class="detail-footer">
  <span>Modified {timeAgo(modifiedAt)}</span>
  <span>Created on {formatDate(createdAt)}</span>
  <button class="history-btn" aria-label="View history" title="View history" {onclick}>
    <Icon name="history" size={12} />
    <span>History · {historyCount} revisions</span>
  </button>
</div>

<style>
  .detail-footer {
    padding-top: var(--space-3);
    border-top: 0.5px solid var(--border);
    display: grid;
    grid-template-columns: 1fr auto;
    gap: var(--space-1) var(--space-3);
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .history-btn {
    grid-column: 2;
    grid-row: 1 / span 2;
    align-self: center;
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .history-btn:hover {
    color: var(--text-secondary);
  }
</style>
