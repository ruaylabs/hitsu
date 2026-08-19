<script lang="ts">
  import { untrack } from "svelte";
  import * as entriesBridge from "$lib/bridge/entries";
  import { type EntryPatch, toSummary } from "$lib/bridge/entries";
  import type { Entry } from "$lib/bridge/types";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { entryDeletion } from "$lib/stores/entryDeletion.svelte";
  import { features } from "$lib/stores/features.svelte";
  import { saveStatus } from "$lib/stores/saveStatus.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { errorMessage } from "$lib/utils/errorMessage";
  import { keyboardShortcut } from "$lib/utils/keyboardShortcut";
  import { tagColor } from "$lib/utils/tagColor";
  import GeneratorPanel from "../generator/GeneratorPanel.svelte";
  import Button from "../ui/Button.svelte";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";
  import Dialog from "../ui/Dialog.svelte";
  import Icon from "../ui/Icon.svelte";
  import TotpSetupDialog from "../ui/TotpSetupDialog.svelte";
  import AttachmentList from "./AttachmentList.svelte";
  import CardDetailView from "./CardDetailView.svelte";
  import {
    type CardErrors,
    hasCardErrors,
    NO_CARD_ERRORS,
    validateCardFields,
  } from "./cardValidation";
  import DetailFooter from "./DetailFooter.svelte";
  import DetailHeader from "./DetailHeader.svelte";
  import EmptyDetail from "./EmptyDetail.svelte";
  import EntryEditForm from "./EntryEditForm.svelte";
  import ExpirationIndicator from "./ExpirationIndicator.svelte";
  import { cloneEditForm, createEditForm, type EditFormState } from "./editForm";
  import Field from "./Field.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import HistoryDialog from "./HistoryDialog.svelte";
  import IdentityDetailView from "./IdentityDetailView.svelte";
  import LoginDetailView from "./LoginDetailView.svelte";
  import MoveToFolderDialog from "./MoveToFolderDialog.svelte";
  import NotesField from "./NotesField.svelte";
  import PassportDetailView from "./PassportDetailView.svelte";
  import PasswordField from "./PasswordField.svelte";
  import PgpKeyDetailView from "./PgpKeyDetailView.svelte";
  import SoftwareLicenseDetailView from "./SoftwareLicenseDetailView.svelte";

  let _entry = $state<Entry | undefined>(undefined);
  let entryLoading = $state(false);
  let entryError = $state("");

  const KEYBOARD_DETAIL_DEBOUNCE_MS = 80;
  let fetchId = 0;
  let loadingTimer: ReturnType<typeof setTimeout> | undefined;

  function installUpdatedEntry(updated: Entry) {
    vault.setEntries(
      vault.entries.map((summary) => (summary.id === updated.id ? toSummary(updated) : summary)),
    );
    if (selection.selectedId === updated.id) _entry = updated;
  }

  async function refreshEntry(id: string) {
    const thisFetch = ++fetchId;
    try {
      const refreshed = await entriesBridge.entryGet(id);
      if (thisFetch === fetchId && selection.selectedId === id) {
        installUpdatedEntry(refreshed);
      }
    } catch (error) {
      if (thisFetch === fetchId && selection.selectedId === id) {
        console.error("Failed to refresh entry", error);
      }
    }
  }

  // Fetch the full entry whenever selection changes. Keyboard navigation uses
  // a short trailing debounce so rapidly skipped entries never reach the backend.
  $effect(() => {
    // A disk reload can update the selected entry without changing its UUID.
    // Depend on the vault revision so the detail projection is fetched again.
    void vault.revision;
    const id = selection.selectedId;
    const fetchMode = selection.detailFetchMode;
    if (loadingTimer) clearTimeout(loadingTimer);
    if (!id) {
      ++fetchId;
      _entry = undefined;
      entryLoading = false;
      entryError = "";
      return;
    }

    const thisFetch = ++fetchId;
    entryLoading = false;
    entryError = "";
    const startFetch = () => {
      // Keep previous entry visible during refetch; only show "Loading…" after
      // a short delay and only when we have no prior data to display.
      // untrack prevents `_entry` assignments from becoming effect dependencies.
      if (!untrack(() => _entry)) {
        loadingTimer = setTimeout(() => {
          if (thisFetch === fetchId) entryLoading = true;
        }, 120);
      }
      entriesBridge
        .entryGet(id)
        .then((e) => {
          if (thisFetch === fetchId) {
            if (loadingTimer) clearTimeout(loadingTimer);
            installUpdatedEntry(e);
            entryLoading = false;
          }
        })
        .catch((err) => {
          if (thisFetch === fetchId) {
            if (loadingTimer) clearTimeout(loadingTimer);
            entryError = errorMessage(err);
            entryLoading = false;
          }
        });
    };

    if (fetchMode === "keyboard") {
      const debounce = setTimeout(startFetch, KEYBOARD_DETAIL_DEBOUNCE_MS);
      return () => clearTimeout(debounce);
    }
    startFetch();
  });

  let editing = $state(false);
  let newEntryId = $state<string | null>(null);
  let showHistory = $state(false);
  let showMoveDialog = $state(false);
  let showGenerator = $state(false);
  let showTotpSetup = $state(false);
  let downloadingFavicon = $state(false);
  let form = $state(createEditForm());
  let cardErrors = $state<CardErrors>(NO_CARD_ERRORS);
  let pendingNavigation = $state<(() => void) | null>(null);

  let initialEditForm: EditFormState | null = null;

  function editFormsMatch(left: EditFormState, right: EditFormState) {
    return JSON.stringify(left) === JSON.stringify(right);
  }

  function buildEditPatch(): EntryPatch {
    const current = form;
    if (!initialEditForm) return current;

    const patch: EntryPatch = {};
    const writablePatch = patch as Record<string, unknown>;
    for (const key of Object.keys(current) as (keyof EditFormState)[]) {
      if (JSON.stringify(current[key]) === JSON.stringify(initialEditForm[key])) continue;
      writablePatch[key] =
        key === "customFields"
          ? current.customFields.map((field) => ({ ...field, name: field.name.trim() }))
          : current[key];
    }
    return patch;
  }

  function clearEditSecrets() {
    form.password = "";
    form.totp = "";
    form.cardNumber = "";
    form.cardCvv = "";
    form.cardPin = "";
    form.licenseKey = "";
    form.passportNumber = "";
    form.pgpPrivateKey = "";
    form.customFields = [];
    initialEditForm = null;
  }

  function hasUnsavedChanges() {
    return (
      newEntryId !== null || initialEditForm === null || !editFormsMatch(form, initialEditForm)
    );
  }

  function clearCardErrors() {
    cardErrors = NO_CARD_ERRORS;
  }

  /** Reset everything the edit session accumulated: secret buffers and
          card validation errors. */
  function resetEditState() {
    clearEditSecrets();
    clearCardErrors();
  }

  // Auto-enter edit mode when a new entry is created
  $effect(() => {
    if (_entry && vault.editingId === _entry.id) {
      if (vault.creatingId === _entry.id) newEntryId = _entry.id;
      vault.setCreatingId(null);
      vault.setEditingId(null);
      // Edit mode is entered only after the buffers (including revealed
      // secrets) are filled — saving a half-populated form would delete
      // the missing fields (`Some("")` clears a field on the backend).
      populateEdit()
        .then(() => (editing = true))
        .catch((e) => console.error("Failed to prepare edit form", e));
    }
  });

  // Auto-discard an unsaved new entry when the user navigates away from it
  // without saving. The stub lives only in the in-memory db; an unrelated
  // save later in the session (entry_update/entry_delete on another entry,
  // change_password, upgrade_kdf) would otherwise persist the whole db and
  // leak the stub to disk. Dropping it from memory here prevents that.
  $effect(() => {
    const selectedId = selection.selectedId;
    if (newEntryId && selectedId !== newEntryId) {
      const id = newEntryId;
      newEntryId = null;
      // Exit edit mode immediately so the unsaved entry's edit form
      // disappears while the new selection loads (or the pane goes empty).
      editing = false;
      resetEditState();
      if (selectedId !== _entry?.id) {
        // Clear the discarded stub from the pane until the next entry loads.
        _entry = undefined;
      }
      saveStatus.markSaved();
      void discardNewEntry(id);
    }
  });

  async function populateEdit() {
    if (!_entry) return;
    const e = _entry;
    // Fetch all protected edit values with one backend lock and entry lookup.
    // Entries without protected values can use their existing safe DTO directly.
    const needsSecretPayload =
      e.hasPassword ||
      e.hasTotp ||
      Boolean(e.card?.hasNumber || e.card?.hasCvv || e.card?.hasPin) ||
      Boolean(e.softwareLicense?.hasLicenseKey) ||
      Boolean(e.passport?.hasNumber) ||
      Boolean(e.pgpKey?.hasPrivateKey) ||
      e.customFields.some((field) => field.protected);
    const payload = needsSecretPayload ? await entriesBridge.entryEditPayload(e.id) : null;
    form = createEditForm(e, payload);
    initialEditForm = cloneEditForm(form);
    clearCardErrors();
  }

  async function toggleFavorite() {
    if (!_entry) return;
    saveStatus.markSaving();
    try {
      const updated = await entriesBridge.entryUpdate(_entry.id, {
        favorite: !_entry.favorite,
      });
      installUpdatedEntry(updated);
      saveStatus.markSaved();
    } catch (e) {
      const message = errorMessage(e);
      console.error("Failed to toggle favorite", e);
      saveStatus.markError(message);
      toast.error(message);
      // No edit session here, so an external-change check may reload at once.
      vault.refreshIfChanged().catch(() => {});
    }
  }

  async function downloadFavicon() {
    if (!_entry || downloadingFavicon) return;
    downloadingFavicon = true;
    try {
      const updated = await entriesBridge.entryDownloadFavicon(_entry.id);
      installUpdatedEntry(updated);
      toast.success("Favicon downloaded");
    } catch (e) {
      console.error("Failed to download favicon", e);
      toast.error(errorMessage(e));
    } finally {
      downloadingFavicon = false;
    }
  }

  /** Auto-download favicon silently (no toast, fire-and-forget). */
  function downloadFaviconSilent(entryId: string) {
    entriesBridge
      .entryDownloadFavicon(entryId)
      .then((withIcon) => {
        if (selection.selectedId === entryId) installUpdatedEntry(withIcon);
      })
      .catch((e) => {
        console.error("Failed to auto-download favicon", e);
      });
  }

  async function startEdit() {
    if (!_entry) return;
    newEntryId = null;
    try {
      await populateEdit();
    } catch (e) {
      // Don't enter edit mode with half-filled buffers: saving them would
      // delete the fields that failed to load.
      console.error("Failed to prepare edit form", e);
      return;
    }
    editing = true;
  }

  /** Drop a never-saved entry stub from the backend's in-memory database and
          the entry list. Returns false when the backend call fails — the stub
          then remains in memory and could persist on a later vault save. */
  async function discardNewEntry(id: string): Promise<boolean> {
    try {
      await entriesBridge.entryDiscard(id);
    } catch (e) {
      console.error("Failed to discard new entry", e);
      toast.error(errorMessage(e));
      return false;
    }
    vault.setEntries(vault.entries.filter((entry) => entry.id !== id));
    return true;
  }

  async function cancelEdit() {
    // Only discard when the entry on screen is the brand-new one we just
    // created. Tracking the id (not a boolean) prevents accidentally
    // discarding a real entry after the user navigated away mid-creation.
    if (newEntryId && _entry && _entry.id === newEntryId) {
      await discardNewEntry(newEntryId);
      _entry = undefined;
      selection.selectedId = null;
      newEntryId = null;
    }
    editing = false;
    saveError = "";
    resetEditState();
    saveStatus.markSaved();
  }

  let saveError = $state("");

  async function saveEdit(): Promise<boolean> {
    if (!_entry) return false;
    cardErrors = validateCardFields(form);
    if (hasCardErrors(cardErrors)) {
      saveStatus.markError("Fix validation errors before saving");
      return false;
    }
    saveError = "";
    const patch = buildEditPatch();
    if (newEntryId === null && Object.keys(patch).length === 0) {
      editing = false;
      resetEditState();
      saveStatus.markSaved();
      return true;
    }

    saveStatus.markSaving();
    try {
      const updated = await entriesBridge.entryUpdate(_entry.id, patch);
      installUpdatedEntry(updated);
      const isNewEntry = newEntryId !== null;
      editing = false;
      newEntryId = null;
      resetEditState();
      saveStatus.markSaved();
      // Auto-download favicon on new entry creation when URL is present
      if (isNewEntry && updated.url && !updated.hasCustomIcon) {
        downloadFaviconSilent(updated.id);
      }
      return true;
    } catch (e) {
      // Surface the failure (e.g. the vault file changed on disk) instead
      // of silently staying in edit mode.
      saveError = errorMessage(e);
      console.error("Failed to save", e);
      saveStatus.markError(saveError);
      // If an external vault change caused this, arm the store: the reload
      // then runs as soon as the edit ends (MainApp's deferred-reload effect),
      // and the inline "discard and reload" affordance appears.
      vault.refreshIfChanged().catch(() => {});
      return false;
    }
  }

  async function saveAndNavigate() {
    const navigate = pendingNavigation;
    if (!navigate) return;
    if (await saveEdit()) {
      pendingNavigation = null;
      navigate();
    } else {
      pendingNavigation = null;
    }
  }

  async function discardAndNavigate() {
    const navigate = pendingNavigation;
    if (!navigate) return;

    if (newEntryId && _entry?.id === newEntryId) {
      if (!(await discardNewEntry(newEntryId))) return;
      newEntryId = null;
    }

    editing = false;
    saveError = "";
    resetEditState();
    saveStatus.markSaved();
    pendingNavigation = null;
    navigate();
  }

  $effect(() => {
    vault.setEditSessionActive(editing);
    return () => vault.setEditSessionActive(false);
  });

  $effect(() => {
    if (!editing) return;
    if (hasUnsavedChanges()) saveStatus.markDirty();
    else saveStatus.markSaved();
  });

  $effect(() => {
    if (!editing) return;
    return selection.setNavigationGuard((navigate) => {
      if (pendingNavigation) return false;
      if (!hasUnsavedChanges()) {
        editing = false;
        clearEditSecrets();
        saveStatus.markSaved();
        return true;
      }
      pendingNavigation = navigate;
      return false;
    });
  });

  function openMoveDialog() {
    if (!_entry) return;
    showMoveDialog = true;
  }

  function confirmDelete() {
    if (!_entry) return;
    // The shared flow moves active entries to the bin and permanently removes
    // entries already in it. The callback clears local editing state.
    entryDeletion.request(_entry.id, _entry.title, () => {
      editing = false;
      newEntryId = null;
      clearEditSecrets();
      _entry = undefined;
    });
  }

  async function restoreEntry() {
    if (!_entry?.trashed) return;
    const id = _entry.id;
    try {
      await entriesBridge.entryRestore(id);
      vault.setEntries(
        vault.entries.map((entry) => (entry.id === id ? { ...entry, trashed: false } : entry)),
      );
      _entry = undefined;
      if (selection.selectedId === id) selection.selectedId = null;
      toast.success("Entry restored");
    } catch (e) {
      console.error("Failed to restore entry", e);
      toast.error(errorMessage(e));
    }
  }

  function requestCancelEdit() {
    if (!hasUnsavedChanges()) {
      void cancelEdit();
      return;
    }
    // Reuse the navigation confirmation flow without changing selection.
    // Saving or discarding exits edit mode; keeping edits leaves it open.
    pendingNavigation = () => {};
  }

  // Edit-mode shortcuts: Cmd/Ctrl+S saves, Esc requests cancellation. Skipped when a child dialog
  // (generator / delete-confirm / history) is open — those own Escape — and
  // when not editing. Bound at the window level so it works regardless of
  // where focus sits in the detail pane.
  function onEditKeydown(e: KeyboardEvent) {
    if (!editing) return;
    if (showGenerator || entryDeletion.pending || showHistory || pendingNavigation) return;

    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      if (hasUnsavedChanges()) void saveEdit();
    } else if (e.key === "Escape") {
      e.preventDefault();
      requestCancelEdit();
    }
  }
</script>

<svelte:window onkeydown={onEditKeydown} />

{#if showGenerator}
  <GeneratorPanel
    onUse={(pw) => { form.password = pw; showGenerator = false; }}
    oncancel={() => (showGenerator = false)}
  />
{/if}

{#if entryLoading && !_entry}
  <div class="detail-pane">
    <div class="empty-detail">
      <p>Loading…</p>
    </div>
  </div>
{:else if entryError}
  <div class="detail-pane">
    <div class="empty-detail">
      <p class="error-msg">{entryError}</p>
    </div>
  </div>
{:else if _entry}
  {@const entry = _entry}
  <div class="detail-pane">
    <div class="detail-toolbar">
      {#if entry.trashed}
        <button
          class="toolbar-btn toolbar-save"
          onclick={restoreEntry}
          aria-label="Restore"
          title="Restore"
        >
          <Icon name="restore" size={14} />
          <span>Restore</span>
        </button>
        <button
          class="toolbar-btn toolbar-delete"
          onclick={confirmDelete}
          aria-label="Delete permanently"
          title="Delete permanently"
        >
          <Icon name="trash-x" size={14} />
          <span>Delete permanently</span>
        </button>
      {:else if editing}
        <button
          class="toolbar-btn"
          onclick={requestCancelEdit}
          aria-label="Cancel"
          title="Cancel (Esc)"
        >
          <Icon name="x" size={14} />
          <span>Cancel</span>
        </button>
        <button
          class="toolbar-btn toolbar-save"
          onclick={saveEdit}
          aria-label="Save"
          title={`Save (${keyboardShortcut("S")})`}
          disabled={!hasUnsavedChanges()}
        >
          <Icon name="check" size={14} />
          <span>Save</span>
        </button>
        <button
          class="toolbar-btn toolbar-delete"
          onclick={confirmDelete}
          aria-label="Delete"
          title="Delete"
        >
          <Icon name="trash" size={14} />
          <span>Delete</span>
        </button>
      {/if}
    </div>

    {#if editing && saveError}
      <p class="save-error">
        {saveError}
        {#if vault.externalChangePending}
          <button class="save-error-action" onclick={cancelEdit}>
            Discard edit and reload latest
          </button>
        {/if}
      </p>
    {/if}

    {#if editing}
      <div class="edit-title">
        <input
          class="edit-title-input"
          type="text"
          placeholder="Title"
          autofocus
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
          bind:value={form.title}
        />
      </div>
    {:else}
      <DetailHeader
        {entry}
        onFavorite={toggleFavorite}
        onEdit={startEdit}
        onMove={openMoveDialog}
        showMove={features.foldersEnabled && !entry.trashed}
        onTotpSetup={() => (showTotpSetup = true)}
        showTotpSetup={entry.type === "login" && !entry.hasTotp}
        onDownloadFavicon={downloadFavicon}
        showDownloadFavicon={Boolean(entry.url)}
        {downloadingFavicon}
        readOnly={entry.trashed}
      />
    {/if}

    {#if editing}
      <EntryEditForm
        entryType={entry.type}
        {form}
        {cardErrors}
        onShowGenerator={() => (showGenerator = true)}
        onShowTotpSetup={() => (showTotpSetup = true)}
      />
    {:else if entry.type === "password" || entry.type === "login" || entry.type === "note"}
      <LoginDetailView {entry} />
    {/if}

    {#if !editing && entry.type === "identity"}
      <IdentityDetailView {entry} />
    {/if}

    {#if !editing && entry.type === "card"}
      <CardDetailView {entry} />
    {/if}

    {#if !editing && entry.type === "software_license"}
      <SoftwareLicenseDetailView {entry} />
    {/if}

    {#if !editing && entry.type === "passport"}
      <PassportDetailView {entry} />
    {/if}

    {#if !editing && entry.type === "pgp_key"}
      <PgpKeyDetailView {entry} />
    {/if}

    {#if !editing && entry.expiresAt}
      <ExpirationIndicator expiresAt={entry.expiresAt} />
    {/if}

    {#if !editing}
      {#if entry.tags.length > 0}
        <div class="tags-display">
          {#each entry.tags as tag}
            <span class="tag-badge" style={`--tag-color: ${tagColor(tag)}`}>{tag}</span>
          {/each}
        </div>
      {/if}
      {#if entry.notes}
        <NotesField notes={entry.notes} />
      {/if}
      {#if entry.customFields.length > 0}
        <FieldGroup>
          {#each entry.customFields as customField}
            {#if customField.protected}
              <PasswordField
                label={customField.name}
                reveal={() => entriesBridge.entryRevealCustomField(entry.id, customField.name)}
                copy={() => clipboard.copyCustomField(entry.id, customField.name)}
              />
            {:else}
              <Field
                label={customField.name}
                value={customField.value}
                onCopy={() => clipboard.copyPlain(customField.value)}
              />
            {/if}
          {/each}
        </FieldGroup>
      {/if}
    {/if}

    {#if !editing}
      <AttachmentList
        entryId={entry.id}
        attachments={entry.attachments}
        onchange={() => void refreshEntry(entry.id)}
      />
      <DetailFooter
        modifiedAt={entry.modifiedAt}
        historyCount={entry.historyCount}
        onclick={() => (showHistory = true)}
      />
    {/if}
  </div>
{:else}
  <EmptyDetail />
{/if}

{#if pendingNavigation}
  <ConfirmDialog
    title="Save changes?"
    message="You have unsaved changes. Save them before leaving this entry?"
    confirmLabel="Save"
    secondaryLabel="Discard"
    secondaryDanger={true}
    cancelLabel="Keep editing"
    onconfirm={saveAndNavigate}
    onsecondary={discardAndNavigate}
    oncancel={() => (pendingNavigation = null)}
  />
{/if}

{#if showHistory && _entry}
  <HistoryDialog entryId={_entry.id} onclose={() => (showHistory = false)} />
{/if}

{#if showMoveDialog && _entry}
  <MoveToFolderDialog
    entry={_entry}
    onclose={() => (showMoveDialog = false)}
    onmove={(updated) => {
      installUpdatedEntry(updated);
      showMoveDialog = false;
    }}
  />
{/if}

{#if showTotpSetup && _entry}
  <TotpSetupDialog
    oncancel={() => (showTotpSetup = false)}
    onconfirm={async (uri) => {
      showTotpSetup = false;
      if (!_entry) return;
      try {
        const updated = await entriesBridge.entryUpdate(_entry.id, { totp: uri });
        installUpdatedEntry(updated);
        if (editing && selection.selectedId === updated.id) form.totp = uri;
        toast.success("TOTP configured successfully");
      } catch (e) {
        toast.error(errorMessage(e));
      }
    }}
  />
{/if}

<style>
  .detail-pane {
    padding: 22px 24px;
    min-width: 0;
    min-height: 0;
    overflow-y: auto;
    background: var(--surface-2);
  }

  .detail-toolbar {
    display: flex;
    gap: 6px;
    margin-bottom: 16px;
  }

  .toolbar-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    background: var(--surface-1);
  }

  .toolbar-btn:hover:not(:disabled) {
    background: var(--border);
  }

  .toolbar-btn:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .toolbar-save {
    color: #fff;
    background: var(--accent);
    border-color: var(--accent);
  }

  .toolbar-save:hover:not(:disabled) {
    opacity: 0.9;
  }

  .toolbar-delete:hover {
    color: var(--danger);
    border-color: var(--danger);
  }

  .edit-title {
    margin-bottom: 20px;
  }

  .edit-title-input {
    font-size: 18px;
    font-weight: 500;
    padding: 6px 0;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border);
    width: 100%;
    color: var(--text-primary);
  }

  .edit-title-input:focus {
    border-bottom-color: var(--accent);
    outline: none;
  }

  .tags-display {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 16px;
  }

  .tag-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 8px;
    color: var(--tag-color);
    background: color-mix(in srgb, var(--tag-color) 12%, transparent);
    border: 0.5px solid color-mix(in srgb, var(--tag-color) 28%, transparent);
    border-radius: 4px;
    font-size: var(--text-sm);
  }

  .tag-badge::before {
    width: 6px;
    height: 6px;
    background: var(--tag-color);
    border-radius: 50%;
    content: "";
  }

  .error-msg {
    color: var(--danger);
    font-size: 13px;
  }

  .save-error {
    color: var(--danger);
    font-size: var(--text-sm);
    line-height: 1.4;
    margin-bottom: 12px;
  }

  .save-error-action {
    margin-left: 6px;
    color: var(--danger);
    font-size: inherit;
    text-decoration: underline;
  }

  .save-error-action:hover {
    color: var(--text-primary);
  }
</style>
