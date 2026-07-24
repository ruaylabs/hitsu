<script lang="ts">
  import { vault } from "$lib/stores/vault.svelte";
  import { errorMessage } from "$lib/utils/errorMessage";
  import PasswordDialog from "../ui/PasswordDialog.svelte";

  let {
    path,
    title,
    confirmLabel = "Unlock",
    showCancel = true,
    onunlock,
    oncancel,
  }: {
    path: string;
    title: string;
    confirmLabel?: string;
    showCancel?: boolean;
    onunlock?: () => void;
    oncancel?: () => void;
  } = $props();

  let error = $state("");

  async function onSubmit(password: string) {
    error = "";
    try {
      await vault.open(path, password);
      onunlock?.();
    } catch (e) {
      error = errorMessage(e);
    }
  }
</script>

<PasswordDialog
  {title}
  vaultPath={path}
  {confirmLabel}
  {showCancel}
  errorMessage={error}
  transparentOverlay
  onconfirm={onSubmit}
  {oncancel}
/>
