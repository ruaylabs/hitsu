<script lang="ts">
  import { vault } from "$lib/stores/vault.svelte";
  import * as vaultBridge from "$lib/bridge/vault";
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
    onunlock: () => void;
    oncancel?: () => void;
  } = $props();

  let error = $state("");

  async function onSubmit(password: string) {
    error = "";
    try {
      const meta = await vaultBridge.vaultOpen(path, password);
      vault.openVault(meta);
      onunlock();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<PasswordDialog
  {title}
  {confirmLabel}
  {showCancel}
  errorMessage={error}
  transparentOverlay
  onconfirm={onSubmit}
  {oncancel}
/>
