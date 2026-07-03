<script lang="ts">
  import { onMount } from "svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { security } from "$lib/stores/security.svelte";
  import UnlockScreen from "$lib/components/unlock/UnlockScreen.svelte";
  import OnboardingView from "$lib/components/unlock/OnboardingView.svelte";
  import MainApp from "$lib/components/MainApp.svelte";

  let startupDialog: "password" | null = $state(null);
  let startupPath = $state("");
  let startupChecked = $state(false);

  onMount(() => {
    // Load preferences — startup vault and security settings
    security
      .load()
      .then((prefs) => {
        if (prefs.lastVault) {
          startupPath = prefs.lastVault;
          startupDialog = "password";
        }
        startupChecked = true;
      })
      .catch(() => {
        startupChecked = true;
      });
  });
</script>

{#if startupDialog === "password"}
  <UnlockScreen
    path={startupPath}
    title="Unlock vault"
    confirmLabel="Unlock"
    onunlock={() => (startupDialog = null)}
    oncancel={() => {
      startupDialog = null;
      vault.setMeta(null);
    }}
  />
{:else if vault.locked && vault.meta}
  <UnlockScreen
    path={vault.meta!.path}
    title="Locked"
    confirmLabel="Unlock"
    showCancel={false}
    onunlock={() => vault.unlock()}
  />
{:else if !startupChecked}
<!-- Waiting for startup check — show blank -->
{:else if !vault.meta}
  <OnboardingView />
{:else}
  <MainApp />
{/if}
