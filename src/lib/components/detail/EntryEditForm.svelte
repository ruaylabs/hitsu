<script lang="ts">
  import CardEditForm from "./CardEditForm.svelte";
  import CommonEditFields from "./CommonEditFields.svelte";
  import type { EditFormState } from "./editForm";
  import FieldGroup from "./FieldGroup.svelte";
  import IdentityEditForm from "./IdentityEditForm.svelte";
  import LoginEditForm from "./LoginEditForm.svelte";
  import PassportEditForm from "./PassportEditForm.svelte";
  import PasswordEditForm from "./PasswordEditForm.svelte";
  import PgpKeyEditForm from "./PgpKeyEditForm.svelte";
  import SoftwareLicenseEditForm from "./SoftwareLicenseEditForm.svelte";

  let {
    entryType,
    form = $bindable(),
    cardNumberError = "",
    cardExpMonthError = "",
    cardExpYearError = "",
    cardCvvError = "",
    cardPinError = "",
    onShowGenerator,
    onShowTotpSetup,
  }: {
    entryType: string;
    form: EditFormState;
    cardNumberError?: string;
    cardExpMonthError?: string;
    cardExpYearError?: string;
    cardCvvError?: string;
    cardPinError?: string;
    onShowGenerator: () => void;
    onShowTotpSetup: () => void;
  } = $props();
</script>

<FieldGroup>
  {#if entryType === "login"}
    <LoginEditForm
      bind:editUsername={form.username}
      bind:editPassword={form.password}
      bind:editUrl={form.url}
      bind:editTotp={form.totp}
      {onShowGenerator}
      {onShowTotpSetup}
    />
  {:else if entryType === "password"}
    <PasswordEditForm bind:editPassword={form.password} bind:editUrl={form.url} {onShowGenerator} />
  {:else if entryType === "identity"}
    <IdentityEditForm
      bind:editFirstName={form.firstName}
      bind:editLastName={form.lastName}
      bind:editEmail={form.email}
      bind:editPhone={form.phone}
      bind:editAddress={form.address}
      bind:editDob={form.dob}
    />
  {:else if entryType === "card"}
    <CardEditForm
      bind:editCardHolder={form.cardHolder}
      bind:editCardNumber={form.cardNumber}
      bind:editCardType={form.cardType}
      bind:editCardExpMonth={form.cardExpMonth}
      bind:editCardExpYear={form.cardExpYear}
      bind:editCardCvv={form.cardCvv}
      bind:editCardPin={form.cardPin}
      {cardNumberError}
      {cardExpMonthError}
      {cardExpYearError}
      {cardCvvError}
      {cardPinError}
    />
  {:else if entryType === "software_license"}
    <SoftwareLicenseEditForm
      bind:editLicenseVersion={form.licenseVersion}
      bind:editLicenseKey={form.licenseKey}
      bind:editLicenseLicensedTo={form.licenseLicensedTo}
      bind:editLicenseRegisteredEmail={form.licenseRegisteredEmail}
      bind:editLicenseCompany={form.licenseCompany}
      bind:editLicenseDownloadPage={form.licenseDownloadPage}
      bind:editLicensePublisher={form.licensePublisher}
      bind:editLicenseWebsite={form.licenseWebsite}
      bind:editLicenseRetailPrice={form.licenseRetailPrice}
      bind:editLicenseSupportEmail={form.licenseSupportEmail}
      bind:editLicensePurchaseDate={form.licensePurchaseDate}
      bind:editLicenseOrderNumber={form.licenseOrderNumber}
      bind:editLicenseOrderTotal={form.licenseOrderTotal}
    />
  {:else if entryType === "passport"}
    <PassportEditForm
      bind:editPassportType={form.passportType}
      bind:editPassportIssuingCountry={form.passportIssuingCountry}
      bind:editPassportNumber={form.passportNumber}
      bind:editPassportFullName={form.passportFullName}
      bind:editPassportSex={form.passportSex}
      bind:editPassportNationality={form.passportNationality}
      bind:editPassportIssuingAuthority={form.passportIssuingAuthority}
      bind:editPassportBirthDate={form.passportBirthDate}
      bind:editPassportBirthPlace={form.passportBirthPlace}
      bind:editPassportIssueDate={form.passportIssueDate}
      bind:editPassportExpiryDate={form.passportExpiryDate}
    />
  {:else if entryType === "pgp_key"}
    <PgpKeyEditForm
      bind:editPgpPublicKey={form.pgpPublicKey}
      bind:editPgpPrivateKey={form.pgpPrivateKey}
      bind:editPgpFingerprint={form.pgpFingerprint}
      bind:editPgpKeyId={form.pgpKeyId}
      bind:editPgpUserIds={form.pgpUserIds}
      bind:editPgpAlgorithm={form.pgpAlgorithm}
      bind:editPgpExpiresAt={form.pgpExpiresAt}
    />
  {/if}
</FieldGroup>

<CommonEditFields
  bind:editExpiresAt={form.expiresAt}
  bind:editCustomFields={form.customFields}
  bind:editTags={form.tags}
  bind:editNotes={form.notes}
/>
