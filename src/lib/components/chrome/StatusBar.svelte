<script lang="ts">
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { saveStatus } from "$lib/stores/saveStatus.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import Icon from "../ui/Icon.svelte";

  let {
    onHelpClick,
    onSettingsClick,
  }: {
    onHelpClick: () => void;
    onSettingsClick: () => void;
  } = $props();

  let itemCount = $derived(vault.entries.filter((entry) => !entry.trashed).length);
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
      <button
        type="button"
        class="countdown"
        onclick={() => clipboard.cancel()}
        aria-label="Clear clipboard now"
        title="Clear clipboard now"
      >
        Clears in {Math.ceil(clipboard.remainingMs / 1000)}s
      </button>
    {/if}
    {#if vault.meta}
      <button
        class="lock-btn"
        onclick={() => selection.requestNavigation(() => void vault.lock())}
        aria-label="Lock vault"
        title="Lock vault (⌘L)"
      >
        <Icon name="lock" size={12} />
        <span class="lock-name">{vault.meta.name}</span>
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

  .countdown {
    color: var(--text-muted);
    text-decoration: underline;
    text-decoration-color: transparent;
    text-underline-offset: 2px;
  }

  .countdown:hover {
    color: var(--text-secondary);
    text-decoration-color: currentColor;
  }

  .lock-btn,
  .settings-gear {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 20px;
    border-radius: 3px;
    color: var(--text-muted);
  }

  .settings-gear {
    width: 20px;
  }

  .lock-btn {
    gap: 5px;
    padding: 0 6px;
  }

  .lock-name {
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .lock-btn:hover,
  .settings-gear:hover {
    background: var(--border);
    color: var(--text-secondary);
  }
</style>
