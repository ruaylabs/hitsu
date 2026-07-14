<script lang="ts">
  import Dialog from "$lib/components/ui/Dialog.svelte";

  let {
    onclose,
    onconfirm,
    showHeader = true,
    placement = "center",
  }: {
    onclose: () => void;
    onconfirm?: () => void;
    showHeader?: boolean;
    placement?: "center" | "top";
  } = $props();
</script>

<Dialog title="Test dialog" {onclose} {onconfirm} {showHeader} {placement} size="sm">
  <button type="button">First action</button>
  <button type="button">Last action</button>
</Dialog>
