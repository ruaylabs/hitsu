<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import type { ThemePreference } from "$lib/bridge/prefs";
  import type { SkippedImportEntry } from "$lib/bridge/vault";
  import * as vaultBridge from "$lib/bridge/vault";
  import { app } from "$lib/stores/app.svelte";
  import { features } from "$lib/stores/features.svelte";
  import { nativeDialog } from "$lib/stores/nativeDialog.svelte";
  import { recycleBin } from "$lib/stores/recycleBin.svelte";
  import { security } from "$lib/stores/security.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { errorMessage } from "$lib/utils/errorMessage";
  import Dialog from "../ui/Dialog.svelte";
  import Icon from "../ui/Icon.svelte";
  import PasswordDialog from "../ui/PasswordDialog.svelte";

  let dialog:
    | { kind: "open" }
    | { kind: "create" }
    | { kind: "change-password" }
    | { kind: "new-password" }
    | { kind: "import-details" }
    | null = $state(null);

  let statusMsg = $state("");
  let statusError = $state(false);
  let importing = $state(false);
  let skippedEntries = $state<SkippedImportEntry[]>([]);
  let recentVaults = $state<string[]>([]);
  let lastStatusToast = "";

  $effect(() => {
    if (!statusMsg) {
      lastStatusToast = "";
      return;
    }
    if (statusMsg === lastStatusToast) return;
    lastStatusToast = statusMsg;
    if (statusError) toast.error(statusMsg);
    else toast.success(statusMsg);
  });

  async function handleOpen() {
    try {
      const result = await nativeDialog.during(() =>
        open({
          multiple: false,
          filters: [{ name: "KeePass Database", extensions: ["kdbx"] }],
        }),
      );
      if (!result) return;
      selectedPath = result;
      dialog = { kind: "open" };
    } catch (e) {
      statusError = true;
      statusMsg = String(e);
    }
  }

  let selectedPath = $state("");

  onMount(async () => {
    try {
      const prefs = await security.load();
      features.hydrate(prefs);
      recentVaults = prefs.recentVaults ?? [];
    } catch (error) {
      statusError = true;
      statusMsg = errorMessage(error);
    }
  });

  function onIdleChange(e: Event) {
    const mins = parseInt((e.target as HTMLSelectElement).value, 10);
    security.save(mins, security.clipboardClearSeconds);
  }

  function onClipboardChange(e: Event) {
    const secs = parseInt((e.target as HTMLSelectElement).value, 10);
    security.save(security.idleLockMinutes, secs);
  }

  async function onThemeChange(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value as ThemePreference;
    try {
      await theme.save(value);
    } catch (error) {
      statusError = true;
      statusMsg = errorMessage(error);
    }
  }

  async function onFoldersChange(event: Event) {
    const enabled = (event.currentTarget as HTMLInputElement).checked;
    try {
      await features.setFoldersEnabled(enabled);
      if (!enabled && selection.filter.kind === "folder") {
        selection.requestNavigation(() => {
          selection.filter = { kind: "all" };
        });
      }
    } catch (error) {
      statusError = true;
      statusMsg = errorMessage(error);
    }
  }

  async function onBrowserIntegrationChange(event: Event) {
    const enabled = (event.currentTarget as HTMLInputElement).checked;
    try {
      await features.setBrowserIntegrationEnabled(enabled);
    } catch (error) {
      statusError = true;
      statusMsg = errorMessage(error);
    }
  }

  async function doOpen(password: string) {
    dialog = null;
    try {
      await vault.open(selectedPath, password);
      app.view = "main";
    } catch (e) {
      statusError = true;
      statusMsg = String(e);
    }
  }

  async function doCreate(password: string) {
    dialog = null;
    try {
      await vault.create(selectedPath, password);
      app.view = "main";
    } catch (e) {
      statusError = true;
      statusMsg = String(e);
    }
  }

  /** Pick the destination path first, then open the password dialog —
   *  mirrors the open-vault flow so the user isn't asked for a password
   *  before choosing where the vault will live. */
  async function handleCreate() {
    try {
      const result = await nativeDialog.during(() =>
        save({
          filters: [{ name: "KeePass Database", extensions: ["kdbx"] }],
          defaultPath: "vault.kdbx",
        }),
      );
      if (!result) return;
      selectedPath = result;
      dialog = { kind: "create" };
    } catch (e) {
      statusError = true;
      statusMsg = String(e);
    }
  }

  let pendingOldPw = $state("");

  async function handleChangePassword(oldPassword: string) {
    pendingOldPw = oldPassword;
    dialog = { kind: "new-password" };
  }

  async function handleSetNewPassword(newPassword: string) {
    dialog = null;
    try {
      await vaultBridge.vaultChangePassword(pendingOldPw, newPassword);
      statusError = false;
      statusMsg = "Password changed successfully";
    } catch (e) {
      statusError = true;
      statusMsg = String(e);
    }
  }

  async function handleImport1pif() {
    importing = true;
    skippedEntries = [];
    statusMsg = "";
    try {
      // The backend opens the 1PIF file picker; keep the privacy screen away.
      const report = await nativeDialog.during(() => vaultBridge.vaultImport1pif());
      if (!report) return;
      vault.setEntries(report.entries);
      if (vault.meta) {
        vault.setMeta({ ...vault.meta, itemCount: report.entries.length, entries: report.entries });
      }
      statusError = false;
      skippedEntries = report.skippedEntries;
      const attachments = report.importedAttachments
        ? ` and ${report.importedAttachments} attachment${report.importedAttachments === 1 ? "" : "s"}`
        : "";
      const skipped = report.skippedItems ? ` (${report.skippedItems} skipped)` : "";
      statusMsg = `Imported ${report.importedItems} item${report.importedItems === 1 ? "" : "s"}${attachments}${skipped}.`;
    } catch (e) {
      statusError = true;
      statusMsg = String(e);
    } finally {
      importing = false;
    }
  }
</script>

<div class="settings-overlay" role="dialog" aria-label="Settings">
  {#if dialog}
    {#if dialog.kind === "import-details"}
      <Dialog
        title="Entries not imported"
        onclose={() => (dialog = null)}
        bodyOverflow="auto"
        bodyMaxHeight="50vh"
      >
        <ul class="skipped-list">
          {#each skippedEntries as entry}
            <li>
              <span class="skipped-title">{entry.title}</span>
              <span class="skipped-reason">{entry.reason}</span>
            </li>
          {/each}
        </ul>
      </Dialog>
    {:else if dialog.kind === "open"}
      <PasswordDialog
        title="Open vault"
        confirmLabel="Open"
        onconfirm={doOpen}
        oncancel={() => (dialog = null)}
      />
    {:else if dialog.kind === "create"}
      <PasswordDialog
        title="Create new vault"
        confirmLabel="Create"
        confirm={true}
        showStrength={true}
        minStrength={1}
        onconfirm={doCreate}
        oncancel={() => (dialog = null)}
      />
    {:else if dialog.kind === "change-password"}
      <PasswordDialog
        title="Current master password"
        confirmLabel="Next"
        onconfirm={handleChangePassword}
        oncancel={() => (dialog = null)}
      />
    {:else if dialog.kind === "new-password"}
      <PasswordDialog
        title="New master password"
        confirmLabel="Change"
        confirm={true}
        showStrength={true}
        minStrength={1}
        onconfirm={handleSetNewPassword}
        oncancel={() => (dialog = null)}
      />
    {/if}
  {/if}

  <div class="settings-pane">
    <header class="settings-header">
      <h1 class="settings-title">Settings</h1>
      <button
        class="close-btn"
        onclick={() => (app.view = "main")}
        aria-label="Close settings"
        title="Close settings"
      >
        <Icon name="x" size={16} />
      </button>
    </header>

    <div class="settings-content">
      <nav class="settings-nav" aria-label="Settings sections">
        <a href="#settings-vault">Vault</a>
        <a href="#settings-appearance">Appearance</a>
        <a href="#settings-features">Features</a>
        <a href="#settings-security">Security</a>
        <a href="#settings-about">About</a>
      </nav>

      <div class="settings-body">
        <section class="settings-section" id="settings-vault">
          <h2 class="section-heading">Vault</h2>

          <div class="vault-info">
            <span class="vault-label">Current vault</span>
            {#if vault.meta}
              <span class="vault-path">{vault.meta.name} — {vault.meta.itemCount} items</span>
              <span class="vault-path-sub">{vault.meta.path}</span>
            {:else}
              <span class="vault-none">No vault open</span>
            {/if}
          </div>

          <div class="settings-actions">
            <button class="settings-btn" onclick={handleOpen}>
              <Icon name="folder-open" size={14} />
              Open vault…
            </button>
            <button class="settings-btn" onclick={handleCreate}>
              <Icon name="plus" size={14} />
              Create new vault…
            </button>
            {#if vault.meta}
              <button class="settings-btn" onclick={() => (dialog = { kind: "change-password" })}>
                <Icon name="exchange" size={14} />
                Change master password…
              </button>
              <button class="settings-btn" onclick={handleImport1pif} disabled={importing}>
                <Icon name="database-import" size={14} />
                {importing ? "Importing…" : "Import 1Password 7 (.1pif)…"}
              </button>
            {/if}
          </div>

          {#if skippedEntries.length > 0}
            <div class="status-row">
              <button class="details-btn" onclick={() => (dialog = { kind: "import-details" })}>
                View {skippedEntries.length} skipped
                {skippedEntries.length === 1 ? "entry" : "entries"}
              </button>
            </div>
          {/if}
        </section>

        {#if vault.meta}
          <section class="settings-section">
            <h2 class="section-heading">Vault maintenance</h2>
            <div class="maintenance-card danger-card">
              <div>
                <h3 class="maintenance-title">Recycle Bin</h3>
                <p class="setting-description">
                  {recycleBin.count === 0
                  ? "The Recycle Bin is empty."
                  : `${recycleBin.count} entr${recycleBin.count === 1 ? "y" : "ies"} will be permanently deleted.`}
                </p>
              </div>
              <button
                class="settings-btn danger-btn"
                class:loading={recycleBin.emptying}
                onclick={() => recycleBin.requestEmpty()}
                disabled={recycleBin.emptying || recycleBin.count === 0}
              >
                <Icon name="trash" size={14} />
                {recycleBin.emptying ? "Emptying…" : "Empty Recycle Bin…"}
              </button>
            </div>
          </section>
        {/if}

        <section class="settings-section">
          <h2 class="section-heading">Recent vaults</h2>
          {#if recentVaults.length === 0}
            <p class="empty-text">No recent vaults.</p>
          {:else}
            <ul class="recent-list">
              {#each recentVaults as path}
                {@const active = vault.meta?.path === path}
                <li class="recent-item">
                  <button
                    class="recent-btn"
                    class:active
                    disabled={active}
                    title={active ? "Currently open" : "Open vault"}
                    onclick={async () => {
                    selectedPath = path;
                    dialog = { kind: "open" };
                  }}
                  >
                    <Icon name={active ? "check" : "database"} size={14} />
                    <span class="recent-path">{path}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </section>

        <section class="settings-section" id="settings-appearance">
          <h2 class="section-heading">Appearance</h2>
          <div class="setting-row theme-setting">
            <span class="setting-label-group">
              <span class="setting-label">Theme</span>
              <span class="setting-description">Choose a theme or follow your system setting.</span>
            </span>
            <select
              class="control control--compact control--select setting-select"
              aria-label="Theme"
              onchange={onThemeChange}
            >
              {#each [
                { value: "system", label: "System" },
                { value: "light", label: "Light" },
                { value: "dark", label: "Dark" },
              ] as option}
                <option value={option.value} selected={theme.preference === option.value}>
                  {option.label}
                </option>
              {/each}
            </select>
          </div>
        </section>

        <section class="settings-section" id="settings-features">
          <h2 class="section-heading">Features</h2>
          <label class="setting-row">
            <span class="setting-label-group">
              <span class="setting-label">Folders</span>
              <span class="setting-description"
                >Show the KDBX folder tree and entry move controls.</span
              >
            </span>
            <input
              class="setting-switch"
              type="checkbox"
              role="switch"
              aria-label="Enable folders"
              checked={features.foldersEnabled}
              onchange={onFoldersChange}
            />
          </label>
          <label class="setting-row">
            <span class="setting-label-group">
              <span class="setting-label">Browser integration</span>
              <span class="setting-description"
                >Let the Hitsu browser extension fill logins from this app (developer preview).</span
              >
            </span>
            <input
              class="setting-switch"
              type="checkbox"
              role="switch"
              aria-label="Enable browser integration"
              checked={features.browserIntegrationEnabled}
              onchange={onBrowserIntegrationChange}
            />
          </label>
        </section>

        <section class="settings-section" id="settings-security">
          <h2 class="section-heading">Security</h2>
          <div class="setting-row">
            <span class="setting-label">Lock on idle</span>
            <select
              class="control control--compact control--select setting-select"
              onchange={onIdleChange}
            >
              {#each [
              { value: 0, label: "Never" },
              { value: 1, label: "1 minute" },
              { value: 2, label: "2 minutes" },
              { value: 5, label: "5 minutes" },
              { value: 10, label: "10 minutes" },
              { value: 30, label: "30 minutes" },
              { value: 60, label: "1 hour" },
            ] as opt}
                <option value={opt.value} selected={security.idleLockMinutes === opt.value}>
                  {opt.label}
                </option>
              {/each}
            </select>
          </div>
          <div class="setting-row">
            <span class="setting-label">Clipboard clear</span>
            <select
              class="control control--compact control--select setting-select"
              onchange={onClipboardChange}
            >
              {#each [
              { value: 5, label: "5 seconds" },
              { value: 10, label: "10 seconds" },
              { value: 15, label: "15 seconds" },
              { value: 30, label: "30 seconds" },
              { value: 60, label: "1 minute" },
              { value: 0, label: "Never" },
            ] as opt}
                <option value={opt.value} selected={security.clipboardClearSeconds === opt.value}>
                  {opt.label}
                </option>
              {/each}
            </select>
          </div>
        </section>

        <section class="settings-section" id="settings-about">
          <h2 class="section-heading">About</h2>
          <div class="about-card">
            <span class="about-name">Hitsu</span>
            <span class="version">Version 0.1.0</span>
          </div>
        </section>
      </div>
    </div>
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
    z-index: var(--z-view-overlay);
  }

  .settings-pane {
    width: min(720px, calc(100vw - 40px));
    height: min(680px, 80vh);
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
    width: var(--icon-button-size);
    height: var(--icon-button-size);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }

  .close-btn:hover {
    background: var(--border);
  }

  .settings-content {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .settings-nav {
    display: flex;
    flex: 0 0 132px;
    flex-direction: column;
    gap: 3px;
    padding: 16px 10px;
    border-right: 0.5px solid var(--border);
    background: var(--surface-1);
  }

  .settings-nav a {
    padding: 7px 10px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 12.5px;
    text-decoration: none;
  }

  .settings-nav a:hover,
  .settings-nav a:focus-visible {
    background: var(--border);
    color: var(--text-primary);
  }

  .settings-body {
    flex: 1;
    min-width: 0;
    padding: 20px 24px;
    overflow-y: auto;
    scroll-behavior: smooth;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .settings-section {
    scroll-margin-top: 20px;
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

  .settings-btn:hover:not(:disabled) {
    background: var(--border);
  }

  .settings-btn:disabled {
    cursor: wait;
    opacity: 0.65;
  }

  .maintenance-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-1);
  }

  .danger-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .maintenance-title {
    margin: 0 0 2px;
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 500;
  }

  .danger-card {
    flex-direction: row;
  }

  .danger-btn {
    flex-shrink: 0;
    color: var(--danger);
  }

  .danger-btn:disabled {
    cursor: not-allowed;
  }

  .danger-btn.loading:disabled {
    cursor: wait;
  }

  .status-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .details-btn {
    color: var(--accent);
    font-size: 12px;
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .details-btn:hover {
    color: var(--text-accent);
  }

  .skipped-list {
    margin: 0;
    padding-left: 20px;
    color: var(--text-primary);
    font-size: 13px;
  }

  .skipped-list li + li {
    margin-top: 10px;
  }

  .skipped-title,
  .skipped-reason {
    display: block;
  }

  .skipped-title {
    font-weight: 500;
  }

  .skipped-reason {
    margin-top: 2px;
    color: var(--text-muted);
    font-size: 12px;
  }

  .empty-text {
    font-size: 13px;
    color: var(--text-muted);
    font-style: italic;
  }

  .recent-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .recent-item {
    margin: 0;
    padding: 0;
  }

  .recent-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 10px;
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--surface-1);
    color: var(--text-primary);
    font-size: 13px;
    text-align: left;
    transition: background 0.1s;
  }

  .recent-btn:hover {
    background: var(--border);
  }

  .recent-btn:disabled {
    cursor: default;
    opacity: 0.7;
  }

  .recent-btn.active {
    background: var(--surface-2);
    border-color: var(--accent);
    color: var(--accent);
  }

  .recent-btn.active .recent-path {
    color: var(--text-primary);
  }

  .recent-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-muted);
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0;
  }

  .theme-setting {
    gap: 24px;
  }

  .setting-label-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .setting-label {
    font-size: 13px;
    color: var(--text-primary);
  }

  .setting-description {
    color: var(--text-muted);
    font-size: 11.5px;
  }

  .setting-select {
    min-width: 120px;
    padding-block: 5px;
    padding-left: 10px;
  }

  .setting-switch {
    position: relative;
    width: 32px;
    height: 18px;
    flex-shrink: 0;
    appearance: none;
    -webkit-appearance: none;
    border: 1px solid var(--border-strong);
    border-radius: 999px;
    background: var(--border-strong);
    cursor: pointer;
    transition:
      background var(--transition-fast),
      border-color var(--transition-fast);
  }

  .setting-switch::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--surface-2);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
    transition: transform var(--transition-fast);
  }

  .setting-switch:checked {
    border-color: var(--accent);
    background: var(--accent);
  }

  .setting-switch:checked::after {
    transform: translateX(14px);
  }

  .setting-switch:focus-visible {
    box-shadow: 0 0 0 2px var(--bg-accent);
  }

  .about-card {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 12px;
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-1);
  }

  .about-name {
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 500;
  }

  .version {
    font-size: 11.5px;
    color: var(--text-muted);
  }
</style>
