<script lang="ts">
  import { CARD_BRANDS } from "$lib/utils/format";
  import type { CardErrors } from "./cardValidation";
  import DetailFieldRow from "./DetailFieldRow.svelte";
  import type { EditFormState } from "./editForm";
  import SecretEditInput from "./SecretEditInput.svelte";

  let { form, cardErrors }: { form: EditFormState; cardErrors: CardErrors } = $props();
</script>

<DetailFieldRow label="Holder">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Card holder"
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    bind:value={form.cardHolder}
  />
</DetailFieldRow>
<DetailFieldRow label="Number" alignStart>
  <div class="card-input-wrap">
    <SecretEditInput
      bind:value={form.cardNumber}
      label="card number"
      placeholder="Card number"
      inputmode="numeric"
      pattern="[0-9]*"
      invalid={Boolean(cardErrors.cardNumber)}
      sanitize={(value: string) => value.replace(/\D/g, "")}
    />
    {#if cardErrors.cardNumber}
      <span class="control-error">{cardErrors.cardNumber}</span>
    {/if}
  </div>
</DetailFieldRow>
<DetailFieldRow label="Type">
  <select class="control control--compact control--select" bind:value={form.cardType}>
    <option value="">Select brand</option>
    {#each Object.entries(CARD_BRANDS) as [ key, name ]}
      <option value={key}>{name}</option>
    {/each}
  </select>
</DetailFieldRow>
<DetailFieldRow label="Exp month" alignStart>
  <div class="card-input-wrap">
    <input
      class="control control--compact edit-input"
      type="text"
      inputmode="numeric"
      pattern="[0-9]*"
      aria-invalid={Boolean(cardErrors.cardExpMonth)}
      placeholder="MM"
      maxlength="2"
      autocomplete="off"
      bind:value={form.cardExpMonth}
      oninput={(e) => { const el = e.currentTarget; el.value = el.value.replace(/\D/g, '').slice(0, 2); form.cardExpMonth = el.value; }}
    />
    {#if cardErrors.cardExpMonth}
      <span class="control-error">{cardErrors.cardExpMonth}</span>
    {/if}
  </div>
</DetailFieldRow>
<DetailFieldRow label="Exp year" alignStart>
  <div class="card-input-wrap">
    <input
      class="control control--compact edit-input"
      type="text"
      inputmode="numeric"
      pattern="[0-9]*"
      aria-invalid={Boolean(cardErrors.cardExpYear)}
      placeholder="YYYY"
      maxlength="4"
      autocomplete="off"
      bind:value={form.cardExpYear}
      oninput={(e) => { const el = e.currentTarget; el.value = el.value.replace(/\D/g, ''); form.cardExpYear = el.value; }}
    />
    {#if cardErrors.cardExpYear}
      <span class="control-error">{cardErrors.cardExpYear}</span>
    {/if}
  </div>
</DetailFieldRow>
<DetailFieldRow label="CVV" alignStart>
  <div class="card-input-wrap">
    <SecretEditInput
      bind:value={form.cardCvv}
      label="CVV"
      placeholder="CVV"
      inputmode="numeric"
      pattern="[0-9]*"
      maxlength={4}
      invalid={Boolean(cardErrors.cardCvv)}
      sanitize={(value: string) => value.replace(/\D/g, "").slice(0, 4)}
    />
    {#if cardErrors.cardCvv}
      <span class="control-error">{cardErrors.cardCvv}</span>
    {/if}
  </div>
</DetailFieldRow>
<DetailFieldRow label="PIN" alignStart>
  <div class="card-input-wrap">
    <SecretEditInput
      bind:value={form.cardPin}
      label="PIN"
      placeholder="PIN"
      inputmode="numeric"
      pattern="[0-9]*"
      maxlength={12}
      invalid={Boolean(cardErrors.cardPin)}
      sanitize={(value: string) => value.replace(/\D/g, "").slice(0, 12)}
    />
    {#if cardErrors.cardPin}
      <span class="control-error">{cardErrors.cardPin}</span>
    {/if}
  </div>
</DetailFieldRow>

<style>
  .card-input-wrap {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    width: 100%;
  }
  .control-error {
    font-size: var(--text-sm);
    color: var(--danger);
  }
</style>
