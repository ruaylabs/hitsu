<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    label,
    children,
    alignStart = false,
    standalone = false,
    status = "default",
  }: {
    label: string;
    children: Snippet;
    alignStart?: boolean;
    standalone?: boolean;
    status?: "default" | "success" | "danger";
  } = $props();
</script>

<div
  class="detail-field-row"
  class:align-start={alignStart}
  class:standalone
  class:success={status === "success"}
  class:danger={status === "danger"}
>
  <span class="detail-field-label" title={label}>{label}</span>
  {@render children()}
</div>

<style>
  .detail-field-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-height: var(--row-md);
    padding: var(--space-3) var(--space-3);
    background: var(--surface-2);
  }

  .detail-field-row.align-start {
    align-items: flex-start;
  }

  .detail-field-row.standalone {
    margin-bottom: 16px;
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
    transition: border-color var(--transition-base);
  }

  .detail-field-row.success {
    border-color: var(--success);
  }

  .detail-field-row.danger {
    border-color: var(--danger);
  }

  /* Fixed so values line up down the pane. Custom field names are
     user-supplied and unbounded, so clip rather than wrap — wrapping would
     push the row past its 38px rhythm. The full name stays in the title. */
  .detail-field-label {
    width: 70px;
    flex-shrink: 0;
    overflow: hidden;
    color: var(--text-muted);
    font-size: var(--text-sm);
    white-space: nowrap;
    text-overflow: ellipsis;
  }
</style>
