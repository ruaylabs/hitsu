<script lang="ts">
  import type { Snippet } from "svelte";
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";

  let {
    title,
    confirmLabel,
    pendingLabel = "Saving…",
    busy = false,
    disabled = false,
    onconfirm,
    oncancel,
    children,
  }: {
    title: string;
    confirmLabel: string;
    pendingLabel?: string;
    busy?: boolean;
    disabled?: boolean;
    onconfirm: () => void;
    oncancel: () => void;
    children: Snippet;
  } = $props();
</script>

<Dialog {title} onclose={oncancel} {onconfirm} size="sm" closeLabel="Cancel">
  {@render children()}

  {#snippet footer()}
    <Button onclick={oncancel}>Cancel</Button>
    <Button variant="primary" onclick={onconfirm} disabled={busy || disabled}>
      {busy ? pendingLabel : confirmLabel}
    </Button>
  {/snippet}
</Dialog>
