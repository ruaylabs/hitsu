<script lang="ts">
  import type { Entry } from "$lib/bridge/types";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import Field from "./Field.svelte";
  import FieldGroup from "./FieldGroup.svelte";

  let { entry }: { entry: Entry } = $props();
  const identity = $derived(entry.identity);
</script>

{#if identity}
  <FieldGroup>
    {#if identity.firstName}
      <Field label="First name" value={identity.firstName} />
    {/if}
    {#if identity.lastName}
      <Field label="Last name" value={identity.lastName} />
    {/if}
    {#if identity.email}
      <Field
        label="Email"
        value={identity.email}
        onCopy={() => clipboard.copyPlain(identity.email!)}
      />
    {/if}
    {#if identity.phone}
      <Field
        label="Phone"
        value={identity.phone}
        onCopy={() => clipboard.copyPlain(identity.phone!)}
      />
    {/if}
    {#if identity.address}
      <Field label="Address" value={identity.address} />
    {/if}
    {#if identity.dob}
      <Field label="Date of birth" value={identity.dob} />
    {/if}
  </FieldGroup>
{/if}
