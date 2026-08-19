<script lang="ts">
  import * as entriesBridge from "$lib/bridge/entries";
  import type { Entry } from "$lib/bridge/types";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import Field from "./Field.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import PasswordField from "./PasswordField.svelte";

  let { entry }: { entry: Entry } = $props();
  const passport = $derived(entry.passport);
</script>

{#if passport}
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
