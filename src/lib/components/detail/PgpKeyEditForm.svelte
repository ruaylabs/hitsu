<script lang="ts">
  import DetailFieldRow from "./DetailFieldRow.svelte";
  import type { EditFormState } from "./editForm";
  import SecretEditInput from "./SecretEditInput.svelte";

  let { form }: { form: EditFormState } = $props();
</script>

<DetailFieldRow label="Fingerprint">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Fingerprint"
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    bind:value={form.pgpFingerprint}
  />
</DetailFieldRow>
<DetailFieldRow label="Key ID">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Key ID"
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    bind:value={form.pgpKeyId}
  />
</DetailFieldRow>
<DetailFieldRow label="User IDs">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Name <email>, …"
    autocomplete="off"
    bind:value={form.pgpUserIds}
  />
</DetailFieldRow>
<DetailFieldRow label="Algorithm">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Ed25519, RSA 4096, …"
    autocomplete="off"
    bind:value={form.pgpAlgorithm}
  />
</DetailFieldRow>
<DetailFieldRow label="Expires">
  <input
    class="control control--compact edit-input"
    type="date"
    aria-label="Key expiration date"
    autocomplete="off"
    bind:value={form.pgpExpiresAt}
  />
</DetailFieldRow>
<DetailFieldRow label="Public key">
  <textarea
    class="control control--compact edit-textarea"
    placeholder="-----BEGIN PGP PUBLIC KEY BLOCK-----&#10;…&#10;-----END PGP PUBLIC KEY BLOCK-----"
    autocomplete="off"
    autocapitalize="off"
    spellcheck="false"
    bind:value={form.pgpPublicKey}
    rows="6"
  ></textarea>
</DetailFieldRow>
<DetailFieldRow label="Private key">
  <SecretEditInput
    bind:value={form.pgpPrivateKey}
    label="private key"
    placeholder="-----BEGIN PGP PRIVATE KEY BLOCK-----&#10;…&#10;-----END PGP PRIVATE KEY BLOCK-----"
  />
</DetailFieldRow>
