<script lang="ts">
  import { onMount } from "svelte";
  import * as biometricBridge from "$lib/bridge/biometric";
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
  let touchIdAvailable = $state(false);
  let touchIdEnabled = $state(false);
  let mounted = false;

  onMount(() => {
    mounted = true;
    biometricBridge
      .biometricStatus(path)
      .then((status) => {
        if (!mounted) return;
        touchIdAvailable = status.available;
        touchIdEnabled = status.enabled;
      })
      .catch((cause) => console.debug("Touch ID status unavailable", cause));
    return () => {
      mounted = false;
    };
  });

  async function onSubmit(password: string) {
    error = "";
    try {
      await vault.open(path, password);
      onunlock?.();
    } catch (e) {
      error = errorMessage(e);
    }
  }

  async function onTouchIdUnlock() {
    error = "";
    try {
      await vault.unlockWithBiometric(path);
      onunlock?.();
    } catch (cause) {
      const message = errorMessage(cause);
      if (message === "Touch ID was canceled.") return;
      error = message;

      // A stale saved password is removed by the backend. Refresh status in
      // the background so the master-password fallback is re-enabled at once.
      void biometricBridge
        .biometricStatus(path)
        .then((status) => {
          if (!mounted) return;
          touchIdAvailable = status.available;
          touchIdEnabled = status.enabled;
        })
        .catch(() => {
          // Keep the existing status; the actionable unlock error is already shown.
        });
    }
  }
</script>

<PasswordDialog
  {title}
  vaultPath={path}
  {confirmLabel}
  {showCancel}
  errorMessage={error}
  alternateLabel={touchIdAvailable && touchIdEnabled ? "Unlock with Touch ID" : undefined}
  alternatePendingLabel="Waiting for Touch ID…"
  alternateIcon="fingerprint"
  onalternate={touchIdAvailable && touchIdEnabled ? onTouchIdUnlock : undefined}
  transparentOverlay
  onconfirm={onSubmit}
  {oncancel}
/>
