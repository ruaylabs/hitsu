<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { untrack } from "svelte";
  import * as entriesBridge from "$lib/bridge/entries";
  import { toSummary } from "$lib/bridge/entries";
  import type { Entry } from "$lib/bridge/types";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { entryDeletion } from "$lib/stores/entryDeletion.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { CARD_BRANDS, cardBrandName, formatCardNumber } from "$lib/utils/format";
  import GeneratorPanel from "../generator/GeneratorPanel.svelte";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";
  import Icon from "../ui/Icon.svelte";
  import PasswordStrengthMeter from "../ui/PasswordStrengthMeter.svelte";
  import TagInput from "../ui/TagInput.svelte";
  import TotpSetupDialog from "../ui/TotpSetupDialog.svelte";
  import AttachmentList from "./AttachmentList.svelte";
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

  let fetchId = 0;
  let loadingTimer: ReturnType<typeof setTimeout> | undefined;

  // Fetch the full entry whenever selection changes
  $effect(() => {
    const id = selection.selectedId;
    if (!id) {
      _entry = undefined;
      entryLoading = false;
      entryError = "";
      return;
    }
    const thisFetch = ++fetchId;
    // Keep previous entry visible during refetch; only show "Loading…" after a
    // short delay and only when we have no prior data to display.
    // untrack: this effect must depend only on the selected id — reading
    // `_entry` tracked would make the `_entry = e` assignment below re-trigger
    // the effect, refetching in a loop on every completion.
    if (loadingTimer) clearTimeout(loadingTimer);
    if (!untrack(() => _entry)) {
      loadingTimer = setTimeout(() => {
        if (thisFetch === fetchId) entryLoading = true;
      }, 120);
    }
    entryError = "";
    entriesBridge
      .entryGet(id)
      .then((e) => {
        if (thisFetch === fetchId) {
          if (loadingTimer) clearTimeout(loadingTimer);
          _entry = e;
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
  });

  // Card numbers are shown in full by default (unlike password/CVV/PIN),
  // so fetch the plaintext when a card entry is selected. Falls back to the
  // backend-masked value until it arrives (or if the fetch fails).
  let cardNumberPlain = $state("");
  // Which entry id the fetched (or in-flight) number belongs to. Keyed on the
  // id, not the `_entry` object: `_entry` is reassigned on refetches of the
  // same entry (favorite toggle, save), and resetting/refetching then would
  // flash the field back to the masked value.
  let cardNumberEntryId: string | null = null;
  $effect(() => {
    const e = _entry;
    const wantId = e?.type === "card" && e.card?.hasNumber ? e.id : null;
    if (wantId === cardNumberEntryId) return;
    cardNumberEntryId = wantId;
    cardNumberPlain = "";
    if (wantId) {
      entriesBridge
        .entryRevealField(wantId, "cardNumber")
        .then((n) => {
          if (cardNumberEntryId === wantId) cardNumberPlain = n;
        })
        .catch((err) => console.error("Failed to load card number", err));
    }
  });

  let editing = $state(false);
  let newEntryId = $state<string | null>(null);
  let showHistory = $state(false);
  let showGenerator = $state(false);
  let showTotpSetup = $state(false);
  let editTitle = $state("");
  let editUsername = $state("");
  let editPassword = $state("");
  let editUrl = $state("");
  let editTotp = $state("");
  let editNotes = $state("");
  let editTags = $state<string[]>([]);
  // Identity fields
  let editFirstName = $state("");
  let editLastName = $state("");
  let editEmail = $state("");
  let editPhone = $state("");
  let editAddress = $state("");
  // Card fields
  let editCardHolder = $state("");
  let editCardNumber = $state("");
  let editCardType = $state("");
  let editCardExpMonth = $state("");
  let editCardExpYear = $state("");
  let editCardCvv = $state("");
  let editCardPin = $state("");

  // Validation errors for card fields
  let cardNumberError = $state("");
  let cardExpMonthError = $state("");
  let cardExpYearError = $state("");
  let cardCvvError = $state("");
  let cardPinError = $state("");
  let initialEditSnapshot = "";
  let pendingNavigation = $state<(() => void) | null>(null);

  function editSnapshot() {
    return JSON.stringify({
      title: editTitle,
      username: editUsername,
      password: editPassword,
      url: editUrl,
      totp: editTotp,
      notes: editNotes,
      tags: editTags,
      firstName: editFirstName,
      lastName: editLastName,
      email: editEmail,
      phone: editPhone,
      address: editAddress,
      cardHolder: editCardHolder,
      cardNumber: editCardNumber,
      cardType: editCardType,
      cardExpMonth: editCardExpMonth,
      cardExpYear: editCardExpYear,
      cardCvv: editCardCvv,
      cardPin: editCardPin,
    });
  }

  function hasUnsavedChanges() {
    return newEntryId !== null || editSnapshot() !== initialEditSnapshot;
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
      if (selectedId !== _entry?.id) {
        // Clear the discarded stub from the pane until the next entry loads.
        _entry = undefined;
      }
      clearCardErrors();
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
    editFirstName = e.identity?.firstName ?? "";
    editLastName = e.identity?.lastName ?? "";
    editEmail = e.identity?.email ?? "";
    editPhone = e.identity?.phone ?? "";
    editAddress = e.identity?.address ?? "";
    editCardHolder = e.card?.holder ?? "";
    editCardType = e.card?.type ?? "";
    editCardExpMonth = e.card?.expMonth?.toString() ?? "";
    editCardExpYear = e.card?.expYear?.toString() ?? "";
    // Secrets are not in the Entry DTO — fetch the ones that exist so the
    // form is prefilled and an untouched save round-trips them unchanged.
    const [password, totp, cardNumber, cardCvv, cardPin] = await Promise.all([
      e.hasPassword ? entriesBridge.entryRevealField(e.id, "password") : "",
      e.hasTotp ? entriesBridge.entryRevealField(e.id, "totp") : "",
      e.card?.hasNumber ? entriesBridge.entryRevealField(e.id, "cardNumber") : "",
      e.card?.hasCvv ? entriesBridge.entryRevealField(e.id, "cardCvv") : "",
      e.card?.hasPin ? entriesBridge.entryRevealField(e.id, "cardPin") : "",
    ]);
    editPassword = password;
    editTotp = totp;
    editCardNumber = cardNumber;
    editCardCvv = cardCvv;
    editCardPin = cardPin;
    initialEditSnapshot = editSnapshot();
    clearCardErrors();
  }

  async function toggleFavorite() {
    if (!_entry) return;
    try {
      const updated = await entriesBridge.entryUpdate(_entry.id, {
        favorite: !_entry.favorite,
      });
      _entry = updated;
      vault.setEntries(vault.entries.map((s) => (s.id === updated.id ? toSummary(updated) : s)));
    } catch (e) {
      console.error("Failed to toggle favorite", e);
      toast.error(e instanceof Error ? e.message : String(e));
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
    clearCardErrors();
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
    if (!validateCardFields()) return false;
    saveError = "";
    try {
      const updated = await entriesBridge.entryUpdate(_entry.id, {
        title: editTitle,
        username: editUsername,
        password: editPassword,
        url: editUrl,
        totp: editTotp,
        notes: editNotes,
        tags: editTags,
        firstName: editFirstName,
        lastName: editLastName,
        email: editEmail,
        phone: editPhone,
        address: editAddress,
        cardHolder: editCardHolder,
        cardNumber: editCardNumber,
        cardType: editCardType,
        cardExpMonth: editCardExpMonth,
        cardExpYear: editCardExpYear,
        cardCvv: editCardCvv,
        cardPin: editCardPin,
      });
      _entry = updated;
      vault.setEntries(vault.entries.map((s) => (s.id === updated.id ? toSummary(updated) : s)));
      editing = false;
      newEntryId = null;
      initialEditSnapshot = editSnapshot();
      clearCardErrors();
      return true;
    } catch (e) {
      // Surface the failure (e.g. the vault file changed on disk) instead
      // of silently staying in edit mode.
      saveError = e instanceof Error ? e.message : String(e);
      console.error("Failed to save", e);
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
    clearCardErrors();
    pendingNavigation = null;
    navigate();
  }

  $effect(() => {
    if (!editing) return;
    return selection.setNavigationGuard((navigate) => {
      if (pendingNavigation) return false;
      if (!hasUnsavedChanges()) {
        editing = false;
        return true;
      }
      pendingNavigation = navigate;
      return false;
    });
  });

  function confirmDelete() {
    if (!_entry) return;
    // The shared flow removes the entry and clears the selection; the
    // callback resets local state so the auto-discard effect doesn't try
    // to discard an entry that was just deleted.
    entryDeletion.request(_entry.id, _entry.title, () => {
      editing = false;
      newEntryId = null;
      _entry = undefined;
    });
  }

  function openEntryUrl(rawUrl: string): void {
    const url = rawUrl.includes("://") ? rawUrl : `https://${rawUrl}`;
    try {
      const parsed = new URL(url);
      if (parsed.protocol === "http:" || parsed.protocol === "https:") {
        openUrl(url);
      } else {
        console.warn("Blocked opening URL with disallowed scheme:", parsed.protocol);
      }
    } catch {
      console.warn("Blocked opening invalid URL:", url);
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
      {#if editing}
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
      <p class="save-error">{saveError}</p>
    {/if}

    {#if editing}
      <div class="edit-title">
        <input
          class="edit-input edit-title-input"
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
        onTotpSetup={() => (showTotpSetup = true)}
        showTotpSetup={entry.type === "login" && !entry.hasTotp}
      />
    {/if}

    {#if editing}
      <!-- Edit mode: type-specific fields -->
      <FieldGroup>
        {#if entry.type === "login"}
          <div class="field-row">
            <span class="field-label">Username</span>
            <input
              class="edit-input"
              type="text"
              placeholder="Username"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editUsername}
            />
          </div>
          <div class="field-row">
            <span class="field-label">Password</span>
            <div class="password-edit-col">
              <div class="password-edit-row">
                <input
                  class="edit-input"
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
          </div>
          <div class="field-row">
            <span class="field-label">URL</span>
            <input
              class="edit-input"
              type="text"
              placeholder="URL"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editUrl}
            />
          </div>
          <div class="field-row">
            <span class="field-label">TOTP</span>
            <div class="totp-edit-wrap">
              <input
                class="edit-input"
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
          </div>
        {:else if entry.type === "identity"}
          <div class="field-row">
            <span class="field-label">First name</span>
            <input
              class="edit-input"
              type="text"
              placeholder="First name"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editFirstName}
            />
          </div>
          <div class="field-row">
            <span class="field-label">Last name</span>
            <input
              class="edit-input"
              type="text"
              placeholder="Last name"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editLastName}
            />
          </div>
          <div class="field-row">
            <span class="field-label">Email</span>
            <input
              class="edit-input"
              type="text"
              placeholder="Email"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editEmail}
            />
          </div>
          <div class="field-row">
            <span class="field-label">Phone</span>
            <input
              class="edit-input"
              type="text"
              placeholder="Phone"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editPhone}
            />
          </div>
          <div class="field-row">
            <span class="field-label">Address</span>
            <input
              class="edit-input"
              type="text"
              placeholder="Address"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editAddress}
            />
          </div>
        {:else if entry.type === "card"}
          <div class="field-row">
            <span class="field-label">Holder</span>
            <input
              class="edit-input"
              type="text"
              placeholder="Card holder"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              bind:value={editCardHolder}
            />
          </div>
          <div class="field-row card-field-row">
            <span class="field-label">Number</span>
            <div class="card-input-wrap">
              <input
                class="edit-input"
                type="text"
                inputmode="numeric"
                pattern="[0-9]*"
                placeholder="Card number"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                bind:value={editCardNumber}
                oninput={(e) => { const el = e.currentTarget; el.value = el.value.replace(/\D/g, ''); editCardNumber = el.value; }}
              />
              {#if cardNumberError}
                <span class="field-error">{cardNumberError}</span>
              {/if}
            </div>
          </div>
          <div class="field-row">
            <span class="field-label">Type</span>
            <select class="edit-select" bind:value={editCardType}>
              <option value="">Select brand</option>
              {#each Object.entries(CARD_BRANDS) as [ key, name ]}
                <option value={key}>{name}</option>
              {/each}
            </select>
          </div>
          <div class="field-row card-field-row">
            <span class="field-label">Exp month</span>
            <div class="card-input-wrap">
              <input
                class="edit-input"
                type="text"
                inputmode="numeric"
                pattern="[0-9]*"
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
                <span class="field-error">{cardExpMonthError}</span>
              {/if}
            </div>
          </div>
          <div class="field-row card-field-row">
            <span class="field-label">Exp year</span>
            <div class="card-input-wrap">
              <input
                class="edit-input"
                type="text"
                inputmode="numeric"
                pattern="[0-9]*"
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
                <span class="field-error">{cardExpYearError}</span>
              {/if}
            </div>
          </div>
          <div class="field-row card-field-row">
            <span class="field-label">CVV</span>
            <div class="card-input-wrap">
              <input
                class="edit-input"
                type="text"
                inputmode="numeric"
                pattern="[0-9]*"
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
                <span class="field-error">{cardCvvError}</span>
              {/if}
            </div>
          </div>
          <div class="field-row card-field-row">
            <span class="field-label">PIN</span>
            <div class="card-input-wrap">
              <input
                class="edit-input"
                type="text"
                inputmode="numeric"
                pattern="[0-9]*"
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
                <span class="field-error">{cardPinError}</span>
              {/if}
            </div>
          </div>
        {/if}
      </FieldGroup>
    {:else if entry.type === "login" || entry.type === "note"}
      {#if entry.hasTotp}
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
            onOpenUrl={() => openEntryUrl(entry.url!)}
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

    {#if editing}
      <div class="edit-tags">
        <span class="notes-label">Tags</span>
        <TagInput initialTags={editTags} onupdate={(t) => (editTags = t)} />
      </div>
      <div class="edit-notes">
        <span class="notes-label">Notes</span>
        <textarea
          class="edit-textarea"
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
    {/if}

    {#if !editing}
      <AttachmentList
        entryId={entry.id}
        attachments={entry.attachments}
        onchange={() => {
          entriesBridge.entryGet(entry.id).then((e) => { _entry = e; }).catch(() => {});
        }}
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

{#if showTotpSetup && _entry}
  <TotpSetupDialog
    oncancel={() => (showTotpSetup = false)}
    onconfirm={async (uri) => {
      showTotpSetup = false;
      if (!_entry) return;
      try {
        const updated = await entriesBridge.entryUpdate(_entry.id, { totp: uri });
        _entry = updated;
        vault.setEntries(vault.entries.map((s) => (s.id === updated.id ? toSummary(updated) : s)));
        if (editing) {
          editTotp = uri;
        }
        toast.success("TOTP configured successfully");
      } catch (e) {
        toast.error(e instanceof Error ? e.message : String(e));
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

  .edit-input {
    width: 100%;
    padding: 6px 8px;
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 13.5px;
    color: var(--text-primary);
  }

  .card-field-row {
    align-items: flex-start;
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

  .field-error {
    font-size: 11px;
    color: var(--danger);
    line-height: 1.3;
  }

  .edit-input:focus,
  .edit-select:focus {
    border-color: var(--accent);
    outline: none;
  }

  .edit-select {
    width: 100%;
    padding: 6px 8px;
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 13.5px;
    color: var(--text-primary);
    cursor: pointer;
    appearance: none;
    -webkit-appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23a1a09a' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
    padding-right: 28px;
  }

  .field-row {
    background: var(--surface-2);
    padding: 10px 12px;
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 38px;
  }

  .field-label {
    font-size: 11px;
    color: var(--text-muted);
    width: 70px;
    flex-shrink: 0;
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
    width: 100%;
    min-height: 80px;
    padding: 8px 10px;
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
    font-size: 13px;
    line-height: 1.55;
    color: var(--text-primary);
    resize: vertical;
  }

  .edit-textarea:focus {
    border-color: var(--accent);
    outline: none;
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
