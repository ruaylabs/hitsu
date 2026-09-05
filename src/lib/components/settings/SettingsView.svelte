<script lang="ts">
  import { onMount } from "svelte";
  import type { BiometricStatus } from "$lib/bridge/biometric";
  import * as biometricBridge from "$lib/bridge/biometric";
  import * as entriesBridge from "$lib/bridge/entries";
  import type { ThemePreference } from "$lib/bridge/prefs";
  import type { ImportReport, SkippedImportEntry } from "$lib/bridge/vault";
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
  import { pickVaultToCreate, pickVaultToOpen } from "$lib/utils/vaultFilePicker";
  import Button from "../ui/Button.svelte";

  const appVersion = import.meta.env.VITE_APP_VERSION;

  import Dialog from "../ui/Dialog.svelte";
  import Icon from "../ui/Icon.svelte";
  import PasswordDialog from "../ui/PasswordDialog.svelte";

  let dialog:
    | { kind: "open" }
    | { kind: "create" }
    | { kind: "change-password" }
    | { kind: "new-password" }
    | { kind: "touch-id-enable" }
    | { kind: "import-confirm" }
    | { kind: "import-details" }
    | { kind: "favicon-confirm" }
    | null = $state(null);

  let statusMsg = $state("");
  let statusError = $state(false);
  let importing = $state(false);
  let downloadingFavicons = $state(false);
  let importReport = $state<ImportReport | null>(null);
  let importError = $state("");
  let skippedEntries = $state<SkippedImportEntry[]>([]);
  let recentVaults = $state<string[]>([]);
  let vaultItemCount = $derived(vault.entries.length);
  let biometric = $state<BiometricStatus>({ available: false, enabled: false });
  let biometricBusy = $state(false);
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
      const result = await pickVaultToOpen();
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

    const path = vault.meta?.path;
    if (!path) return;
    try {
      biometric = await biometricBridge.biometricStatus(path);
    } catch (error) {
      console.debug("Touch ID status unavailable", error);
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

  async function onTouchIdChange(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    input.checked = biometric.enabled;
    const path = vault.meta?.path;
    if (!path || biometricBusy) return;

    if (!biometric.enabled) {
      dialog = { kind: "touch-id-enable" };
      return;
    }

    biometricBusy = true;
    try {
      await biometricBridge.biometricDisable(path);
      biometric = { ...biometric, enabled: false };
      statusError = false;
      statusMsg = "Touch ID disabled";
    } catch (error) {
      statusError = true;
      statusMsg = errorMessage(error);
    } finally {
      biometricBusy = false;
    }
  }

  async function enableTouchId(password: string) {
    const path = vault.meta?.path;
    if (!path) return;
    await biometricBridge.biometricEnable(path, password);
    biometric = { available: true, enabled: true };
    dialog = null;
    statusError = false;
    statusMsg = "Touch ID enabled";
  }

  async function doOpen(password: string) {
    await vault.open(selectedPath, password);
    dialog = null;
    app.view = "main";
  }

  async function doCreate(password: string) {
    await vault.create(selectedPath, password);
    dialog = null;
    app.view = "main";
  }

  /** Pick the destination path first, then open the password dialog —
   *  mirrors the open-vault flow so the user isn't asked for a password
   *  before choosing where the vault will live. */
  async function handleCreate() {
    try {
      const result = await pickVaultToCreate();
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
    await vaultBridge.vaultChangePassword(pendingOldPw, newPassword);
    pendingOldPw = "";
    biometric = { ...biometric, enabled: false };
    dialog = null;
    statusError = false;
    statusMsg = "Password changed successfully";
  }

  function requestImport() {
    dialog = { kind: "import-confirm" };
  }

  function csvCell(value: string) {
    return `"${value.replaceAll('"', '""')}"`;
  }

  function skippedReportCsv() {
    const rows = [
      ["Status", "Title", "Reason"],
      ...skippedEntries.map((entry) => [
        entry.failed ? "Failed" : "Skipped",
        entry.title,
        entry.reason,
      ]),
    ];
    return rows.map((row) => row.map(csvCell).join(",")).join("\n");
  }

  async function exportSkippedReport() {
    try {
      // The backend owns the save dialog; wrap the call so the privacy
      // screen stays away while it is open.
      const written = await nativeDialog.during(() =>
        vaultBridge.importReportExport(skippedReportCsv()),
      );
      if (!written) return;
      statusError = false;
      statusMsg = "Import report exported.";
    } catch (error) {
      statusError = true;
      statusMsg = errorMessage(error);
    }
  }

  async function downloadFavicons() {
    dialog = null;
    downloadingFavicons = true;
    statusMsg = "";
    try {
      const report = await entriesBridge.entriesDownloadFavicons();
      vault.setEntries(report.entries);
      if (vault.meta) {
        vault.setMeta({ ...vault.meta, entries: report.entries });
      }
      statusError = false;
      const failed = report.failed ? ` ${report.failed} failed.` : "";
      statusMsg = `Downloaded ${report.downloaded} website icon${report.downloaded === 1 ? "" : "s"}.${failed}`;
    } catch (error) {
      statusError = true;
      statusMsg = errorMessage(error);
    } finally {
      downloadingFavicons = false;
    }
  }

  async function handleImport1pif() {
    dialog = null;
    importing = true;
    importError = "";
    statusMsg = "";
    try {
      // The backend opens the 1PIF file picker; keep the privacy screen away.
      const report = await nativeDialog.during(() => vaultBridge.vaultImport1pif());
      if (!report) return;
      vault.setEntries(report.entries);
      if (vault.meta) {
        vault.setMeta({ ...vault.meta, entries: report.entries });
      }
      statusError = false;
      importReport = report;
      skippedEntries = report.skippedEntries;
      const skippedItems = report.skippedItems - report.failedItems;
      const skipped = skippedItems ? ` ${skippedItems} skipped.` : "";
      const failed = report.failedItems ? ` ${report.failedItems} failed.` : "";
      statusMsg = `Imported ${report.importedItems} item${report.importedItems === 1 ? "" : "s"}.${skipped}${failed}`;
    } catch (error) {
      importError = errorMessage(error);
      statusError = true;
      statusMsg = importError;
    } finally {
      importing = false;
    }
  }
</script>

<div class="settings-overlay" role="dialog" aria-label="Settings">
  {#if dialog}
    {#if dialog.kind === "import-details"}
      <Dialog
        title="Items not imported"
        onclose={() => (dialog = null)}
        bodyOverflow="auto"
        bodyMaxHeight="50vh"
      >
        <ul class="skipped-list">
          {#each skippedEntries as entry}
            <li>
              <span class:failed={entry.failed} class="import-result-kind">
                {entry.failed ? "Failed" : "Skipped"}
              </span>
              <span class="skipped-title">{entry.title}</span>
              <span class="skipped-reason">{entry.reason}</span>
            </li>
          {/each}
        </ul>
        {#snippet footer()}
          <Button onclick={exportSkippedReport}>
            <Icon name="download" size={14} />
            Export CSV…
          </Button>
          <Button variant="primary" onclick={() => (dialog = null)}>Done</Button>
        {/snippet}
      </Dialog>
    {:else if dialog.kind === "favicon-confirm"}
      <Dialog title="Download website icons" onclose={() => (dialog = null)}>
        <div class="import-confirm">
          <p>Download missing website icons for entries in this vault?</p>
          <p>
            This contacts the domains stored in your vault, which may reveal which services you use.
            Existing custom icons will not be replaced.
          </p>
        </div>
        {#snippet footer()}
          <Button onclick={() => (dialog = null)}>Cancel</Button>
          <Button variant="primary" onclick={downloadFavicons}>Download icons</Button>
        {/snippet}
      </Dialog>
    {:else if dialog.kind === "import-confirm"}
      <Dialog title="Import 1Password 7 data" onclose={() => (dialog = null)}>
        <div class="import-confirm">
          <p>Import into <strong>{vault.meta?.name ?? "the current vault"}</strong>?</p>
          <p>
            Items and attachments from the selected .1pif export will be merged into this vault.
            Existing entries will remain unchanged.
          </p>
        </div>
        {#snippet footer()}
          <Button onclick={() => (dialog = null)}>Cancel</Button>
          <Button variant="primary" onclick={handleImport1pif}>Choose export…</Button>
        {/snippet}
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
        showRecoveryWarning={true}
        minStrength={1}
        onconfirm={doCreate}
        oncancel={() => (dialog = null)}
      />
    {:else if dialog.kind === "touch-id-enable"}
      <PasswordDialog
        title="Enable Touch ID"
        vaultPath={vault.meta?.path}
        confirmLabel="Enable"
        pendingLabel="Enabling…"
        onconfirm={enableTouchId}
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
        showRecoveryWarning={true}
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
        {#if vault.meta}
          <a href="#settings-maintenance">Maintenance</a>
        {/if}
        <a href="#settings-recent">Recent vaults</a>
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
              <span class="vault-path">{vault.meta.name} — {vaultItemCount} items</span>
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
              <button
                class="settings-btn"
                onclick={() => (dialog = { kind: "favicon-confirm" })}
                disabled={importing || downloadingFavicons}
              >
                <Icon name="photo-down" size={14} />
                {downloadingFavicons ? "Downloading icons…" : "Download website icons…"}
              </button>
              <button
                class="settings-btn"
                onclick={requestImport}
                disabled={importing || downloadingFavicons}
              >
                <Icon name="database-import" size={14} />
                {importing ? "Importing…" : "Import 1Password 7 (.1pif)…"}
              </button>
            {/if}
          </div>

          {#if importing}
            <div class="import-progress" role="status" aria-label="Import in progress">
              <span class="import-spinner" aria-hidden="true"></span>
              <div>
                <strong>Importing items…</strong>
                <span>Reading entries and attachments. Large exports may take a while.</span>
              </div>
            </div>
          {:else if downloadingFavicons}
            <div class="import-progress" role="status" aria-label="Icon download in progress">
              <span class="import-spinner" aria-hidden="true"></span>
              <div>
                <strong>Downloading website icons…</strong>
                <span>Large vaults may take a while. The vault will be saved once.</span>
              </div>
            </div>
          {:else if importError}
            <div class="import-failure" role="alert">
              <Icon name="alert-triangle" size={16} />
              <div>
                <strong>Import failed</strong>
                <span>{importError}</span>
              </div>
              <button class="details-btn" onclick={requestImport}>Try again</button>
            </div>
          {:else if importReport}
            {@const skippedItems = importReport.skippedItems - importReport.failedItems}
            <div class="import-summary">
              <div class="import-summary-heading" role="status">
                <Icon name="circle-check" size={16} />
                <strong>Import complete</strong>
              </div>
              <dl class="import-counts">
                <div>
                  <dt>Imported</dt>
                  <dd>{importReport.importedItems}</dd>
                </div>
                <div>
                  <dt>Attachments</dt>
                  <dd>{importReport.importedAttachments}</dd>
                </div>
                <div>
                  <dt>Skipped</dt>
                  <dd>{skippedItems}</dd>
                </div>
                <div>
                  <dt>Failed</dt>
                  <dd>{importReport.failedItems}</dd>
                </div>
              </dl>
              <button class="details-btn" onclick={() => (dialog = { kind: "favicon-confirm" })}>
                Download missing website icons
              </button>
              {#if skippedEntries.length > 0}
                <button class="details-btn" onclick={() => (dialog = { kind: "import-details" })}>
                  Review {skippedEntries.length} item{skippedEntries.length === 1 ? "" : "s"}
                  not imported
                </button>
              {/if}
            </div>
          {/if}
        </section>

        {#if vault.meta}
          <section class="settings-section" id="settings-maintenance">
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
                onclick={() => recycleBin.requestEmpty()}
                disabled={recycleBin.emptying || recycleBin.count === 0}
              >
                <Icon name="trash" size={14} />
                {recycleBin.emptying ? "Emptying…" : "Empty Recycle Bin…"}
              </button>
            </div>
          </section>
        {/if}

        <section class="settings-section" id="settings-recent">
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
          {#if vault.meta && (biometric.available || biometric.enabled)}
            <label class="setting-row">
              <span class="setting-label-group">
                <span class="setting-label">Unlock with Touch ID</span>
                <span class="setting-description">
                  Store this vault's master password in the device-only macOS Keychain.
                </span>
              </span>
              <input
                class="setting-switch"
                type="checkbox"
                role="switch"
                aria-label="Unlock with Touch ID"
                checked={biometric.enabled}
                disabled={biometricBusy}
                onchange={onTouchIdChange}
              />
            </label>
          {/if}
          <div class="setting-row">
            <label class="setting-label" for="setting-idle-lock">Lock on idle</label>
            <select
              id="setting-idle-lock"
              class="control control--compact control--select setting-select"
              onchange={onIdleChange}
            >
              {#each [
              ...(import.meta.env.DEV ? [{ value: 0, label: "Never (dev)" }] : []),
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
            <label class="setting-label" for="setting-clipboard-clear">Clipboard clear</label>
            <select
              id="setting-clipboard-clear"
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
            <span class="version">Version {appVersion}</span>
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
    padding: var(--space-4) var(--space-5);
    border-bottom: 0.5px solid var(--border);
  }

  .settings-title {
    font-size: var(--text-lg);
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
    gap: var(--space-1);
    padding: var(--space-4) var(--space-3);
    border-right: 0.5px solid var(--border);
    background: var(--surface-1);
  }

  .settings-nav a {
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: var(--text-base);
  }

  .settings-nav a:hover,
  .settings-nav a:focus-visible {
    background: var(--border);
    color: var(--text-primary);
  }

  .settings-body {
    flex: 1;
    min-width: 0;
    padding: var(--space-5) var(--space-6);
    overflow-y: auto;
    scroll-behavior: smooth;
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .settings-section {
    scroll-margin-top: 20px;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .section-heading {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .vault-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-half);
    padding: var(--space-3) var(--space-3);
    background: var(--surface-1);
    border-radius: var(--radius);
  }

  .vault-label {
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .vault-path {
    font-size: var(--text-base);
    color: var(--text-primary);
  }

  .vault-path-sub {
    font-size: var(--text-base);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .vault-none {
    font-size: var(--text-base);
    color: var(--text-muted);
    font-style: italic;
  }

  .settings-actions {
    display: flex;
    gap: var(--space-2);
    flex-wrap: wrap;
  }

  .settings-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: var(--text-base);
    color: var(--text-primary);
    background: var(--surface-1);
    transition: background var(--transition-fast);
  }

  .settings-btn:hover:not(:disabled) {
    background: var(--border);
  }

  .maintenance-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3);
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-1);
  }

  .danger-card {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .maintenance-title {
    margin: 0 0 2px;
    color: var(--text-primary);
    font-size: var(--text-base);
    font-weight: 500;
  }

  .danger-btn {
    flex-shrink: 0;
    color: var(--danger);
  }

  .import-confirm {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    color: var(--text-secondary);
    font-size: var(--text-base);
    line-height: var(--leading-normal);
  }

  .import-confirm strong {
    color: var(--text-primary);
  }

  .import-progress,
  .import-failure,
  .import-summary {
    padding: var(--space-3);
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
  }

  .import-progress,
  .import-failure {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .import-progress strong,
  .import-progress span,
  .import-failure strong,
  .import-failure span {
    display: block;
  }

  .import-progress strong,
  .import-failure strong {
    color: var(--text-primary);
    font-size: var(--text-base);
    font-weight: 500;
  }

  .import-progress div > span,
  .import-failure div > span {
    margin-top: 2px;
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .import-failure {
    border-color: color-mix(in srgb, var(--danger) 45%, var(--border));
  }

  .import-failure > div {
    flex: 1;
  }

  .import-failure > :global(.ti) {
    color: var(--danger);
  }

  .import-spinner {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    border: 2px solid var(--border-strong);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: import-spin 0.75s linear infinite;
  }

  @keyframes import-spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .import-spinner {
      animation: none;
    }
  }

  .import-summary {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .import-summary-heading {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--success);
    font-size: var(--text-base);
  }

  .import-counts {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .import-counts div {
    padding: var(--space-2) var(--space-2);
    border-right: 0.5px solid var(--border);
    text-align: center;
  }

  .import-counts div:last-child {
    border-right: 0;
  }

  .import-counts dt {
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .import-counts dd {
    margin: 2px 0 0;
    color: var(--text-primary);
    font-size: var(--text-md);
    font-weight: 500;
  }

  .details-btn {
    color: var(--accent);
    font-size: var(--text-sm);
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .details-btn:hover {
    color: var(--text-accent);
  }

  .skipped-list {
    padding-left: var(--space-5);
    color: var(--text-primary);
    font-size: var(--text-base);
  }

  .skipped-list li + li {
    margin-top: 10px;
  }

  .import-result-kind {
    display: inline-block;
    margin-bottom: 3px;
    padding: var(--space-half) var(--space-1);
    color: var(--text-secondary);
    background: var(--surface-1);
    border-radius: var(--radius-xs);
    font-size: var(--text-xs);
    font-weight: 500;
    text-transform: uppercase;
  }

  .import-result-kind.failed {
    color: var(--danger);
    background: var(--danger-bg);
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
    font-size: var(--text-sm);
  }

  .empty-text {
    font-size: var(--text-base);
    color: var(--text-muted);
    font-style: italic;
  }

  .recent-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .recent-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--surface-1);
    color: var(--text-primary);
    font-size: var(--text-base);
    text-align: left;
    transition: background var(--transition-fast);
  }

  .recent-btn:hover:not(:disabled) {
    background: var(--border);
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
    padding: var(--space-2) 0;
  }

  .theme-setting {
    gap: var(--space-6);
  }

  .setting-label-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-half);
  }

  .setting-label {
    font-size: var(--text-base);
    color: var(--text-primary);
  }

  .setting-description {
    color: var(--text-muted);
    font-size: var(--text-base);
  }

  .setting-select {
    min-width: 120px;
    padding-block: var(--space-1);
    padding-left: var(--space-3);
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
    padding: var(--space-3);
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
    background: var(--surface-1);
  }

  .about-name {
    color: var(--text-primary);
    font-size: var(--text-base);
    font-weight: 500;
  }

  .version {
    font-size: var(--text-sm);
    color: var(--text-muted);
  }
</style>
