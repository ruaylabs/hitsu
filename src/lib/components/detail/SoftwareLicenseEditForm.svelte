<script lang="ts">
  import DetailFieldRow from "./DetailFieldRow.svelte";
  import type { EditFormState } from "./editForm";
  import SecretEditInput from "./SecretEditInput.svelte";

  let { form }: { form: EditFormState } = $props();
</script>

<DetailFieldRow label="Version">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Version"
    autocomplete="off"
    bind:value={form.licenseVersion}
  />
</DetailFieldRow>
<DetailFieldRow label="License key">
  <SecretEditInput bind:value={form.licenseKey} label="license key" placeholder="License key" />
</DetailFieldRow>
<DetailFieldRow label="Licensed to">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Licensed to"
    autocomplete="off"
    bind:value={form.licenseLicensedTo}
  />
</DetailFieldRow>
<DetailFieldRow label="Registered email">
  <input
    class="control control--compact edit-input"
    type="email"
    placeholder="Registered email"
    autocomplete="off"
    bind:value={form.licenseRegisteredEmail}
  />
</DetailFieldRow>
<DetailFieldRow label="Company">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Company"
    autocomplete="off"
    bind:value={form.licenseCompany}
  />
</DetailFieldRow>
<DetailFieldRow label="Download page">
  <input
    class="control control--compact edit-input"
    type="url"
    placeholder="Download page"
    autocomplete="off"
    bind:value={form.licenseDownloadPage}
  />
</DetailFieldRow>
<DetailFieldRow label="Publisher">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Publisher"
    autocomplete="off"
    bind:value={form.licensePublisher}
  />
</DetailFieldRow>
<DetailFieldRow label="Website">
  <input
    class="control control--compact edit-input"
    type="url"
    placeholder="Website"
    autocomplete="off"
    bind:value={form.licenseWebsite}
  />
</DetailFieldRow>
<DetailFieldRow label="Retail price">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Retail price"
    autocomplete="off"
    bind:value={form.licenseRetailPrice}
  />
</DetailFieldRow>
<DetailFieldRow label="Support email">
  <input
    class="control control--compact edit-input"
    type="email"
    placeholder="Support email"
    autocomplete="off"
    bind:value={form.licenseSupportEmail}
  />
</DetailFieldRow>
<DetailFieldRow label="Purchase date">
  <input
    class="control control--compact edit-input"
    type="date"
    aria-label="Purchase date"
    autocomplete="off"
    bind:value={form.licensePurchaseDate}
  />
</DetailFieldRow>
<DetailFieldRow label="Order number">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Order number"
    autocomplete="off"
    bind:value={form.licenseOrderNumber}
  />
</DetailFieldRow>
<DetailFieldRow label="Order total">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Order total"
    autocomplete="off"
    bind:value={form.licenseOrderTotal}
  />
</DetailFieldRow>
