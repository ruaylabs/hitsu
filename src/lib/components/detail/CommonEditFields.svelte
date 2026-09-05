<script lang="ts">
  import Icon from "../ui/Icon.svelte";
  import TagInput from "../ui/TagInput.svelte";
  import type { EditFormState } from "./editForm";
  import SecretEditInput from "./SecretEditInput.svelte";

  let { form }: { form: EditFormState } = $props();
</script>

<div class="edit-expiration">
  <span class="notes-label">Expiration date</span>
  <input
    class="control control--compact expiration-input"
    type="date"
    aria-label="Entry expiration date"
    autocomplete="off"
    bind:value={form.expiresAt}
  />
</div>
<div class="custom-fields-editor">
  <div class="custom-fields-heading">
    <span class="notes-label">Custom fields</span>
    <button
      type="button"
      class="add-custom-field"
      onclick={() => { form.customFields = [...form.customFields, { name: "", value: "", protected: false }]; }}
    >
      <Icon name="plus" size={13} />
      Add field
    </button>
  </div>
  {#each form.customFields as field, index}
    <div class="custom-field-edit-row">
      <input
        class="control control--compact custom-field-name"
        placeholder="Field name"
        aria-label="Custom field name"
        autocomplete="off"
        bind:value={field.name}
      />
      {#if field.protected}
        <SecretEditInput
          bind:value={field.value}
          label="custom field value"
          placeholder="Value"
          class="custom-field-value"
        />
      {:else}
        <input
          class="control control--compact custom-field-value"
          type="text"
          placeholder="Value"
          aria-label="Custom field value"
          autocomplete="off"
          bind:value={field.value}
        />
      {/if}
      <label class="protect-custom-field" title="Protect this value in the vault">
        <input type="checkbox" bind:checked={field.protected} aria-label="Protect custom field" />
        <Icon name="lock" size={13} />
      </label>
      <button
        type="button"
        class="remove-custom-field"
        aria-label="Remove custom field"
        title="Remove custom field"
        onclick={() => { form.customFields = form.customFields.filter((_, i) => i !== index); }}
      >
        <Icon name="x" size={14} />
      </button>
    </div>
  {/each}
</div>
<div class="edit-tags">
  <span class="notes-label">Tags</span>
  <TagInput initialTags={form.tags} onupdate={(t) => (form.tags = t)} />
</div>
<div class="edit-notes">
  <span class="notes-label">Notes</span>
  <textarea
    class="control edit-textarea"
    placeholder="Notes"
    autocomplete="off"
    spellcheck="false"
    bind:value={form.notes}
  ></textarea>
</div>

<style>
  .edit-expiration,
  .edit-tags,
  .edit-notes {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin-top: 12px;
  }
  .notes-label {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    font-weight: 600;
  }
  .expiration-input {
    max-width: 200px;
  }
  .custom-fields-editor {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-top: 12px;
  }
  .custom-fields-heading {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .add-custom-field {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-sm);
    color: var(--text-accent);
    padding: var(--space-half) var(--space-1);
    border-radius: var(--radius-xs);
  }
  .add-custom-field:hover {
    background: var(--surface-hover);
  }
  .custom-field-edit-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .custom-field-edit-row :global(.custom-field-name) {
    flex: 1;
    min-width: 0;
  }
  .custom-field-edit-row :global(.custom-field-value) {
    flex: 2;
    min-width: 0;
  }
  .protect-custom-field {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: var(--text-secondary);
  }
  .protect-custom-field input {
    display: none;
  }
  .protect-custom-field:has(input:checked) {
    color: var(--text-accent);
  }
  .remove-custom-field {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    padding: var(--space-half);
    border-radius: var(--radius-xs);
  }
  .remove-custom-field:hover {
    color: var(--danger);
    background: var(--surface-hover);
  }
  .edit-textarea {
    min-height: 100px;
    resize: vertical;
  }
</style>
