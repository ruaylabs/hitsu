<script lang="ts">
  import DetailFieldRow from "./DetailFieldRow.svelte";
  import SecretEditInput from "./SecretEditInput.svelte";

  let {
    editLicenseVersion = $bindable(""),
    editLicenseKey = $bindable(""),
    editLicenseLicensedTo = $bindable(""),
    editLicenseRegisteredEmail = $bindable(""),
    editLicenseCompany = $bindable(""),
    editLicenseDownloadPage = $bindable(""),
    editLicensePublisher = $bindable(""),
    editLicenseWebsite = $bindable(""),
    editLicenseRetailPrice = $bindable(""),
    editLicenseSupportEmail = $bindable(""),
    editLicensePurchaseDate = $bindable(""),
    editLicenseOrderNumber = $bindable(""),
    editLicenseOrderTotal = $bindable(""),
  }: {
    editLicenseVersion?: string;
    editLicenseKey?: string;
    editLicenseLicensedTo?: string;
    editLicenseRegisteredEmail?: string;
    editLicenseCompany?: string;
    editLicenseDownloadPage?: string;
    editLicensePublisher?: string;
    editLicenseWebsite?: string;
    editLicenseRetailPrice?: string;
    editLicenseSupportEmail?: string;
    editLicensePurchaseDate?: string;
    editLicenseOrderNumber?: string;
    editLicenseOrderTotal?: string;
  } = $props();
</script>

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
  <SecretEditInput bind:value={editLicenseKey} label="license key" placeholder="License key" />
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
