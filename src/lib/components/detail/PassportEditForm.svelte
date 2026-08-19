<script lang="ts">
  import DetailFieldRow from "./DetailFieldRow.svelte";
  import type { EditFormState } from "./editForm";
  import SecretEditInput from "./SecretEditInput.svelte";

  let { form }: { form: EditFormState } = $props();
</script>

<DetailFieldRow label="Type">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Passport type"
    autocomplete="off"
    bind:value={form.passportType}
  />
</DetailFieldRow>
<DetailFieldRow label="Issuing country">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Issuing country"
    autocomplete="off"
    bind:value={form.passportIssuingCountry}
  />
</DetailFieldRow>
<DetailFieldRow label="Number">
  <SecretEditInput
    bind:value={form.passportNumber}
    label="passport number"
    placeholder="Passport number"
  />
</DetailFieldRow>
<DetailFieldRow label="Full name">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Full name"
    autocomplete="off"
    bind:value={form.passportFullName}
  />
</DetailFieldRow>
<DetailFieldRow label="Sex">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Sex"
    autocomplete="off"
    bind:value={form.passportSex}
  />
</DetailFieldRow>
<DetailFieldRow label="Nationality">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Nationality"
    autocomplete="off"
    bind:value={form.passportNationality}
  />
</DetailFieldRow>
<DetailFieldRow label="Issuing authority">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Issuing authority"
    autocomplete="off"
    bind:value={form.passportIssuingAuthority}
  />
</DetailFieldRow>
<DetailFieldRow label="Date of birth">
  <input
    class="control control--compact edit-input"
    type="date"
    aria-label="Passport date of birth"
    autocomplete="bday"
    bind:value={form.passportBirthDate}
  />
</DetailFieldRow>
<DetailFieldRow label="Place of birth">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Place of birth"
    autocomplete="off"
    bind:value={form.passportBirthPlace}
  />
</DetailFieldRow>
<DetailFieldRow label="Issued on">
  <input
    class="control control--compact edit-input"
    type="date"
    aria-label="Passport issue date"
    autocomplete="off"
    bind:value={form.passportIssueDate}
  />
</DetailFieldRow>
<DetailFieldRow label="Expiry date">
  <input
    class="control control--compact edit-input"
    type="date"
    aria-label="Passport expiry date"
    autocomplete="off"
    bind:value={form.passportExpiryDate}
  />
</DetailFieldRow>
