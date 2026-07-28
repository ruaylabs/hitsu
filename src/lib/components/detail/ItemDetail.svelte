<script lang="ts">
  import { untrack } from "svelte";
  import * as entriesBridge from "$lib/bridge/entries";
  import { type EntryPatch, toSummary } from "$lib/bridge/entries";
  import type { CustomField, Entry } from "$lib/bridge/types";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { entryDeletion } from "$lib/stores/entryDeletion.svelte";
  import { features } from "$lib/stores/features.svelte";
  import { saveStatus } from "$lib/stores/saveStatus.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { errorMessage } from "$lib/utils/errorMessage";
  import { cardBrandName, formatCardNumber } from "$lib/utils/format";
  import { openHttpUrl } from "$lib/utils/openHttpUrl";
  import { tagColor } from "$lib/utils/tagColor";
  import GeneratorPanel from "../generator/GeneratorPanel.svelte";
  import Button from "../ui/Button.svelte";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";
  import Dialog from "../ui/Dialog.svelte";
  import Icon from "../ui/Icon.svelte";
  import TotpSetupDialog from "../ui/TotpSetupDialog.svelte";
  import AttachmentList from "./AttachmentList.svelte";
  import EntryEditForm from "./EntryEditForm.svelte";
  import DetailFooter from "./DetailFooter.svelte";
  import DetailHeader from "./DetailHeader.svelte";
  import EmptyDetail from "./EmptyDetail.svelte";
  import Field from "./Field.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import HistoryDialog from "./HistoryDialog.svelte";
  import MoveToFolderDialog from "./MoveToFolderDialog.svelte";
  import NotesField from "./NotesField.svelte";
  import PasswordField from "./PasswordField.svelte";
  import TOTPField from "./TOTPField.svelte";

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
  let editTitle = $state("");
  let editUsername = $state("");
  let editPassword = $state("");
  let editUrl = $state("");
  let editTotp = $state("");
  let editNotes = $state("");
  let editExpiresAt = $state("");
  let editTags = $state<string[]>([]);
  let editCustomFields = $state<CustomField[]>([]);
  // Identity fields
  let editFirstName = $state("");
  let editLastName = $state("");
  let editEmail = $state("");
  let editPhone = $state("");
  let editAddress = $state("");
  let editDob = $state("");
  // Card fields
  let editCardHolder = $state("");
  let editCardNumber = $state("");
  let editCardType = $state("");
  let editCardExpMonth = $state("");
  let editCardExpYear = $state("");
  let editCardCvv = $state("");
  let editCardPin = $state("");
  // Software license fields
  let editLicenseVersion = $state("");
  let editLicenseKey = $state("");
  let editLicenseLicensedTo = $state("");
  let editLicenseRegisteredEmail = $state("");
  let editLicenseCompany = $state("");
  let editLicenseDownloadPage = $state("");
  let editLicensePublisher = $state("");
  let editLicenseWebsite = $state("");
  let editLicenseRetailPrice = $state("");
  let editLicenseSupportEmail = $state("");
  let editLicensePurchaseDate = $state("");
  let editLicenseOrderNumber = $state("");
  let editLicenseOrderTotal = $state("");
  // Passport fields
  let editPassportType = $state("");
  let editPassportIssuingCountry = $state("");
  let editPassportNumber = $state("");
  let editPassportFullName = $state("");
  let editPassportSex = $state("");
  let editPassportNationality = $state("");
  let editPassportIssuingAuthority = $state("");
  let editPassportBirthDate = $state("");
  let editPassportBirthPlace = $state("");
  let editPassportIssueDate = $state("");
  let editPassportExpiryDate = $state("");
  // PGP key fields
  let editPgpPublicKey = $state("");
  let editPgpPrivateKey = $state("");
  let editPgpFingerprint = $state("");
  let editPgpKeyId = $state("");
  let editPgpUserIds = $state("");
  let editPgpAlgorithm = $state("");
  let editPgpExpiresAt = $state("");

  // Validation errors for card fields
  let cardNumberError = $state("");
  let cardExpMonthError = $state("");
  let cardExpYearError = $state("");
  let cardCvvError = $state("");
  let cardPinError = $state("");
  let pendingNavigation = $state<(() => void) | null>(null);

  function captureEditForm() {
    return {
      title: editTitle,
      username: editUsername,
      password: editPassword,
      url: editUrl,
      totp: editTotp,
      notes: editNotes,
      expiresAt: editExpiresAt,
      tags: [...editTags],
      customFields: editCustomFields.map((field) => ({ ...field })),
      firstName: editFirstName,
      lastName: editLastName,
      email: editEmail,
      phone: editPhone,
      address: editAddress,
      dob: editDob,
      cardHolder: editCardHolder,
      cardNumber: editCardNumber,
      cardType: editCardType,
      cardExpMonth: editCardExpMonth,
      cardExpYear: editCardExpYear,
      cardCvv: editCardCvv,
      cardPin: editCardPin,
      licenseVersion: editLicenseVersion,
      licenseKey: editLicenseKey,
      licenseLicensedTo: editLicenseLicensedTo,
      licenseRegisteredEmail: editLicenseRegisteredEmail,
      licenseCompany: editLicenseCompany,
      licenseDownloadPage: editLicenseDownloadPage,
      licensePublisher: editLicensePublisher,
      licenseWebsite: editLicenseWebsite,
      licenseRetailPrice: editLicenseRetailPrice,
      licenseSupportEmail: editLicenseSupportEmail,
      licensePurchaseDate: editLicensePurchaseDate,
      licenseOrderNumber: editLicenseOrderNumber,
      licenseOrderTotal: editLicenseOrderTotal,
      passportType: editPassportType,
      passportIssuingCountry: editPassportIssuingCountry,
      passportNumber: editPassportNumber,
      passportFullName: editPassportFullName,
      passportSex: editPassportSex,
      passportNationality: editPassportNationality,
      passportIssuingAuthority: editPassportIssuingAuthority,
      passportBirthDate: editPassportBirthDate,
      passportBirthPlace: editPassportBirthPlace,
      passportIssueDate: editPassportIssueDate,
      passportExpiryDate: editPassportExpiryDate,
      pgpPublicKey: editPgpPublicKey,
      pgpPrivateKey: editPgpPrivateKey,
      pgpFingerprint: editPgpFingerprint,
      pgpKeyId: editPgpKeyId,
      pgpUserIds: editPgpUserIds,
      pgpAlgorithm: editPgpAlgorithm,
      pgpExpiresAt: editPgpExpiresAt,
    };
  }

  type EditForm = ReturnType<typeof captureEditForm>;
  let initialEditForm: EditForm | null = null;

  function editFormsMatch(left: EditForm, right: EditForm) {
    return JSON.stringify(left) === JSON.stringify(right);
  }

  function buildEditPatch(): EntryPatch {
    const current = captureEditForm();
    if (!initialEditForm) return current;

    const patch: EntryPatch = {};
    const writablePatch = patch as Record<string, unknown>;
    for (const key of Object.keys(current) as (keyof EditForm)[]) {
      if (JSON.stringify(current[key]) === JSON.stringify(initialEditForm[key])) continue;
      writablePatch[key] =
        key === "customFields"
          ? current.customFields.map((field) => ({ ...field, name: field.name.trim() }))
          : current[key];
    }
    return patch;
  }

  function clearEditSecrets() {
    editPassword = "";
    editTotp = "";
    editCardNumber = "";
    editCardCvv = "";
    editCardPin = "";
    editLicenseKey = "";
    editPassportNumber = "";
    editPgpPrivateKey = "";
    editCustomFields = [];
    initialEditForm = null;
  }

  function hasUnsavedChanges() {
    return (
      newEntryId !== null ||
      initialEditForm === null ||
      !editFormsMatch(captureEditForm(), initialEditForm)
    );
  }

  function clearCardErrors() {
    cardNumberError = "";
    cardExpMonthError = "";
    cardExpYearError = "";
    cardCvvError = "";
    cardPinError = "";
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
      clearEditSecrets();
      if (selectedId !== _entry?.id) {
        // Clear the discarded stub from the pane until the next entry loads.
        _entry = undefined;
      }
      clearCardErrors();
      saveStatus.markSaved();
      entriesBridge.entryDiscard(id).catch((e) => console.error("Failed to discard new entry", e));
      untrack(() => {
        vault.setEntries(vault.entries.filter((s) => s.id !== id));
      });
    }
  });

  async function populateEdit() {
    if (!_entry) return;
    const e = _entry;
    editTitle = e.title;
    editUsername = e.username ?? "";
    editUrl = e.url ?? "";
    editTags = [...e.tags];
    editNotes = e.notes ?? "";
    editExpiresAt = e.expiresAt ?? "";
    editFirstName = e.identity?.firstName ?? "";
    editLastName = e.identity?.lastName ?? "";
    editEmail = e.identity?.email ?? "";
    editPhone = e.identity?.phone ?? "";
    editAddress = e.identity?.address ?? "";
    editDob = e.identity?.dob ?? "";
    editCardHolder = e.card?.holder ?? "";
    editCardType = e.card?.type ?? "";
    editCardExpMonth = e.card?.expMonth?.toString() ?? "";
    editCardExpYear = e.card?.expYear?.toString() ?? "";
    editLicenseVersion = e.softwareLicense?.version ?? "";
    editLicenseLicensedTo = e.softwareLicense?.licensedTo ?? "";
    editLicenseRegisteredEmail = e.softwareLicense?.registeredEmail ?? "";
    editLicenseCompany = e.softwareLicense?.company ?? "";
    editLicenseDownloadPage = e.softwareLicense?.downloadPage ?? "";
    editLicensePublisher = e.softwareLicense?.publisher ?? "";
    editLicenseWebsite = e.softwareLicense?.website ?? "";
    editLicenseRetailPrice = e.softwareLicense?.retailPrice ?? "";
    editLicenseSupportEmail = e.softwareLicense?.supportEmail ?? "";
    editLicensePurchaseDate = e.softwareLicense?.purchaseDate ?? "";
    editLicenseOrderNumber = e.softwareLicense?.orderNumber ?? "";
    editLicenseOrderTotal = e.softwareLicense?.orderTotal ?? "";
    editPassportType = e.passport?.type ?? "";
    editPassportIssuingCountry = e.passport?.issuingCountry ?? "";
    editPassportFullName = e.passport?.fullName ?? "";
    editPassportSex = e.passport?.sex ?? "";
    editPassportNationality = e.passport?.nationality ?? "";
    editPassportIssuingAuthority = e.passport?.issuingAuthority ?? "";
    editPassportBirthDate = e.passport?.birthDate ?? "";
    editPassportBirthPlace = e.passport?.birthPlace ?? "";
    editPassportIssueDate = e.passport?.issueDate ?? "";
    editPassportExpiryDate = e.passport?.expiryDate ?? "";
    editPgpPublicKey = e.pgpKey?.publicKey ?? "";
    editPgpFingerprint = e.pgpKey?.fingerprint ?? "";
    editPgpKeyId = e.pgpKey?.keyId ?? "";
    editPgpUserIds = e.pgpKey?.userIds ?? "";
    editPgpAlgorithm = e.pgpKey?.algorithm ?? "";
    editPgpExpiresAt = e.pgpKey?.expiresAt ?? "";
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
    editPassword = payload?.password ?? "";
    editTotp = payload?.totp ?? "";
    editCardNumber = payload?.cardNumber ?? "";
    editCardCvv = payload?.cardCvv ?? "";
    editCardPin = payload?.cardPin ?? "";
    editLicenseKey = payload?.licenseKey ?? "";
    editPassportNumber = payload?.passportNumber ?? "";
    editPgpPrivateKey = payload?.pgpPrivateKey ?? "";
    editCustomFields = (payload?.customFields ?? e.customFields).map((field) => ({ ...field }));
    initialEditForm = captureEditForm();
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

  async function cancelEdit() {
    // Only discard when the entry on screen is the brand-new one we just
    // created. Tracking the id (not a boolean) prevents accidentally
    // discarding a real entry after the user navigated away mid-creation.
    if (newEntryId && _entry && _entry.id === newEntryId) {
      const id = _entry.id;
      try {
        await entriesBridge.entryDiscard(id);
      } catch (e) {
        console.error("Failed to discard new entry", e);
      }
      vault.setEntries(vault.entries.filter((s) => s.id !== id));
      _entry = undefined;
      selection.selectedId = null;
      newEntryId = null;
    }
    editing = false;
    saveError = "";
    clearEditSecrets();
    clearCardErrors();
    saveStatus.markSaved();
  }

  function validateCardFields(): boolean {
    let valid = true;
    // Card number: digits only, 13-19 chars (standard card lengths)
    if (editCardNumber && editCardNumber.length > 0 && editCardNumber.length < 13) {
      cardNumberError = "Card number too short";
      valid = false;
    } else {
      cardNumberError = "";
    }
    // Exp month: 2 digits, 01-12
    if (editCardExpMonth && editCardExpMonth.length !== 2) {
      cardExpMonthError = "Must be 2 digits (01-12)";
      valid = false;
    } else if (editCardExpMonth) {
      const m = Number.parseInt(editCardExpMonth, 10);
      if (m < 1 || m > 12) {
        cardExpMonthError = "Must be 01-12";
        valid = false;
      } else {
        cardExpMonthError = "";
      }
    } else {
      cardExpMonthError = "";
    }
    // Exp year: 4 digits
    if (editCardExpYear && editCardExpYear.length !== 4) {
      cardExpYearError = "Year must be 4 digits";
      valid = false;
    } else {
      cardExpYearError = "";
    }
    // CVV: 3 or 4 digits
    if (editCardCvv && editCardCvv.length !== 3 && editCardCvv.length !== 4) {
      cardCvvError = "CVV must be 3 or 4 digits";
      valid = false;
    } else {
      cardCvvError = "";
    }
    // PIN: 4-12 digits (ISO 9564 range)
    if (editCardPin && (editCardPin.length < 4 || editCardPin.length > 12)) {
      cardPinError = "PIN must be 4-12 digits";
      valid = false;
    } else {
      cardPinError = "";
    }
    return valid;
  }

  let saveError = $state("");

  async function saveEdit(): Promise<boolean> {
    if (!_entry) return false;
    if (!validateCardFields()) {
      saveStatus.markError("Fix validation errors before saving");
      return false;
    }
    saveError = "";
    const patch = buildEditPatch();
    if (newEntryId === null && Object.keys(patch).length === 0) {
      editing = false;
      clearEditSecrets();
      clearCardErrors();
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
      clearEditSecrets();
      clearCardErrors();
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
      const id = newEntryId;
      try {
        await entriesBridge.entryDiscard(id);
      } catch (e) {
        toast.error(errorMessage(e));
        return;
      }
      vault.setEntries(vault.entries.filter((entry) => entry.id !== id));
      newEntryId = null;
    }

    editing = false;
    saveError = "";
    clearEditSecrets();
    clearCardErrors();
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

  function localDateString() {
    const now = new Date();
    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, "0");
    const day = String(now.getDate()).padStart(2, "0");
    return `${year}-${month}-${day}`;
  }

  function expirationLabel(expiresAt: string) {
    const formatted = new Date(`${expiresAt}T00:00:00`).toLocaleDateString();
    if (expiresAt < localDateString()) return `Expired on ${formatted}`;
    if (expiresAt === localDateString()) return "Expires today";
    return `Expires on ${formatted}`;
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

  // Edit-mode shortcuts: ⌘S saves, Esc requests cancellation. Skipped when a child dialog
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
    onUse={(pw) => { editPassword = pw; showGenerator = false; }}
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
        <button class="toolbar-btn" onclick={cancelEdit} aria-label="Cancel" title="Cancel (Esc)">
          <Icon name="x" size={14} />
          <span>Cancel</span>
        </button>
        <button
          class="toolbar-btn toolbar-save"
          onclick={saveEdit}
          aria-label="Save"
          title="Save (⌘S)"
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
          bind:value={editTitle}
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
        bind:editUsername
        bind:editPassword
        bind:editUrl
        bind:editTotp
        bind:editFirstName
        bind:editLastName
        bind:editEmail
        bind:editPhone
        bind:editAddress
        bind:editDob
        bind:editCardHolder
        bind:editCardNumber
        bind:editCardType
        bind:editCardExpMonth
        bind:editCardExpYear
        bind:editCardCvv
        bind:editCardPin
        {cardNumberError}
        {cardExpMonthError}
        {cardExpYearError}
        {cardCvvError}
        {cardPinError}
        bind:editLicenseVersion
        bind:editLicenseKey
        bind:editLicenseLicensedTo
        bind:editLicenseRegisteredEmail
        bind:editLicenseCompany
        bind:editLicenseDownloadPage
        bind:editLicensePublisher
        bind:editLicenseWebsite
        bind:editLicenseRetailPrice
        bind:editLicenseSupportEmail
        bind:editLicensePurchaseDate
        bind:editLicenseOrderNumber
        bind:editLicenseOrderTotal
        bind:editPassportType
        bind:editPassportIssuingCountry
        bind:editPassportNumber
        bind:editPassportFullName
        bind:editPassportSex
        bind:editPassportNationality
        bind:editPassportIssuingAuthority
        bind:editPassportBirthDate
        bind:editPassportBirthPlace
        bind:editPassportIssueDate
        bind:editPassportExpiryDate
        bind:editPgpPublicKey
        bind:editPgpPrivateKey
        bind:editPgpFingerprint
        bind:editPgpKeyId
        bind:editPgpUserIds
        bind:editPgpAlgorithm
        bind:editPgpExpiresAt
        bind:editExpiresAt
        bind:editCustomFields
        bind:editTags
        bind:editNotes
        onShowGenerator={() => (showGenerator = true)}
        onShowTotpSetup={() => (showTotpSetup = true)}
      />
    {:else if entry.type === "password"}
      {#if entry.hasPassword}
        <FieldGroup>
          <PasswordField
            label="Password"
            reveal={() => entriesBridge.entryRevealField(entry.id, "password")}
            copy={() => clipboard.copySecretField(entry.id, "password")}
            showStrength
          />
          {#if entry.url}
            <Field
              label="URL"
              value={entry.url}
              onOpenUrl={() => openHttpUrl(entry.url!)}
              onCopy={() => clipboard.copyPlain(entry.url!)}
            />
          {/if}
        </FieldGroup>
      {/if}
    {:else if entry.type === "login" || entry.type === "note"}
      {#if entry.type === "login" && entry.hasTotp}
        <TOTPField entryId={entry.id} />
      {/if}
      <FieldGroup>
        {#if entry.username}
          <Field
            label="Username"
            value={entry.username}
            onCopy={() => clipboard.copyPlain(entry.username!)}
          />
        {/if}
        {#if entry.hasPassword}
          <PasswordField
            label="Password"
            reveal={() => entriesBridge.entryRevealField(entry.id, "password")}
            copy={() => clipboard.copySecretField(entry.id, "password")}
            showStrength
          />
        {/if}
        {#if entry.url}
          <Field
            label="URL"
            value={entry.url}
            mono={false}
            onOpenUrl={() => openHttpUrl(entry.url!)}
            onCopy={() => clipboard.copyPlain(entry.url!)}
          />
        {/if}
      </FieldGroup>
    {/if}

    {#if !editing && entry.type === "identity" && entry.identity}
      <FieldGroup>
        {#if entry.identity.firstName}
          <Field label="First name" value={entry.identity.firstName} />
        {/if}
        {#if entry.identity.lastName}
          <Field label="Last name" value={entry.identity.lastName} />
        {/if}
        {#if entry.identity.email}
          <Field
            label="Email"
            value={entry.identity.email}
            onCopy={() => clipboard.copyPlain(entry.identity!.email!)}
          />
        {/if}
        {#if entry.identity.phone}
          <Field
            label="Phone"
            value={entry.identity.phone}
            onCopy={() => clipboard.copyPlain(entry.identity!.phone!)}
          />
        {/if}
        {#if entry.identity.address}
          <Field label="Address" value={entry.identity.address} />
        {/if}
        {#if entry.identity.dob}
          <Field label="Date of birth" value={entry.identity.dob} />
        {/if}
      </FieldGroup>
    {/if}

    {#if !editing && entry.type === "card" && entry.card}
      <FieldGroup>
        {#if entry.card.type}
          <Field label="Type" value={cardBrandName(entry.card.type)} />
        {/if}
        {#if entry.card.holder}
          <Field label="Holder" value={entry.card.holder} />
        {/if}
        {#if entry.card.hasNumber}
          <!-- Masked by default like the other secrets: the full PAN only
               crosses IPC on an explicit reveal, never on selection. -->
          <PasswordField
            label="Number"
            masked={entry.card.numberMasked || undefined}
            reveal={async () =>
              formatCardNumber(
                await entriesBridge.entryRevealField(entry.id, "cardNumber"),
                entry.card?.type,
              )}
            copy={() => clipboard.copySecretField(entry.id, "cardNumber")}
          />
        {/if}
        {#if entry.card.expMonth && entry.card.expYear}
          <Field
            label="Expires"
            value={`${String(entry.card.expMonth).padStart(2, "0")}/${entry.card.expYear}`}
          />
        {/if}
        {#if entry.card.hasCvv}
          <PasswordField
            label="CVV"
            reveal={() => entriesBridge.entryRevealField(entry.id, "cardCvv")}
            copy={() => clipboard.copySecretField(entry.id, "cardCvv")}
          />
        {/if}
        {#if entry.card.hasPin}
          <PasswordField
            label="PIN"
            reveal={() => entriesBridge.entryRevealField(entry.id, "cardPin")}
            copy={() => clipboard.copySecretField(entry.id, "cardPin")}
          />
        {/if}
      </FieldGroup>
    {/if}

    {#if !editing && entry.type === "software_license" && entry.softwareLicense}
      {@const license = entry.softwareLicense}
      <FieldGroup>
        {#if license.version}
          <Field label="Version" value={license.version} />
        {/if}
        {#if license.hasLicenseKey}
          <PasswordField
            label="License key"
            reveal={() => entriesBridge.entryRevealField(entry.id, "licenseKey")}
            copy={() => clipboard.copySecretField(entry.id, "licenseKey")}
          />
        {/if}
      </FieldGroup>
      <FieldGroup>
        {#if license.licensedTo}
          <Field label="Licensed to" value={license.licensedTo} />
        {/if}
        {#if license.registeredEmail}
          <Field
            label="Registered email"
            value={license.registeredEmail}
            onCopy={() => clipboard.copyPlain(license.registeredEmail!)}
          />
        {/if}
        {#if license.company}
          <Field label="Company" value={license.company} />
        {/if}
      </FieldGroup>
      <FieldGroup>
        {#if license.downloadPage}
          <Field
            label="Download page"
            value={license.downloadPage}
            onOpenUrl={() => openHttpUrl(license.downloadPage!)}
            onCopy={() => clipboard.copyPlain(license.downloadPage!)}
          />
        {/if}
        {#if license.publisher}
          <Field label="Publisher" value={license.publisher} />
        {/if}
        {#if license.website}
          <Field
            label="Website"
            value={license.website}
            onOpenUrl={() => openHttpUrl(license.website!)}
            onCopy={() => clipboard.copyPlain(license.website!)}
          />
        {/if}
        {#if license.retailPrice}
          <Field label="Retail price" value={license.retailPrice} />
        {/if}
        {#if license.supportEmail}
          <Field
            label="Support email"
            value={license.supportEmail}
            onCopy={() => clipboard.copyPlain(license.supportEmail!)}
          />
        {/if}
      </FieldGroup>
      <FieldGroup>
        {#if license.purchaseDate}
          <Field label="Purchase date" value={license.purchaseDate} />
        {/if}
        {#if license.orderNumber}
          <Field label="Order number" value={license.orderNumber} />
        {/if}
        {#if license.orderTotal}
          <Field label="Order total" value={license.orderTotal} />
        {/if}
      </FieldGroup>
    {/if}

    {#if !editing && entry.type === "passport" && entry.passport}
      {@const passport = entry.passport}
      <FieldGroup>
        {#if passport.type}
          <Field label="Type" value={passport.type} />
        {/if}
        {#if passport.issuingCountry}
          <Field label="Issuing country" value={passport.issuingCountry} />
        {/if}
        {#if passport.hasNumber}
          <PasswordField
            label="Number"
            reveal={() => entriesBridge.entryRevealField(entry.id, "passportNumber")}
            copy={() => clipboard.copySecretField(entry.id, "passportNumber")}
          />
        {/if}
        {#if passport.fullName}
          <Field label="Full name" value={passport.fullName} />
        {/if}
        {#if passport.sex}
          <Field label="Sex" value={passport.sex} />
        {/if}
        {#if passport.nationality}
          <Field label="Nationality" value={passport.nationality} />
        {/if}
        {#if passport.issuingAuthority}
          <Field label="Issuing authority" value={passport.issuingAuthority} />
        {/if}
      </FieldGroup>
      <FieldGroup>
        {#if passport.birthDate}
          <Field label="Date of birth" value={passport.birthDate} />
        {/if}
        {#if passport.birthPlace}
          <Field label="Place of birth" value={passport.birthPlace} />
        {/if}
        {#if passport.issueDate}
          <Field label="Issued on" value={passport.issueDate} />
        {/if}
        {#if passport.expiryDate}
          <Field label="Expiry date" value={passport.expiryDate} />
        {/if}
      </FieldGroup>
    {/if}

    {#if !editing && entry.type === "pgp_key" && entry.pgpKey}
      {@const pgp = entry.pgpKey}
      <FieldGroup>
        {#if pgp.fingerprint}
          <Field
            label="Fingerprint"
            value={pgp.fingerprint}
            mono={true}
            onCopy={() => clipboard.copyPlain(pgp.fingerprint!)}
          />
        {/if}
        {#if pgp.keyId}
          <Field
            label="Key ID"
            value={pgp.keyId}
            mono={true}
            onCopy={() => clipboard.copyPlain(pgp.keyId!)}
          />
        {/if}
        {#if pgp.userIds}
          <Field label="User IDs" value={pgp.userIds} />
        {/if}
        {#if pgp.algorithm}
          <Field label="Algorithm" value={pgp.algorithm} />
        {/if}
        {#if pgp.expiresAt}
          <Field label="Expires" value={pgp.expiresAt} />
        {/if}
      </FieldGroup>
      {#if pgp.publicKey}
        <FieldGroup>
          <Field
            label="Public key"
            value={pgp.publicKey}
            mono={true}
            onCopy={() => clipboard.copyPlain(pgp.publicKey!)}
          />
        </FieldGroup>
      {/if}
      {#if pgp.hasPrivateKey}
        <FieldGroup>
          <PasswordField
            label="Private key"
            reveal={() => entriesBridge.entryRevealField(entry.id, "pgpPrivateKey")}
            copy={() => clipboard.copySecretField(entry.id, "pgpPrivateKey")}
          />
        </FieldGroup>
      {/if}
    {/if}

    {#if !editing && entry.expiresAt}
      {@const expirationDue = entry.expiresAt <= localDateString()}
      <div class="expiration-indicator" class:due={expirationDue} role="status">
        <Icon name={expirationDue ? "alert-triangle" : "calendar-time"} size={14} />
        <span>{expirationLabel(entry.expiresAt)}</span>
      </div>
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
        if (editing && selection.selectedId === updated.id) editTotp = uri;
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
    font-size: 12px;
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

  .expiration-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    width: fit-content;
    margin-bottom: 16px;
    padding: 5px 9px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    background: var(--surface-1);
    font-size: 12px;
  }

  .expiration-indicator.due {
    color: var(--danger);
    background: var(--danger-bg);
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
    font-size: 11.5px;
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
    font-size: 12px;
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
