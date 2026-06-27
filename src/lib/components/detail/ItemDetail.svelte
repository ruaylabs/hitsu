<script lang="ts">
  import { vault } from "$lib/stores/vault.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import DetailHeader from "./DetailHeader.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import Field from "./Field.svelte";
  import PasswordField from "./PasswordField.svelte";
  import TotpField from "./TotpField.svelte";
  import NotesField from "./NotesField.svelte";
  import AttachmentList from "./AttachmentList.svelte";
  import DetailFooter from "./DetailFooter.svelte";
  import EmptyDetail from "./EmptyDetail.svelte";

  let entry = $derived(selection.selectedId ? vault.getEntry(selection.selectedId) : undefined);
</script>

{#if entry}
  <div class="detail-pane">
    <DetailHeader {entry} />

    {#if entry.type === "login" || entry.type === "note"}
      {#if entry.totp}
        <TotpField totpUri={entry.totp} />
      {/if}

      <FieldGroup>
        {#if entry.username}
          <Field
            label="Username"
            value={entry.username}
            onCopy={() => navigator.clipboard.writeText(entry.username!)}
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
            onCopy={() => navigator.clipboard.writeText(entry.url!)}
          />
        {/if}
      </FieldGroup>
    {/if}

    {#if entry.type === "identity" && entry.identity}
      <FieldGroup>
        {#if entry.identity.firstName}
          <Field label="First Name" value={entry.identity.firstName} />
        {/if}
        {#if entry.identity.lastName}
          <Field label="Last Name" value={entry.identity.lastName} />
        {/if}
        {#if entry.identity.email}
          <Field
            label="Email"
            value={entry.identity.email}
            onCopy={() => navigator.clipboard.writeText(entry.identity!.email!)}
          />
        {/if}
        {#if entry.identity.phone}
          <Field
            label="Phone"
            value={entry.identity.phone}
            onCopy={() => navigator.clipboard.writeText(entry.identity!.phone!)}
          />
        {/if}
        {#if entry.identity.address}
          <Field
            label="Address"
            value={entry.identity.address}
            onCopy={() => navigator.clipboard.writeText(entry.identity!.address!)}
          />
        {/if}
      </FieldGroup>
    {/if}

    {#if entry.type === "card" && entry.card}
      <FieldGroup>
        {#if entry.card.brand}
          <Field label="Brand" value={entry.card.brand} />
        {/if}
        {#if entry.card.holder}
          <Field label="Holder" value={entry.card.holder} />
        {/if}
        {#if entry.card.number}
          <PasswordField label="Number" password={entry.card.number} />
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

    {#if entry.customFields.length > 0}
      <FieldGroup>
        {#each entry.customFields as cf}
          {#if cf.protected}
            <PasswordField label={cf.name} password={cf.value} />
          {:else}
            <Field
              label={cf.name}
              value={cf.value}
              onCopy={() => navigator.clipboard.writeText(cf.value)}
            />
          {/if}
        {/each}
      </FieldGroup>
    {/if}

    {#if entry.type === "note"}
      <NotesField notes={entry.notes || ""} />
    {:else if entry.notes}
      <NotesField notes={entry.notes} />
    {/if}

    <AttachmentList attachments={entry.attachments} />
    <DetailFooter modifiedAt={entry.modifiedAt} historyCount={entry.historyCount} />
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
</style>
