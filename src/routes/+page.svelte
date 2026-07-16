<script lang="ts">
  import { onMount } from "svelte";
  import MainApp from "$lib/components/MainApp.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import OnboardingView from "$lib/components/unlock/OnboardingView.svelte";
  import UnlockScreen from "$lib/components/unlock/UnlockScreen.svelte";
  import { security } from "$lib/stores/security.svelte";
  import { vault } from "$lib/stores/vault.svelte";

  let startupDialog: "password" | null = $state(null);
  let startupPath = $state("");
  let startupChecked = $state(false);
  let windowFocused = $state(true);

  onMount(() => {
    windowFocused = document.hasFocus();

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

<svelte:window onfocus={() => (windowFocused = true)} onblur={() => (windowFocused = false)} />

<div class="app-content" inert={!windowFocused} aria-hidden={!windowFocused}>
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
    <UnlockScreen path={vault.meta!.path} title="Locked" confirmLabel="Unlock" showCancel={false} />
  {:else if !startupChecked}
  <!-- Waiting for startup check — show blank -->
  {:else if !vault.meta}
    <OnboardingView />
  {:else}
    <MainApp />
  {/if}
</div>

{#if !windowFocused}
  <div class="privacy-screen" role="status" aria-label="Privacy screen">
    <Icon name="lock" size={24} />
    <span>Kagi is hidden while unfocused</span>
  </div>
{/if}

<style>
  .app-content {
    width: 100%;
    height: 100%;
  }

  .privacy-screen {
    position: fixed;
    inset: 0;
    z-index: var(--z-blocking-overlay);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: var(--text-secondary);
    background: var(--surface-0);
    font-size: 13px;
  }
</style>
