<script lang="ts">
  import { vault } from "$lib/stores/vault.svelte";
  import * as vaultBridge from "$lib/bridge/vault";
  import * as entriesBridge from "$lib/bridge/entries";
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
      vault.setMeta(meta);
      const summaries = await entriesBridge.entriesList();
      vault.setEntries(summaries);
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
