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
  import { CARD_BRANDS, cardBrandName, formatCardNumber } from "$lib/utils/format";
  import { openHttpUrl } from "$lib/utils/openHttpUrl";
  import GeneratorPanel from "../generator/GeneratorPanel.svelte";
  import Button from "../ui/Button.svelte";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";
  import Dialog from "../ui/Dialog.svelte";
  import Icon from "../ui/Icon.svelte";
  import PasswordStrengthMeter from "../ui/PasswordStrengthMeter.svelte";
  import TagInput from "../ui/TagInput.svelte";
  import TotpSetupDialog from "../ui/TotpSetupDialog.svelte";
  import AttachmentList from "./AttachmentList.svelte";
  import DetailFieldRow from "./DetailFieldRow.svelte";
  import DetailFooter from "./DetailFooter.svelte";
  import DetailHeader from "./DetailHeader.svelte";
  import EmptyDetail from "./EmptyDetail.svelte";
  import Field from "./Field.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import HistoryDialog from "./HistoryDialog.svelte";
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
            entryError = err instanceof Error ? err.message : String(err);
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

  // Card numbers are shown in full by default (unlike password/CVV/PIN),
  // so fetch the plaintext when a card entry is selected. Falls back to the
  // backend-masked value until it arrives (or if the fetch fails).
  let cardNumberPlain = $state("");
  let cardNumberRevision = -1;
  // Which entry id the fetched (or in-flight) number belongs to. Keyed on the
  // id, not the `_entry` object: `_entry` is reassigned on refetches of the
  // same entry (favorite toggle, save), and resetting/refetching then would
  // flash the field back to the masked value.
  let cardNumberEntryId: string | null = null;
  $effect(() => {
    const e = _entry;
    const revision = vault.revision;
    const wantId = e?.type === "card" && e.card?.hasNumber ? e.id : null;
    if (wantId === cardNumberEntryId && revision === cardNumberRevision) return;
    cardNumberEntryId = wantId;
    cardNumberRevision = revision;
    cardNumberPlain = "";
    if (wantId) {
      entriesBridge
        .entryRevealField(wantId, "cardNumber")
        .then((n) => {
          if (cardNumberEntryId === wantId && cardNumberRevision === revision) cardNumberPlain = n;
        })
        .catch((err) => console.error("Failed to load card number", err));
    }
  });

  let editing = $state(false);
  let newEntryId = $state<string | null>(null);
  let showHistory = $state(false);
  let showMoveDialog = $state(false);
  let moveFolderId = $state("");
  let movingEntry = $state(false);
  let moveError = $state("");
  let addingMoveFolder = $state(false);
  let newMoveFolderName = $state("");
  let creatingMoveFolder = $state(false);
  let newMoveFolderError = $state("");
  let showGenerator = $state(false);
  let showTotpSetup = $state(false);
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
    // Fetch all protected edit values with one backend lock and entry lookup.
    // Entries without protected values can use their existing safe DTO directly.
    const needsSecretPayload =
      e.hasPassword ||
      e.hasTotp ||
      Boolean(e.card?.hasNumber || e.card?.hasCvv || e.card?.hasPin) ||
      Boolean(e.softwareLicense?.hasLicenseKey) ||
      Boolean(e.passport?.hasNumber) ||
      e.customFields.some((field) => field.protected);
    const payload = needsSecretPayload ? await entriesBridge.entryEditPayload(e.id) : null;
    editPassword = payload?.password ?? "";
    editTotp = payload?.totp ?? "";
    editCardNumber = payload?.cardNumber ?? "";
    editCardCvv = payload?.cardCvv ?? "";
    editCardPin = payload?.cardPin ?? "";
    editLicenseKey = payload?.licenseKey ?? "";
    editPassportNumber = payload?.passportNumber ?? "";
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
      const message = e instanceof Error ? e.message : String(e);
      console.error("Failed to toggle favorite", e);
      saveStatus.markError(message);
      toast.error(message);
      // No edit session here, so an external-change check may reload at once.
      vault.refreshIfChanged().catch(() => {});
    }
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
      editing = false;
      newEntryId = null;
      clearEditSecrets();
      clearCardErrors();
      saveStatus.markSaved();
      return true;
    } catch (e) {
      // Surface the failure (e.g. the vault file changed on disk) instead
      // of silently staying in edit mode.
      saveError = e instanceof Error ? e.message : String(e);
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
        toast.error(e instanceof Error ? e.message : String(e));
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

  function folderPath(folderId: string) {
    const names: string[] = [];
    const seen = new Set<string>();
    let current = vault.folders.find((folder) => folder.id === folderId);
    while (current && !seen.has(current.id)) {
      seen.add(current.id);
      names.unshift(current.name);
      current = current.parentId
        ? vault.folders.find((folder) => folder.id === current?.parentId)
        : undefined;
    }
    return names.join(" / ");
  }

  let moveFolders = $derived(
    [...vault.folders].sort((left, right) =>
      folderPath(left.id).localeCompare(folderPath(right.id)),
    ),
  );

  function openMoveDialog() {
    if (!_entry) return;
    moveFolderId = _entry.folderId ?? "";
    moveError = "";
    addingMoveFolder = false;
    newMoveFolderName = "";
    newMoveFolderError = "";
    showMoveDialog = true;
  }

  async function createMoveFolder() {
    const name = newMoveFolderName.trim();
    if (!name || creatingMoveFolder) return;
    creatingMoveFolder = true;
    newMoveFolderError = "";
    try {
      const folder = await vault.createFolder(moveFolderId || null, name);
      moveFolderId = folder.id;
      addingMoveFolder = false;
      newMoveFolderName = "";
    } catch (error) {
      newMoveFolderError = error instanceof Error ? error.message : String(error);
    } finally {
      creatingMoveFolder = false;
    }
  }

  async function moveEntry() {
    if (!_entry || movingEntry) return;
    movingEntry = true;
    moveError = "";
    try {
      const updated = await entriesBridge.entryMove(_entry.id, moveFolderId || null);
      installUpdatedEntry(updated);
      showMoveDialog = false;
      toast.success("Entry moved");
    } catch (error) {
      moveError = error instanceof Error ? error.message : String(error);
    } finally {
      movingEntry = false;
    }
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
      toast.error(e instanceof Error ? e.message : String(e));
    }
  }

  // Edit-mode shortcuts: ⌘S saves, Esc cancels. Skipped when a child dialog
  // (generator / delete-confirm / history) is open — those own Escape — and
  // when not editing. Bound at the window level so it works regardless of
  // where focus sits in the detail pane.
  function onEditKeydown(e: KeyboardEvent) {
    if (!editing) return;
    if (showGenerator || entryDeletion.pending || showHistory || pendingNavigation) return;

    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      saveEdit();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelEdit();
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
        readOnly={entry.trashed}
      />
    {/if}

    {#if editing}
      <!-- Edit mode: type-specific fields -->
      <FieldGroup>
        {#if entry.type === "login"}
          <DetailFieldRow label="Username">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Username"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editUsername}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Password">
            <div class="password-edit-col">
              <div class="password-edit-row">
                <input
                  class="control control--compact edit-input"
                  type="text"
                  placeholder="Password"
                  autocomplete="off"
                  autocorrect="off"
                  autocapitalize="off"
                  spellcheck="false"
                  bind:value={editPassword}
                />
                <button
                  class="generate-btn"
                  onclick={() => (showGenerator = true)}
                  aria-label="Generate password"
                  title="Generate password"
                >
                  <Icon name="bolt" size={14} />
                </button>
              </div>
              <PasswordStrengthMeter password={editPassword} showWhenEmpty />
            </div>
          </DetailFieldRow>
          <DetailFieldRow label="URL">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="URL"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editUrl}
            />
          </DetailFieldRow>
          <DetailFieldRow label="TOTP">
            <div class="totp-edit-wrap">
              <input
                class="control control--compact edit-input"
                type="text"
                placeholder="otpauth:// URI"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                bind:value={editTotp}
              />
              <button
                class="totp-setup-btn-small"
                onclick={() => (showTotpSetup = true)}
                aria-label="Setup TOTP from seed"
                title="Setup TOTP from seed"
              >
                <Icon name="key" size={13} />
              </button>
            </div>
          </DetailFieldRow>
        {:else if entry.type === "password"}
          <DetailFieldRow label="Password">
            <div class="password-edit-col">
              <div class="password-edit-row">
                <input
                  class="control control--compact edit-input"
                  type="text"
                  placeholder="Password"
                  autocomplete="off"
                  autocorrect="off"
                  autocapitalize="off"
                  spellcheck="false"
                  bind:value={editPassword}
                />
                <button
                  class="generate-btn"
                  onclick={() => (showGenerator = true)}
                  aria-label="Generate password"
                  title="Generate password"
                >
                  <Icon name="bolt" size={14} />
                </button>
              </div>
              <PasswordStrengthMeter password={editPassword} showWhenEmpty />
            </div>
          </DetailFieldRow>
          <DetailFieldRow label="URL">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="URL"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editUrl}
            />
          </DetailFieldRow>
        {:else if entry.type === "identity"}
          <DetailFieldRow label="First name">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="First name"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editFirstName}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Last name">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Last name"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editLastName}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Email">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Email"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editEmail}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Phone">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Phone"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editPhone}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Address">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Address"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editAddress}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Date of birth">
            <input
              class="control control--compact edit-input"
              type="date"
              aria-label="Date of birth"
              autocomplete="bday"
              bind:value={editDob}
            />
          </DetailFieldRow>
        {:else if entry.type === "card"}
          <DetailFieldRow label="Holder">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Card holder"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editCardHolder}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Number" alignStart>
            <div class="card-input-wrap">
              <input
                class="control control--compact edit-input"
                type="text"
                inputmode="numeric"
                pattern="[0-9]*"
                aria-invalid={Boolean(cardNumberError)}
                placeholder="Card number"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                bind:value={editCardNumber}
                oninput={(e) => { const el = e.currentTarget; el.value = el.value.replace(/\D/g, ''); editCardNumber = el.value; }}
              />
              {#if cardNumberError}
                <span class="control-error">{cardNumberError}</span>
              {/if}
            </div>
          </DetailFieldRow>
          <DetailFieldRow label="Type">
            <select class="control control--compact control--select" bind:value={editCardType}>
              <option value="">Select brand</option>
              {#each Object.entries(CARD_BRANDS) as [ key, name ]}
                <option value={key}>{name}</option>
              {/each}
            </select>
          </DetailFieldRow>
          <DetailFieldRow label="Exp month" alignStart>
            <div class="card-input-wrap">
              <input
                class="control control--compact edit-input"
                type="text"
                inputmode="numeric"
                pattern="[0-9]*"
                aria-invalid={Boolean(cardExpMonthError)}
                placeholder="MM"
                maxlength="2"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                bind:value={editCardExpMonth}
                oninput={(e) => { const el = e.currentTarget; el.value = el.value.replace(/\D/g, '').slice(0, 2); editCardExpMonth = el.value; }}
              />
              {#if cardExpMonthError}
                <span class="control-error">{cardExpMonthError}</span>
              {/if}
            </div>
          </DetailFieldRow>
          <DetailFieldRow label="Exp year" alignStart>
            <div class="card-input-wrap">
              <input
                class="control control--compact edit-input"
                type="text"
                inputmode="numeric"
                pattern="[0-9]*"
                aria-invalid={Boolean(cardExpYearError)}
                placeholder="YYYY"
                maxlength="4"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                bind:value={editCardExpYear}
                oninput={(e) => { const el = e.currentTarget; el.value = el.value.replace(/\D/g, ''); editCardExpYear = el.value; }}
              />
              {#if cardExpYearError}
                <span class="control-error">{cardExpYearError}</span>
              {/if}
            </div>
          </DetailFieldRow>
          <DetailFieldRow label="CVV" alignStart>
            <div class="card-input-wrap">
              <input
                class="control control--compact edit-input"
                type="text"
                inputmode="numeric"
                pattern="[0-9]*"
                aria-invalid={Boolean(cardCvvError)}
                placeholder="CVV"
                maxlength="4"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                bind:value={editCardCvv}
                oninput={(e) => { const el = e.currentTarget; el.value = el.value.replace(/\D/g, '').slice(0, 4); editCardCvv = el.value; }}
              />
              {#if cardCvvError}
                <span class="control-error">{cardCvvError}</span>
              {/if}
            </div>
          </DetailFieldRow>
          <DetailFieldRow label="PIN" alignStart>
            <div class="card-input-wrap">
              <input
                class="control control--compact edit-input"
                type="text"
                inputmode="numeric"
                pattern="[0-9]*"
                aria-invalid={Boolean(cardPinError)}
                placeholder="PIN"
                maxlength="12"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                bind:value={editCardPin}
                oninput={(e) => { const el = e.currentTarget; el.value = el.value.replace(/\D/g, '').slice(0, 12); editCardPin = el.value; }}
              />
              {#if cardPinError}
                <span class="control-error">{cardPinError}</span>
              {/if}
            </div>
          </DetailFieldRow>
        {:else if entry.type === "software_license"}
          <DetailFieldRow label="Version">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Version"
              autocomplete="off"
              bind:value={editLicenseVersion}
            />
          </DetailFieldRow>
          <DetailFieldRow label="License key">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="License key"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editLicenseKey}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Licensed to">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Licensed to"
              autocomplete="off"
              bind:value={editLicenseLicensedTo}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Registered email">
            <input
              class="control control--compact edit-input"
              type="email"
              placeholder="Registered email"
              autocomplete="off"
              bind:value={editLicenseRegisteredEmail}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Company">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Company"
              autocomplete="off"
              bind:value={editLicenseCompany}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Download page">
            <input
              class="control control--compact edit-input"
              type="url"
              placeholder="Download page"
              autocomplete="off"
              bind:value={editLicenseDownloadPage}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Publisher">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Publisher"
              autocomplete="off"
              bind:value={editLicensePublisher}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Website">
            <input
              class="control control--compact edit-input"
              type="url"
              placeholder="Website"
              autocomplete="off"
              bind:value={editLicenseWebsite}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Retail price">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Retail price"
              autocomplete="off"
              bind:value={editLicenseRetailPrice}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Support email">
            <input
              class="control control--compact edit-input"
              type="email"
              placeholder="Support email"
              autocomplete="off"
              bind:value={editLicenseSupportEmail}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Purchase date">
            <input
              class="control control--compact edit-input"
              type="date"
              aria-label="Purchase date"
              autocomplete="off"
              bind:value={editLicensePurchaseDate}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Order number">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Order number"
              autocomplete="off"
              bind:value={editLicenseOrderNumber}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Order total">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Order total"
              autocomplete="off"
              bind:value={editLicenseOrderTotal}
            />
          </DetailFieldRow>
        {:else if entry.type === "passport"}
          <DetailFieldRow label="Type">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Passport type"
              autocomplete="off"
              bind:value={editPassportType}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Issuing country">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Issuing country"
              autocomplete="off"
              bind:value={editPassportIssuingCountry}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Number">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Passport number"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="characters"
              spellcheck="false"
              bind:value={editPassportNumber}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Full name">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Full name"
              autocomplete="off"
              bind:value={editPassportFullName}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Sex">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Sex"
              autocomplete="off"
              bind:value={editPassportSex}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Nationality">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Nationality"
              autocomplete="off"
              bind:value={editPassportNationality}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Issuing authority">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Issuing authority"
              autocomplete="off"
              bind:value={editPassportIssuingAuthority}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Date of birth">
            <input
              class="control control--compact edit-input"
              type="date"
              aria-label="Passport date of birth"
              autocomplete="bday"
              bind:value={editPassportBirthDate}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Place of birth">
            <input
              class="control control--compact edit-input"
              type="text"
              placeholder="Place of birth"
              autocomplete="off"
              bind:value={editPassportBirthPlace}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Issued on">
            <input
              class="control control--compact edit-input"
              type="date"
              aria-label="Passport issue date"
              autocomplete="off"
              bind:value={editPassportIssueDate}
            />
          </DetailFieldRow>
          <DetailFieldRow label="Expiry date">
            <input
              class="control control--compact edit-input"
              type="date"
              aria-label="Passport expiry date"
              autocomplete="off"
              bind:value={editPassportExpiryDate}
            />
          </DetailFieldRow>
        {/if}
      </FieldGroup>
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
          <Field
            label="Number"
            value={cardNumberPlain
              ? formatCardNumber(cardNumberPlain, entry.card.type)
              : (entry.card.numberMasked ?? "")}
            mono
            onCopy={() => clipboard.copySecretField(entry.id, "cardNumber")}
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

    {#if !editing && entry.expiresAt}
      {@const expirationDue = entry.expiresAt <= localDateString()}
      <div class="expiration-indicator" class:due={expirationDue} role="status">
        <Icon name={expirationDue ? "alert-triangle" : "calendar-time"} size={14} />
        <span>{expirationLabel(entry.expiresAt)}</span>
      </div>
    {/if}

    {#if editing}
      <div class="edit-expiration">
        <span class="notes-label">Expiration date</span>
        <input
          class="control control--compact expiration-input"
          type="date"
          aria-label="Entry expiration date"
          autocomplete="off"
          bind:value={editExpiresAt}
        />
      </div>
      <div class="custom-fields-editor">
        <div class="custom-fields-heading">
          <span class="notes-label">Custom fields</span>
          <button
            type="button"
            class="add-custom-field"
            onclick={() => {
              editCustomFields = [
                ...editCustomFields,
                { name: "", value: "", protected: false },
              ];
            }}
          >
            <Icon name="plus" size={13} />
            Add field
          </button>
        </div>
        {#each editCustomFields as field, index}
          <div class="custom-field-edit-row">
            <input
              class="control control--compact custom-field-name"
              placeholder="Field name"
              aria-label="Custom field name"
              autocomplete="off"
              bind:value={field.name}
            />
            <input
              class="control control--compact custom-field-value"
              type={field.protected ? "password" : "text"}
              placeholder="Value"
              aria-label="Custom field value"
              autocomplete="off"
              bind:value={field.value}
            />
            <label class="protect-custom-field" title="Protect this value in the vault">
              <input
                type="checkbox"
                bind:checked={field.protected}
                aria-label="Protect custom field"
              />
              <Icon name="lock" size={13} />
            </label>
            <button
              type="button"
              class="remove-custom-field"
              aria-label="Remove custom field"
              title="Remove custom field"
              onclick={() => {
                editCustomFields = editCustomFields.filter((_, itemIndex) => itemIndex !== index);
              }}
            >
              <Icon name="x" size={14} />
            </button>
          </div>
        {/each}
      </div>
      <div class="edit-tags">
        <span class="notes-label">Tags</span>
        <TagInput initialTags={editTags} onupdate={(t) => (editTags = t)} />
      </div>
      <div class="edit-notes">
        <span class="notes-label">Notes</span>
        <textarea
          class="control edit-textarea"
          placeholder="Notes"
          autocomplete="off"
          spellcheck="false"
          bind:value={editNotes}
        ></textarea>
      </div>
    {:else}
      {#if entry.tags.length > 0}
        <div class="tags-display">
          {#each entry.tags as tag}
            <span class="tag-badge">{tag}</span>
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
  <Dialog
    title="Move entry"
    onclose={() => (showMoveDialog = false)}
    onconfirm={addingMoveFolder ? createMoveFolder : moveEntry}
    size="sm"
  >
    <div class="move-entry-content">
      <div class="move-destination-heading">
        <label for="move-folder">Destination</label>
        <button
          type="button"
          class="new-folder-button"
          onclick={() => {
            addingMoveFolder = !addingMoveFolder;
            newMoveFolderName = "";
            newMoveFolderError = "";
          }}
        >
          <Icon name="folder-plus" size={13} />
          New folder
        </button>
      </div>
      <select
        id="move-folder"
        class="control control--compact control--select"
        bind:value={moveFolderId}
      >
        <option value="">Vault root</option>
        {#each moveFolders as folder (folder.id)}
          <option value={folder.id}>{folderPath(folder.id)}</option>
        {/each}
      </select>
      {#if addingMoveFolder}
        <div class="new-folder-form">
          <label class="control-label" for="new-move-folder">
            Create in {moveFolderId ? folderPath(moveFolderId) : "Vault root"}
          </label>
          <div class="new-folder-row">
            <!-- svelte-ignore a11y_autofocus -->
            <input
              id="new-move-folder"
              class="control control--compact"
              bind:value={newMoveFolderName}
              placeholder="Folder name"
              autocomplete="off"
              autofocus
              onkeydown={(event) => {
                if (event.key === "Enter") {
                  event.stopPropagation();
                  void createMoveFolder();
                }
              }}
            />
            <Button
              variant="outline"
              onclick={createMoveFolder}
              disabled={creatingMoveFolder || !newMoveFolderName.trim()}
            >
              {creatingMoveFolder ? "Creating…" : "Create"}
            </Button>
          </div>
          {#if newMoveFolderError}
            <p class="control-error">{newMoveFolderError}</p>
          {/if}
        </div>
      {/if}
      {#if moveError}
        <p class="save-error">{moveError}</p>
      {/if}
    </div>

    {#snippet footer()}
      <Button onclick={() => (showMoveDialog = false)}>Cancel</Button>
      <Button
        variant="primary"
        onclick={moveEntry}
        disabled={movingEntry || moveFolderId === (_entry?.folderId ?? "")}
      >
        {movingEntry ? "Moving…" : "Move"}
      </Button>
    {/snippet}
  </Dialog>
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
        toast.error(e instanceof Error ? e.message : String(e));
      }
    }}
  />
{/if}

<style>
  .move-entry-content {
    display: flex;
    flex-direction: column;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .move-entry-content .save-error {
    margin-bottom: 0;
  }

  .move-destination-heading,
  .new-folder-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .new-folder-button {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--text-accent);
    font-size: 12px;
  }

  .new-folder-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 4px;
    padding: 10px;
    border-radius: var(--radius-sm);
    background: var(--surface-1);
  }

  .new-folder-row .control {
    min-width: 0;
  }

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

  .toolbar-btn:hover {
    background: var(--border);
  }

  .toolbar-save {
    color: #fff;
    background: var(--accent);
    border-color: var(--accent);
  }

  .toolbar-save:hover {
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

  .card-input-wrap {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .card-input-wrap .edit-input {
    width: 100%;
  }

  .password-edit-col {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .password-edit-row {
    display: flex;
    flex: 1;
    gap: 6px;
    align-items: center;
  }

  .password-edit-row .edit-input {
    flex: 1;
  }

  .generate-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: 0.5px solid var(--border-strong);
    border-radius: var(--radius-sm);
    color: var(--accent);
    flex-shrink: 0;
    transition: background 0.1s;
  }

  .generate-btn:hover {
    background: var(--bg-accent);
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

  .edit-expiration {
    margin-bottom: 16px;
  }

  .expiration-input {
    width: 170px;
  }

  .custom-fields-editor {
    margin-bottom: 16px;
  }

  .custom-fields-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }

  .custom-fields-heading .notes-label {
    margin-bottom: 0;
  }

  .add-custom-field {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--accent);
    font-size: 12px;
  }

  .custom-field-edit-row {
    display: grid;
    grid-template-columns: minmax(100px, 0.7fr) minmax(140px, 1.3fr) 28px 28px;
    align-items: center;
    gap: 6px;
    margin-bottom: 6px;
  }

  .custom-field-name,
  .custom-field-value {
    width: 100%;
    min-width: 0;
  }

  .protect-custom-field,
  .remove-custom-field {
    display: flex;
    width: 28px;
    height: 28px;
    align-items: center;
    justify-content: center;
    border: 0.5px solid var(--border-strong);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    cursor: pointer;
  }

  .protect-custom-field:has(input:checked) {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--bg-accent);
  }

  .protect-custom-field input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .remove-custom-field:hover {
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 9%, transparent);
  }

  .edit-notes {
    margin-bottom: 16px;
  }

  .notes-label {
    display: block;
    font-size: 11px;
    color: var(--text-muted);
    margin-bottom: 6px;
  }

  .edit-textarea {
    min-height: 80px;
    border-radius: var(--radius);
    font-size: 13px;
    line-height: 1.55;
    resize: vertical;
  }

  .edit-tags {
    margin-bottom: 12px;
  }

  .tags-display {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 16px;
  }

  .tag-badge {
    display: inline-block;
    padding: 2px 8px;
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: 4px;
    font-size: 11.5px;
    color: var(--text-secondary);
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

  .totp-edit-wrap {
    display: flex;
    gap: 6px;
    align-items: center;
    flex: 1;
    min-width: 0;
  }

  .totp-edit-wrap .edit-input {
    flex: 1;
  }

  .totp-setup-btn-small {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: 0.5px solid var(--border-strong);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    flex-shrink: 0;
    transition: background 0.1s;
  }

  .totp-setup-btn-small:hover {
    background: var(--border);
    color: var(--accent);
  }
</style>
