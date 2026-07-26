<script lang="ts">
  import { CARD_BRANDS } from "$lib/utils/format";
  import DetailFieldRow from "./DetailFieldRow.svelte";
  import SecretEditInput from "./SecretEditInput.svelte";

  let {
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
  }: {
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
  } = $props();
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
    bind:value={editCardHolder}
  />
</DetailFieldRow>
<DetailFieldRow label="Number" alignStart>
  <div class="card-input-wrap">
    <SecretEditInput
      bind:value={editCardNumber}
      label="card number"
      placeholder="Card number"
      inputmode="numeric"
      pattern="[0-9]*"
      invalid={Boolean(cardNumberError)}
      sanitize={(value: string) => value.replace(/\D/g, "")}
    />
    {#if cardNumberError}
      <span class="control-error">{cardNumberError}</span>
    {/if}
  </div>
</DetailFieldRow>
<DetailFieldRow label="Type">
  <select class="control control--compact control--select" bind:value={editCardType}>
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
      aria-invalid={Boolean(cardExpMonthError)}
      placeholder="MM"
      maxlength="2"
      autocomplete="off"
      bind:value={editCardExpMonth}
      oninput={(e) => { const el = e.currentTarget; el.value = el.value.replace(/\D/g, '').slice(0, 2); editCardExpMonth = el.value; }}
    />
    {#if cardExpMonthError}
      <span class="control-error">{cardExpMonthError}</span>
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
      aria-invalid={Boolean(cardExpYearError)}
      placeholder="YYYY"
      maxlength="4"
      autocomplete="off"
      bind:value={editCardExpYear}
      oninput={(e) => { const el = e.currentTarget; el.value = el.value.replace(/\D/g, ''); editCardExpYear = el.value; }}
    />
    {#if cardExpYearError}
      <span class="control-error">{cardExpYearError}</span>
    {/if}
  </div>
</DetailFieldRow>
<DetailFieldRow label="CVV" alignStart>
  <div class="card-input-wrap">
    <SecretEditInput
      bind:value={editCardCvv}
      label="CVV"
      placeholder="CVV"
      inputmode="numeric"
      pattern="[0-9]*"
      maxlength={4}
      invalid={Boolean(cardCvvError)}
      sanitize={(value: string) => value.replace(/\D/g, "").slice(0, 4)}
    />
    {#if cardCvvError}
      <span class="control-error">{cardCvvError}</span>
    {/if}
  </div>
</DetailFieldRow>
<DetailFieldRow label="PIN" alignStart>
  <div class="card-input-wrap">
    <SecretEditInput
      bind:value={editCardPin}
      label="PIN"
      placeholder="PIN"
      inputmode="numeric"
      pattern="[0-9]*"
      maxlength={12}
      invalid={Boolean(cardPinError)}
      sanitize={(value: string) => value.replace(/\D/g, "").slice(0, 12)}
    />
    {#if cardPinError}
      <span class="control-error">{cardPinError}</span>
    {/if}
  </div>
</DetailFieldRow>

<style>
  .card-input-wrap {
    display: flex;
    flex-direction: column;
    gap: 4px;
    width: 100%;
  }
  .control-error {
    margin: 0;
    font-size: 12px;
    color: var(--error);
  }
</style>
