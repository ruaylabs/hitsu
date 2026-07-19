<script lang="ts" module>
  import { security } from "$lib/stores/security.svelte";

  // Start the preferences fetch at module-evaluation time so the IPC
  // roundtrip overlaps with Svelte mounting instead of running after it —
  // the unlock prompt is gated on this result. Errors are swallowed here
  // (resolving null) so a rejection can't fire before onMount subscribes.
  const startupPrefs = security.load().catch((error) => {
    console.error("Failed to load startup preferences", error);
    return null;
  });
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import type MainAppComponent from "$lib/components/MainApp.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import OnboardingView from "$lib/components/unlock/OnboardingView.svelte";
  import UnlockScreen from "$lib/components/unlock/UnlockScreen.svelte";
  import { nativeDialog } from "$lib/stores/nativeDialog.svelte";
  import { vault } from "$lib/stores/vault.svelte";

  let startupDialog: "password" | null = $state(null);
  let startupPath = $state("");
  let startupChecked = $state(false);
  let windowFocused = $state(true);
  // MainApp (and everything it pulls in) is split out of the startup chunk so
  // the unlock prompt renders after parsing only the small shell. The chunk
  // loads in the background while the user types their master password.
  let MainApp = $state<typeof MainAppComponent | null>(null);
  // Hitsu-owned native dialogs (file pickers) steal focus too; don't blank
  // the app behind a dialog the user just opened. True focus loss (switching
  // apps) still hides content instantly.
  let concealed = $derived(!windowFocused && !nativeDialog.open);

  onMount(() => {
    windowFocused = document.hasFocus();

    void import("$lib/components/MainApp.svelte")
      .then((module) => (MainApp = module.default))
      .catch((error) => console.error("Failed to load the main app", error));

    // Startup vault and security settings; the fetch is already in flight
    // (see the module script). null means it failed — proceed to onboarding.
    startupPrefs.then((prefs) => {
      if (prefs?.lastVault) {
        startupPath = prefs.lastVault;
        startupDialog = "password";
      }
      startupChecked = true;
    });
  });
</script>

<svelte:window onfocus={() => (windowFocused = true)} onblur={() => (windowFocused = false)} />

<div class="app-content" inert={!windowFocused} aria-hidden={concealed}>
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
    <!-- Closing the locked prompt forgets the current vault selection and
         returns to onboarding so another vault can be opened. The backend
         already dropped the decrypted data when the vault locked. -->
    <UnlockScreen
      path={vault.meta!.path}
      title="Locked"
      confirmLabel="Unlock"
      oncancel={() => vault.setMeta(null)}
    />
  {:else if !startupChecked}
  <!-- Waiting for startup check — show blank -->
  {:else if !vault.meta}
    <OnboardingView />
  {:else if MainApp}
    <MainApp />
  {/if}
</div>

{#if concealed}
  <div class="privacy-screen" role="status" aria-label="Privacy screen">
    <Icon name="lock" size={24} />
    <span>Hitsu is hidden while unfocused</span>
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
