<script lang="ts">
  import type { CustomField } from "$lib/bridge/types";
  import FieldGroup from "./FieldGroup.svelte";
  import LoginEditForm from "./LoginEditForm.svelte";
  import PasswordEditForm from "./PasswordEditForm.svelte";
  import IdentityEditForm from "./IdentityEditForm.svelte";
  import CardEditForm from "./CardEditForm.svelte";
  import SoftwareLicenseEditForm from "./SoftwareLicenseEditForm.svelte";
  import PassportEditForm from "./PassportEditForm.svelte";
  import CommonEditFields from "./CommonEditFields.svelte";

  let {
    entryType,
    // login
    editUsername = $bindable(""),
    editPassword = $bindable(""),
    editUrl = $bindable(""),
    editTotp = $bindable(""),
    // identity
    editFirstName = $bindable(""),
    editLastName = $bindable(""),
    editEmail = $bindable(""),
    editPhone = $bindable(""),
    editAddress = $bindable(""),
    editDob = $bindable(""),
    // card
    editCardHolder = $bindable(""),
    editCardNumber = $bindable(""),
    editCardType = $bindable(""),
    editCardExpMonth = $bindable(""),
    editCardExpYear = $bindable(""),
    editCardCvv = $bindable(""),
    editCardPin = $bindable(""),
    cardNumberError = "",
    cardExpMonthError = "",
    cardExpYearError = "",
    cardCvvError = "",
    cardPinError = "",
    // software license
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
    // passport
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
    // common
    editExpiresAt = $bindable(""),
    editCustomFields = $bindable([] as CustomField[]),
    editTags = $bindable([] as string[]),
    editNotes = $bindable(""),
    // callbacks
    onShowGenerator,
    onShowTotpSetup,
  }: {
    entryType: string;
    editUsername?: string;
    editPassword?: string;
    editUrl?: string;
    editTotp?: string;
    editFirstName?: string;
    editLastName?: string;
    editEmail?: string;
    editPhone?: string;
    editAddress?: string;
    editDob?: string;
    editCardHolder?: string;
    editCardNumber?: string;
    editCardType?: string;
    editCardExpMonth?: string;
    editCardExpYear?: string;
    editCardCvv?: string;
    editCardPin?: string;
    cardNumberError?: string;
    cardExpMonthError?: string;
    cardExpYearError?: string;
    cardCvvError?: string;
    cardPinError?: string;
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
    editExpiresAt?: string;
    editCustomFields?: CustomField[];
    editTags?: string[];
    editNotes?: string;
    onShowGenerator: () => void;
    onShowTotpSetup: () => void;
  } = $props();
</script>

<FieldGroup>
  {#if entryType === "login"}
    <LoginEditForm
      bind:editUsername
      bind:editPassword
      bind:editUrl
      bind:editTotp
      {onShowGenerator}
      {onShowTotpSetup}
    />
  {:else if entryType === "password"}
    <PasswordEditForm bind:editPassword bind:editUrl {onShowGenerator} />
  {:else if entryType === "identity"}
    <IdentityEditForm
      bind:editFirstName
      bind:editLastName
      bind:editEmail
      bind:editPhone
      bind:editAddress
      bind:editDob
    />
  {:else if entryType === "card"}
    <CardEditForm
      bind:editCardHolder
      bind:editCardNumber
      bind:editCardType
      bind:editCardExpMonth
      bind:editCardExpYear
      bind:editCardCvv
      bind:editCardPin
      {cardNumberError}
      {cardExpMonthError}
      {cardExpYearError}
      {cardCvvError}
      {cardPinError}
    />
  {:else if entryType === "software_license"}
    <SoftwareLicenseEditForm
      bind:editLicenseVersion
      bind:editLicenseKey
      bind:editLicenseLicensedTo
      bind:editLicenseRegisteredEmail
      bind:editLicenseCompany
      bind:editLicenseDownloadPage
      bind:editLicensePublisher
      bind:editLicenseWebsite
      bind:editLicenseRetailPrice
      bind:editLicenseSupportEmail
      bind:editLicensePurchaseDate
      bind:editLicenseOrderNumber
      bind:editLicenseOrderTotal
    />
  {:else if entryType === "passport"}
    <PassportEditForm
      bind:editPassportType
      bind:editPassportIssuingCountry
      bind:editPassportNumber
      bind:editPassportFullName
      bind:editPassportSex
      bind:editPassportNationality
      bind:editPassportIssuingAuthority
      bind:editPassportBirthDate
      bind:editPassportBirthPlace
      bind:editPassportIssueDate
      bind:editPassportExpiryDate
    />
  {/if}
</FieldGroup>

<CommonEditFields bind:editExpiresAt bind:editCustomFields bind:editTags bind:editNotes />
