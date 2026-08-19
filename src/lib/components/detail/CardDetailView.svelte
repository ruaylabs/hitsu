<script lang="ts">
  import * as entriesBridge from "$lib/bridge/entries";
  import type { Entry } from "$lib/bridge/types";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { cardBrandName, formatCardNumber } from "$lib/utils/format";
  import Field from "./Field.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import PasswordField from "./PasswordField.svelte";

  let { entry }: { entry: Entry } = $props();
  const card = $derived(entry.card);
</script>

{#if card}
  <FieldGroup>
    {#if card.type}
      <Field label="Type" value={cardBrandName(card.type)} />
    {/if}
    {#if card.holder}
      <Field label="Holder" value={card.holder} />
    {/if}
    {#if card.hasNumber}
      <!-- Masked by default like the other secrets: the full PAN only
           crosses IPC on an explicit reveal, never on selection. -->
      <PasswordField
        label="Number"
        masked={card.numberMasked || undefined}
        reveal={async () =>
          formatCardNumber(await entriesBridge.entryRevealField(entry.id, "cardNumber"), card.type)}
        copy={() => clipboard.copySecretField(entry.id, "cardNumber")}
      />
    {/if}
    {#if card.expMonth && card.expYear}
      <Field label="Expires" value={`${String(card.expMonth).padStart(2, "0")}/${card.expYear}`} />
    {/if}
    {#if card.hasCvv}
      <PasswordField
        label="CVV"
        reveal={() => entriesBridge.entryRevealField(entry.id, "cardCvv")}
        copy={() => clipboard.copySecretField(entry.id, "cardCvv")}
      />
    {/if}
    {#if card.hasPin}
      <PasswordField
        label="PIN"
        reveal={() => entriesBridge.entryRevealField(entry.id, "cardPin")}
        copy={() => clipboard.copySecretField(entry.id, "cardPin")}
      />
    {/if}
  </FieldGroup>
{/if}
