<script lang="ts">
  import * as entriesBridge from "$lib/bridge/entries";
  import type { Entry } from "$lib/bridge/types";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { openHttpUrl } from "$lib/utils/openHttpUrl";
  import Field from "./Field.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import PasswordField from "./PasswordField.svelte";

  let { entry }: { entry: Entry } = $props();
  const license = $derived(entry.softwareLicense);
</script>

{#if license}
  <FieldGroup>
    {#if license.version}
      <Field label="Version" value={license.version} />
    {/if}
    {#if license.hasLicenseKey}
      <PasswordField
        label="License key"
        reveal={() => entriesBridge.entryRevealField(entry.id, "licenseKey")}
        copy={() => clipboard.copySecretField(entry.id, "licenseKey")}
      />
    {/if}
  </FieldGroup>
  <FieldGroup>
    {#if license.licensedTo}
      <Field label="Licensed to" value={license.licensedTo} />
    {/if}
    {#if license.registeredEmail}
      <Field
        label="Registered email"
        value={license.registeredEmail}
        onCopy={() => clipboard.copyPlain(license.registeredEmail!)}
      />
    {/if}
    {#if license.company}
      <Field label="Company" value={license.company} />
    {/if}
  </FieldGroup>
  <FieldGroup>
    {#if license.downloadPage}
      <Field
        label="Download page"
        value={license.downloadPage}
        onOpenUrl={() => openHttpUrl(license.downloadPage!)}
        onCopy={() => clipboard.copyPlain(license.downloadPage!)}
      />
    {/if}
    {#if license.publisher}
      <Field label="Publisher" value={license.publisher} />
    {/if}
    {#if license.website}
      <Field
        label="Website"
        value={license.website}
        onOpenUrl={() => openHttpUrl(license.website!)}
        onCopy={() => clipboard.copyPlain(license.website!)}
      />
    {/if}
    {#if license.retailPrice}
      <Field label="Retail price" value={license.retailPrice} />
    {/if}
    {#if license.supportEmail}
      <Field
        label="Support email"
        value={license.supportEmail}
        onCopy={() => clipboard.copyPlain(license.supportEmail!)}
      />
    {/if}
  </FieldGroup>
  <FieldGroup>
    {#if license.purchaseDate}
      <Field label="Purchase date" value={license.purchaseDate} />
    {/if}
    {#if license.orderNumber}
      <Field label="Order number" value={license.orderNumber} />
    {/if}
    {#if license.orderTotal}
      <Field label="Order total" value={license.orderTotal} />
    {/if}
  </FieldGroup>
{/if}
