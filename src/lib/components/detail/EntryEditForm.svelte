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
    form,
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

<!-- Each per-type form binds its inputs directly to properties of the shared
     `form` object; only the per-type extras (callbacks, card errors) are
     threaded through here. -->
<FieldGroup>
  {#if entryType === "login"}
    <LoginEditForm {form} {onShowGenerator} {onShowTotpSetup} />
  {:else if entryType === "password"}
    <PasswordEditForm {form} {onShowGenerator} />
  {:else if entryType === "identity"}
    <IdentityEditForm {form} />
  {:else if entryType === "card"}
    <CardEditForm
      {form}
      {cardNumberError}
      {cardExpMonthError}
      {cardExpYearError}
      {cardCvvError}
      {cardPinError}
    />
  {:else if entryType === "software_license"}
    <SoftwareLicenseEditForm {form} />
  {:else if entryType === "passport"}
    <PassportEditForm {form} />
  {:else if entryType === "pgp_key"}
    <PgpKeyEditForm {form} />
  {/if}
</FieldGroup>

<CommonEditFields {form} />
