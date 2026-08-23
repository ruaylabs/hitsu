<script lang="ts">
  import * as entriesBridge from "$lib/bridge/entries";
  import type { SecretField } from "$lib/bridge/types";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { openHttpUrl } from "$lib/utils/openHttpUrl";
  import Field from "./Field.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import PasswordField from "./PasswordField.svelte";

  /** A secret field rendered as a masked PasswordField (reveal/copy on demand). */
  interface SecretFieldSpec {
    field: SecretField;
    /** Backend-masked hint to show while hidden (e.g. "•••• 1234"). */
    masked?: string;
    /** Transform the revealed plaintext before display (e.g. card grouping). */
    format?: (value: string) => string;
    showStrength?: boolean;
  }

  /** A plain field rendered as a Field row; omitted from display when `value`
   *  is empty. `copy` adds a copy button, `link` renders the value as an
   *  http(s) link. */
  interface DetailFieldSpec {
    label: string;
    value?: string;
    mono?: boolean;
    copy?: boolean;
    link?: boolean;
    secret?: SecretFieldSpec;
  }

  type DetailGroupSpec = DetailFieldSpec[];

  let {
    entryId,
    groups,
  }: {
    entryId: string;
    groups: DetailGroupSpec[];
  } = $props();

  // Groups whose every field is empty render nothing at all — an empty
  // FieldGroup would still draw its bordered strip (see LoginDetailView).
  const visibleGroups = $derived(
    groups.filter((group) => group.some((field) => field.secret || field.value)),
  );
</script>

{#each visibleGroups as group, groupIndex (groupIndex)}
  <FieldGroup>
    {#each group as field, fieldIndex (fieldIndex)}
      {#if field.secret}
        <PasswordField
          label={field.label}
          masked={field.secret.masked}
          reveal={async () => {
            let value = await entriesBridge.entryRevealField(entryId, field.secret!.field);
            return field.secret!.format ? field.secret!.format(value) : value;
          }}
          copy={() => clipboard.copySecretField(entryId, field.secret!.field)}
          showStrength={field.secret.showStrength}
        />
      {:else}
        <Field
          label={field.label}
          value={field.value ?? ""}
          mono={field.mono}
          onOpenUrl={field.link && field.value ? () => openHttpUrl(field.value!) : undefined}
          onCopy={field.copy && field.value ? () => clipboard.copyPlain(field.value!) : undefined}
        />
      {/if}
    {/each}
  </FieldGroup>
{/each}
