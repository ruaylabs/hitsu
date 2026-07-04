<script lang="ts">
  import { timeAgo } from "$lib/utils/format";
  import Icon from "../ui/Icon.svelte";

  let {
    modifiedAt,
    historyCount,
    onclick,
  }: {
    modifiedAt: string;
    historyCount: number;
    onclick?: () => void;
  } = $props();
</script>

<div class="detail-footer">
  <span>Modified {timeAgo(modifiedAt)}</span>
  <button class="history-btn" aria-label="View history" title="View history" {onclick}>
    <Icon name="history" size={12} />
    <span>{historyCount} revisions</span>
  </button>
</div>

<style>
  .detail-footer {
    padding-top: 12px;
    border-top: 0.5px solid var(--border);
    display: flex;
    justify-content: space-between;
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .history-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .history-btn:hover {
    color: var(--text-secondary);
  }
</style>
