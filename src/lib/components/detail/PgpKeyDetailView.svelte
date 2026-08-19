<script lang="ts">
  import * as entriesBridge from "$lib/bridge/entries";
  import type { Entry } from "$lib/bridge/types";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import Field from "./Field.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import PasswordField from "./PasswordField.svelte";

  let { entry }: { entry: Entry } = $props();
  const pgp = $derived(entry.pgpKey);
</script>

{#if pgp}
  <FieldGroup>
    {#if pgp.fingerprint}
      <Field
        label="Fingerprint"
        value={pgp.fingerprint}
        mono={true}
        onCopy={() => clipboard.copyPlain(pgp.fingerprint!)}
      />
    {/if}
    {#if pgp.keyId}
      <Field
        label="Key ID"
        value={pgp.keyId}
        mono={true}
        onCopy={() => clipboard.copyPlain(pgp.keyId!)}
      />
    {/if}
    {#if pgp.userIds}
      <Field label="User IDs" value={pgp.userIds} />
    {/if}
    {#if pgp.algorithm}
      <Field label="Algorithm" value={pgp.algorithm} />
    {/if}
    {#if pgp.expiresAt}
      <Field label="Expires" value={pgp.expiresAt} />
    {/if}
  </FieldGroup>
  {#if pgp.publicKey}
    <FieldGroup>
      <Field
        label="Public key"
        value={pgp.publicKey}
        mono={true}
        onCopy={() => clipboard.copyPlain(pgp.publicKey!)}
      />
    </FieldGroup>
  {/if}
  {#if pgp.hasPrivateKey}
    <FieldGroup>
      <PasswordField
        label="Private key"
        reveal={() => entriesBridge.entryRevealField(entry.id, "pgpPrivateKey")}
        copy={() => clipboard.copySecretField(entry.id, "pgpPrivateKey")}
      />
    </FieldGroup>
  {/if}
{/if}
