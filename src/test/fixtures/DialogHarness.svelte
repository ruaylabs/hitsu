<script lang="ts">
  import Dialog from "$lib/components/ui/Dialog.svelte";

  let {
    onclose,
    onconfirm,
  }: {
    onclose: () => void;
    onconfirm?: () => void;
  } = $props();
</script>

<Dialog title="Test dialog" {onclose} {onconfirm} size="sm">
  <button type="button">First action</button>
  <button type="button">Last action</button>
</Dialog>
