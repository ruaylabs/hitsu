<script lang="ts">
  import { app } from "$lib/stores/app.svelte";
  import Icon from "../ui/Icon.svelte";

  // Mock recent vaults
  let recentVaults = $state([
    { path: "~/Dropbox/Apps/Kagi/Personal.kdbx", name: "Personal", itemCount: 24 },
  ]);
</script>

<div class="settings-overlay" role="dialog" aria-label="Settings">
  <div class="settings-pane">
    <header class="settings-header">
      <h1 class="settings-title">Settings</h1>
      <button class="close-btn" onclick={() => (app.view = "main")} aria-label="Close settings">
        <Icon name="x" size={16} />
      </button>
    </header>

    <div class="settings-body">
      <section class="settings-section">
        <h2 class="section-heading">Vault</h2>

        <div class="vault-info">
          <span class="vault-label">Current vault</span>
          {#if recentVaults.length > 0}
            <span class="vault-path"
              >{recentVaults[0].name}
              — {recentVaults[0].itemCount} items</span
            >
            <span class="vault-path-sub">{recentVaults[0].path}</span>
          {:else}
            <span class="vault-none">No vault open</span>
          {/if}
        </div>

        <div class="settings-actions">
          <button class="settings-btn" onclick={() => {}}>
            <Icon name="folder-open" size={14} />
            Open vault…
          </button>
          <button class="settings-btn" onclick={() => {}}>
            <Icon name="plus" size={14} />
            Create new vault…
          </button>
          {#if recentVaults.length > 0}
            <button class="settings-btn" onclick={() => {}}>
              <Icon name="exchange" size={14} />
              Change master password…
            </button>
          {/if}
        </div>
      </section>

      <section class="settings-section">
        <h2 class="section-heading">Recent vaults</h2>
        {#if recentVaults.length > 0}
          <div class="recent-list">
            {#each recentVaults as rv}
              <button class="recent-row" onclick={() => {}}>
                <Icon name="database" size={14} />
                <div class="recent-info">
                  <span class="recent-name">{rv.name}</span>
                  <span class="recent-path">{rv.path}</span>
                </div>
              </button>
            {/each}
          </div>
        {:else}
          <p class="empty-text">No recent vaults.</p>
        {/if}
      </section>

      <section class="settings-section">
        <h2 class="section-heading">Security</h2>
        <div class="setting-row">
          <span class="setting-label">Lock on idle</span>
          <span class="setting-value">5 minutes</span>
        </div>
        <div class="setting-row">
          <span class="setting-label">Clipboard clear</span>
          <span class="setting-value">15 seconds</span>
        </div>
      </section>
    </div>

    <footer class="settings-footer">
      <span class="version">Kagi 0.1.0</span>
    </footer>
  </div>
</div>

<style>
  .settings-overlay {
    position: absolute;
    inset: 0;
    background: var(--surface-0);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .settings-pane {
    width: 520px;
    max-height: 80vh;
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-card);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 0.5px solid var(--border);
  }

  .settings-title {
    font-size: 18px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }

  .close-btn:hover {
    background: var(--border);
  }

  .settings-body {
    padding: 20px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .section-heading {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .vault-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 12px;
    background: var(--surface-1);
    border-radius: var(--radius);
  }

  .vault-label {
    font-size: 11px;
    color: var(--text-muted);
  }

  .vault-path {
    font-size: 13.5px;
    color: var(--text-primary);
  }

  .vault-path-sub {
    font-size: 11.5px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .vault-none {
    font-size: 13.5px;
    color: var(--text-muted);
    font-style: italic;
  }

  .settings-actions {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .settings-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 13px;
    color: var(--text-primary);
    background: var(--surface-1);
    transition: background 0.1s;
  }

  .settings-btn:hover {
    background: var(--border);
  }

  .recent-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .recent-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    width: 100%;
    text-align: left;
    color: var(--text-primary);
  }

  .recent-row:hover {
    background: var(--bg-accent);
  }

  .recent-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .recent-name {
    font-size: 13px;
    color: var(--text-primary);
  }

  .recent-path {
    font-size: 11.5px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty-text {
    font-size: 13px;
    color: var(--text-muted);
    font-style: italic;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0;
  }

  .setting-label {
    font-size: 13px;
    color: var(--text-primary);
  }

  .setting-value {
    font-size: 13px;
    color: var(--text-muted);
  }

  .settings-footer {
    padding: 10px 20px;
    border-top: 0.5px solid var(--border);
    display: flex;
    justify-content: center;
  }

  .version {
    font-size: 11.5px;
    color: var(--text-muted);
  }
</style>
