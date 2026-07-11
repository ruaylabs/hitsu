<script lang="ts">
  import Dialog from "./Dialog.svelte";

  let {
    title = "Confirm",
    message = "Are you sure?",
    confirmLabel = "Confirm",
    cancelLabel = "Cancel",
    secondaryLabel,
    danger = false,
    secondaryDanger = false,
    onconfirm,
    onsecondary,
    oncancel,
  }: {
    title?: string;
    message?: string;
    confirmLabel?: string;
    cancelLabel?: string;
    secondaryLabel?: string;
    danger?: boolean;
    secondaryDanger?: boolean;
    onconfirm: () => void;
    onsecondary?: () => void;
    oncancel: () => void;
  } = $props();
</script>

<Dialog {title} onclose={oncancel} {onconfirm} width="360px" closeLabel="Cancel">
  {#snippet children()}
    <div class="dialog-body">
      <p class="dialog-message">{message}</p>
    </div>
  {/snippet}

  {#snippet footer()}
    <button class="btn btn-cancel" onclick={oncancel}>{cancelLabel}</button>
    {#if secondaryLabel && onsecondary}
      <button class="btn btn-secondary" class:btn-danger={secondaryDanger} onclick={onsecondary}>
        {secondaryLabel}
      </button>
    {/if}
    <button class="btn" class:btn-danger={danger} class:btn-confirm={!danger} onclick={onconfirm}>
      {confirmLabel}
    </button>
  {/snippet}
</Dialog>

<style>
  .dialog-body {
    padding: 20px;
  }

  .dialog-message {
    font-size: 13.5px;
    color: var(--text-secondary);
    line-height: 1.5;
    margin: 0;
  }

  .btn {
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    transition: background var(--transition-fast);
    cursor: pointer;
  }

  .btn-cancel,
  .btn-secondary {
    color: var(--text-secondary);
    background: transparent;
  }

  .btn-cancel:hover,
  .btn-secondary:hover {
    background: var(--border);
  }

  .btn-confirm {
    color: #fff;
    background: var(--accent);
  }

  .btn-confirm:hover {
    opacity: 0.9;
  }

  .btn-danger {
    color: #fff;
    background: var(--danger);
  }

  .btn-danger:hover {
    opacity: 0.9;
  }
</style>
