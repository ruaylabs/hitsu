<script lang="ts">
  import DetailFieldRow from "./DetailFieldRow.svelte";
  import SecretEditInput from "./SecretEditInput.svelte";

  let {
    editPassportType = $bindable(""),
    editPassportIssuingCountry = $bindable(""),
    editPassportNumber = $bindable(""),
    editPassportFullName = $bindable(""),
    editPassportSex = $bindable(""),
    editPassportNationality = $bindable(""),
    editPassportIssuingAuthority = $bindable(""),
    editPassportBirthDate = $bindable(""),
    editPassportBirthPlace = $bindable(""),
    editPassportIssueDate = $bindable(""),
    editPassportExpiryDate = $bindable(""),
  }: {
    editPassportType?: string;
    editPassportIssuingCountry?: string;
    editPassportNumber?: string;
    editPassportFullName?: string;
    editPassportSex?: string;
    editPassportNationality?: string;
    editPassportIssuingAuthority?: string;
    editPassportBirthDate?: string;
    editPassportBirthPlace?: string;
    editPassportIssueDate?: string;
    editPassportExpiryDate?: string;
  } = $props();
</script>

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
  <SecretEditInput
    bind:value={editPassportNumber}
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
