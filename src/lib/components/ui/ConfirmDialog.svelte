<script lang="ts">
  import Button from "./Button.svelte";
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
    <Button onclick={oncancel}>{cancelLabel}</Button>
    {#if secondaryLabel && onsecondary}
      <Button variant={secondaryDanger ? "danger" : "secondary"} onclick={onsecondary}>
        {secondaryLabel}
      </Button>
    {/if}
    <Button variant={danger ? "danger" : "primary"} onclick={onconfirm}>{confirmLabel}</Button>
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
</style>
