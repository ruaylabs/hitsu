<script lang="ts">
  import { vault } from "$lib/stores/vault.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import * as entriesBridge from "$lib/bridge/entries";
  import DetailHeader from "./DetailHeader.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import Field from "./Field.svelte";
  import PasswordField from "./PasswordField.svelte";
  import TOTPField from "./TOTPField.svelte";
  import NotesField from "./NotesField.svelte";
  import AttachmentList from "./AttachmentList.svelte";
  import DetailFooter from "./DetailFooter.svelte";
  import EmptyDetail from "./EmptyDetail.svelte";
  import Icon from "../ui/Icon.svelte";
  import TagInput from "../ui/TagInput.svelte";
  import GeneratorPanel from "../generator/GeneratorPanel.svelte";

  let entry = $derived(selection.selectedId ? vault.getEntry(selection.selectedId) : undefined);

  let editing = $state(false);
  let showGenerator = $state(false);
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

  // Auto-enter edit mode when a new entry is created
  $effect(() => {
    if (entry && vault.editingId === entry.id) {
      populateEdit();
      editing = true;
      vault.setEditingId(null);
    }
  });

  function populateEdit() {
    if (!entry) return;
    editTitle = entry.title;
    editUsername = entry.username ?? "";
    editPassword = entry.password ?? "";
    editUrl = entry.url ?? "";
    editTotp = entry.totp ?? "";
    editTags = [...entry.tags];
    editNotes = entry.notes ?? "";
    editFirstName = entry.identity?.firstName ?? "";
    editLastName = entry.identity?.lastName ?? "";
    editEmail = entry.identity?.email ?? "";
    editPhone = entry.identity?.phone ?? "";
    editAddress = entry.identity?.address ?? "";
    editCardHolder = entry.card?.holder ?? "";
    editCardNumber = entry.card?.number ?? "";
    editCardType = entry.card?.type ?? "";
    editCardExpMonth = entry.card?.expMonth?.toString() ?? "";
    editCardExpYear = entry.card?.expYear?.toString() ?? "";
    editCardCvv = entry.card?.cvv ?? "";
  }

  function startEdit() {
    if (!entry) return;
    populateEdit();
    editing = true;
  }

  function cancelEdit() {
    editing = false;
  }

  async function saveEdit() {
    if (!entry) return;
    try {
      await entriesBridge.entryUpdate(entry.id, {
        title: editTitle || undefined,
        username: editUsername || undefined,
        password: editPassword || undefined,
        url: editUrl || undefined,
        totp: editTotp || undefined,
        notes: editNotes || undefined,
        tags: editTags.length > 0 ? editTags : undefined,
        firstName: editFirstName || undefined,
        lastName: editLastName || undefined,
        email: editEmail || undefined,
        phone: editPhone || undefined,
        address: editAddress || undefined,
        cardHolder: editCardHolder || undefined,
        cardNumber: editCardNumber || undefined,
        cardType: editCardType || undefined,
        cardExpMonth: editCardExpMonth || undefined,
        cardExpYear: editCardExpYear || undefined,
        cardCvv: editCardCvv || undefined,
      });
      const updated = await entriesBridge.entryGet(entry.id);
      vault.setEntries(vault.entries.map((e) => (e.id === updated.id ? updated : e)));
      editing = false;
    } catch (e) {
      console.error("Failed to save", e);
    }
  }

  async function deleteEntry() {
    if (!entry) return;
    if (!confirm(`Delete "${entry.title}"?`)) return;
    try {
      await entriesBridge.entryDelete(entry.id);
      vault.setEntries(vault.entries.filter((e) => e.id !== entry.id));
      selection.selectedId = null;
    } catch (e) {
      console.error("Failed to delete", e);
    }
  }
</script>

{#if showGenerator}
  <GeneratorPanel
    onUse={(pw) => { editPassword = pw; showGenerator = false; }}
    oncancel={() => (showGenerator = false)}
  />
{/if}

{#if entry}
  <div class="detail-pane">
    <div class="detail-toolbar">
      <button
        class="toolbar-btn"
        onclick={editing ? cancelEdit : startEdit}
        aria-label={editing ? "Cancel" : "Edit"}
      >
        <Icon name={editing ? "x" : "pencil"} size={14} />
        <span>{editing ? "Cancel" : "Edit"}</span>
      </button>
      {#if editing}
        <button class="toolbar-btn toolbar-save" onclick={saveEdit} aria-label="Save">
          <Icon name="check" size={14} />
          <span>Save</span>
        </button>
      {/if}
      {#if !editing}
        <button class="toolbar-btn toolbar-delete" onclick={deleteEntry} aria-label="Delete">
          <Icon name="trash" size={14} />
          <span>Delete</span>
        </button>
      {/if}
    </div>

    {#if editing}
      <div class="edit-title">
        <input
          class="edit-input edit-title-input"
          type="text"
          placeholder="Title"
          bind:value={editTitle}
        />
      </div>
    {:else}
      <DetailHeader {entry} />
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
              bind:value={editUsername}
            />
          </div>
          <div class="field-row">
            <span class="field-label">Password</span>
            <div class="password-edit-row">
              <input
                class="edit-input"
                type="text"
                placeholder="Password"
                bind:value={editPassword}
              />
              <button
                class="generate-btn"
                onclick={() => (showGenerator = true)}
                aria-label="Generate password"
              >
                <Icon name="bolt" size={14} />
              </button>
            </div>
          </div>
          <div class="field-row">
            <span class="field-label">URL</span>
            <input class="edit-input" type="text" placeholder="URL" bind:value={editUrl} />
          </div>
          <div class="field-row">
            <span class="field-label">TOTP</span>
            <input
              class="edit-input"
              type="text"
              placeholder="otpauth:// URI"
              bind:value={editTotp}
            />
          </div>
        {:else if entry.type === "identity"}
          <div class="field-row">
            <span class="field-label">First name</span>
            <input
              class="edit-input"
              type="text"
              placeholder="First name"
              bind:value={editFirstName}
            />
          </div>
          <div class="field-row">
            <span class="field-label">Last name</span>
            <input
              class="edit-input"
              type="text"
              placeholder="Last name"
              bind:value={editLastName}
            />
          </div>
          <div class="field-row">
            <span class="field-label">Email</span>
            <input class="edit-input" type="text" placeholder="Email" bind:value={editEmail} />
          </div>
          <div class="field-row">
            <span class="field-label">Phone</span>
            <input class="edit-input" type="text" placeholder="Phone" bind:value={editPhone} />
          </div>
          <div class="field-row">
            <span class="field-label">Address</span>
            <input class="edit-input" type="text" placeholder="Address" bind:value={editAddress} />
          </div>
        {:else if entry.type === "card"}
          <div class="field-row">
            <span class="field-label">Holder</span>
            <input
              class="edit-input"
              type="text"
              placeholder="Card holder"
              bind:value={editCardHolder}
            />
          </div>
          <div class="field-row">
            <span class="field-label">Number</span>
            <input
              class="edit-input"
              type="text"
              placeholder="Card number"
              bind:value={editCardNumber}
            />
          </div>
          <div class="field-row">
            <span class="field-label">Type</span>
            <select class="edit-select" bind:value={editCardType}>
              <option value="">Select brand</option>
              <option value="Visa">Visa</option>
              <option value="Mastercard">Mastercard</option>
              <option value="American Express">American Express</option>
              <option value="Discover">Discover</option>
              <option value="Diners Club">Diners Club</option>
              <option value="JCB">JCB</option>
              <option value="UnionPay">UnionPay</option>
              <option value="Maestro">Maestro</option>
            </select>
          </div>
          <div class="field-row">
            <span class="field-label">Exp month</span>
            <input class="edit-input" type="text" placeholder="MM" bind:value={editCardExpMonth} />
          </div>
          <div class="field-row">
            <span class="field-label">Exp year</span>
            <input class="edit-input" type="text" placeholder="YYYY" bind:value={editCardExpYear} />
          </div>
          <div class="field-row">
            <span class="field-label">CVV</span>
            <input class="edit-input" type="text" placeholder="CVV" bind:value={editCardCvv} />
          </div>
        {/if}
      </FieldGroup>
    {:else if entry.type === "login" || entry.type === "note"}
      {#if entry.totp}
        <TOTPField totpUri={entry.totp} />
      {/if}
      <FieldGroup>
        {#if entry.username}
          <Field
            label="Username"
            value={entry.username}
            onCopy={() => clipboard.copyPlain(entry.username!)}
          />
        {/if}
        {#if entry.password}
          <PasswordField label="Password" password={entry.password} />
        {/if}
        {#if entry.url}
          <Field
            label="URL"
            value={entry.url}
            mono={false}
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
          <Field label="Type" value={entry.card.type} />
        {/if}
        {#if entry.card.holder}
          <Field label="Holder" value={entry.card.holder} />
        {/if}
        {#if entry.card.number}
          <Field
            label="Number"
            value={entry.card.number}
            onCopy={() => clipboard.copyPlain(entry.card!.number!)}
          />
        {/if}
        {#if entry.card.expMonth && entry.card.expYear}
          <Field
            label="Expires"
            value={`${String(entry.card.expMonth).padStart(2, "0")}/${entry.card.expYear}`}
          />
        {/if}
        {#if entry.card.cvv}
          <PasswordField label="CVV" password={entry.card.cvv} />
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
        <textarea class="edit-textarea" placeholder="Notes" bind:value={editNotes}></textarea>
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
      <AttachmentList attachments={entry.attachments} />
      <DetailFooter modifiedAt={entry.modifiedAt} historyCount={entry.historyCount} />
    {/if}
  </div>
{:else}
  <EmptyDetail />
{/if}

<style>
  .detail-pane {
    padding: 22px 24px;
    min-width: 0;
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
</style>
