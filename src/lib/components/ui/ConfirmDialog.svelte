<script lang="ts">
  import Icon from "./Icon.svelte";

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

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      oncancel();
    } else if (e.key === "Enter") {
      onconfirm();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="dialog-overlay" onclick={oncancel} role="dialog" aria-label={title}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="dialog-pane" onclick={(e) => e.stopPropagation()}>
    <header class="dialog-header">
      <h2 class="dialog-title">{title}</h2>
      <button class="dialog-close" onclick={oncancel} aria-label="Cancel" title="Close">
        <Icon name="x" size={16} />
      </button>
    </header>

    <div class="dialog-body">
      <p class="dialog-message">{message}</p>
    </div>

    <footer class="dialog-footer">
      <button class="btn btn-cancel" onclick={oncancel}>{cancelLabel}</button>
      {#if secondaryLabel && onsecondary}
        <button class="btn btn-secondary" class:btn-danger={secondaryDanger} onclick={onsecondary}>
          {secondaryLabel}
        </button>
      {/if}
      <button class="btn" class:btn-danger={danger} class:btn-confirm={!danger} onclick={onconfirm}>
        {confirmLabel}
      </button>
    </footer>
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
    width: 360px;
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
    padding: 20px;
  }

  .dialog-message {
    font-size: 13.5px;
    color: var(--text-secondary);
    line-height: 1.5;
    margin: 0;
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px;
    border-top: 0.5px solid var(--border);
  }

  .btn {
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    transition: background 0.1s;
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
