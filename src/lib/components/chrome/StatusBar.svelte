<script lang="ts">
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { saveStatus } from "$lib/stores/saveStatus.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import Icon from "../ui/Icon.svelte";

  let {
    onHelpClick,
    onSettingsClick,
  }: {
    onHelpClick: () => void;
    onSettingsClick: () => void;
  } = $props();

  let itemCount = $derived(vault.entries.length);
  let statusLabel = $derived(
    saveStatus.state === "dirty"
      ? "Unsaved changes"
      : saveStatus.state === "saving"
        ? "Saving…"
        : saveStatus.state === "error"
          ? "Save failed"
          : "Saved",
  );
</script>

<footer class="statusbar">
  <div
    class="statusbar-left"
    role="status"
    aria-live="polite"
    title={saveStatus.state === "error" ? saveStatus.errorMessage : statusLabel}
  >
    <span
      class="sync-dot"
      class:dirty={saveStatus.state === "dirty"}
      class:saving={saveStatus.state === "saving"}
      class:error={saveStatus.state === "error"}
    ></span>
    <span>{statusLabel}</span>
  </div>
  <div class="statusbar-right">
    <span>{itemCount} items</span>
    {#if clipboard.active}
      <span class="sep">·</span>
      <span class="countdown">Clears in {Math.ceil(clipboard.remainingMs / 1000)}s</span>
    {/if}
    {#if vault.meta}
      <button
        class="lock-btn"
        onclick={() => vault.lock()}
        aria-label="Lock vault"
        title="Lock vault"
      >
        <Icon name="lock" size={12} />
      </button>
    {/if}
    <button class="settings-gear" onclick={onSettingsClick} aria-label="Settings" title="Settings">
      <Icon name="settings" size={12} />
    </button>
    <button
      class="settings-gear"
      onclick={onHelpClick}
      aria-label="Keyboard shortcuts"
      title="Keyboard shortcuts"
    >
      <Icon name="help-circle" size={12} />
    </button>
  </div>
</footer>

<style>
  .statusbar {
    height: var(--statusbar-height);
    padding: 0 14px;
    background: var(--surface-1);
    border-top: 0.5px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .statusbar-left {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .sync-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--success);
    flex-shrink: 0;
  }

  .sync-dot.dirty {
    background: var(--warning);
  }

  .sync-dot.saving {
    background: var(--accent);
    animation: status-pulse 0.8s ease-in-out infinite alternate;
  }

  .sync-dot.error {
    background: var(--danger);
  }

  @keyframes status-pulse {
    to {
      opacity: 0.35;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .sync-dot.saving {
      animation: none;
    }
  }

  .statusbar-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .sep {
    color: var(--text-muted);
  }

  .lock-btn,
  .settings-gear {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 3px;
    color: var(--text-muted);
  }

  .lock-btn:hover,
  .settings-gear:hover {
    background: var(--border);
    color: var(--text-secondary);
  }
</style>
