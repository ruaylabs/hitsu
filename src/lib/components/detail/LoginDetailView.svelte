<script lang="ts">
  import * as entriesBridge from "$lib/bridge/entries";
  import type { Entry } from "$lib/bridge/types";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { openHttpUrl } from "$lib/utils/openHttpUrl";
  import Field from "./Field.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import PasswordField from "./PasswordField.svelte";
  import TOTPField from "./TOTPField.svelte";

  let { entry }: { entry: Entry } = $props();

  // Credential-less notes (and empty passwords) render no group at all —
  // an empty FieldGroup would still draw its bordered strip.
  let hasCredentialFields = $derived(
    Boolean(entry.username) || entry.hasPassword || Boolean(entry.url),
  );
</script>

{#if entry.type === "login" && entry.hasTotp}
  <TOTPField entryId={entry.id} />
{/if}
{#if hasCredentialFields}
  <FieldGroup>
    {#if entry.username}
      <Field
        label="Username"
        value={entry.username}
        onCopy={() => clipboard.copyPlain(entry.username!)}
      />
    {/if}
    {#if entry.hasPassword}
      <PasswordField
        label="Password"
        reveal={() => entriesBridge.entryRevealField(entry.id, "password")}
        copy={() => clipboard.copySecretField(entry.id, "password")}
        showStrength
      />
    {/if}
    {#if entry.url}
      <Field
        label="URL"
        value={entry.url}
        mono={false}
        onOpenUrl={() => openHttpUrl(entry.url!)}
        onCopy={() => clipboard.copyPlain(entry.url!)}
      />
    {/if}
  </FieldGroup>
{/if}
