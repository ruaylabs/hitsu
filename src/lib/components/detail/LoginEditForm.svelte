<script lang="ts">
  import Icon from "../ui/Icon.svelte";
  import PasswordStrengthMeter from "../ui/PasswordStrengthMeter.svelte";
  import DetailFieldRow from "./DetailFieldRow.svelte";
  import type { EditFormState } from "./editForm";
  import SecretEditInput from "./SecretEditInput.svelte";

  let {
    form,
    onShowGenerator,
    onShowTotpSetup,
  }: {
    form: EditFormState;
    onShowGenerator: () => void;
    onShowTotpSetup: () => void;
  } = $props();
</script>

<DetailFieldRow label="Username">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Username"
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    bind:value={form.username}
  />
</DetailFieldRow>
<DetailFieldRow label="Password">
  <div class="password-edit-col">
    <div class="password-edit-row">
      <SecretEditInput bind:value={form.password} label="password" placeholder="Password" />
      <button
        class="generate-btn"
        onclick={onShowGenerator}
        aria-label="Generate password"
        title="Generate password"
      >
        <Icon name="bolt" size={14} />
      </button>
    </div>
    <PasswordStrengthMeter password={form.password} showWhenEmpty />
  </div>
</DetailFieldRow>
<DetailFieldRow label="URL">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="URL"
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    bind:value={form.url}
  />
</DetailFieldRow>
<DetailFieldRow label="TOTP">
  <div class="totp-edit-wrap">
    <SecretEditInput bind:value={form.totp} label="TOTP URI" placeholder="otpauth:// URI" />
    <button
      class="totp-setup-btn-small"
      onclick={onShowTotpSetup}
      aria-label="Setup TOTP from seed"
      title="Setup TOTP from seed"
    >
      <Icon name="key" size={14} />
    </button>
  </div>
</DetailFieldRow>

<style>
  .password-edit-col {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    width: 100%;
  }
  .password-edit-row {
    display: flex;
    gap: var(--space-1);
    align-items: flex-start;
  }
  .password-edit-row :global(input) {
    flex: 1;
  }
  .generate-btn,
  .totp-setup-btn-small {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--icon-button-size);
    height: var(--icon-button-size);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--surface-1);
    color: var(--text-secondary);
    flex-shrink: 0;
  }
  .generate-btn:hover,
  .totp-setup-btn-small:hover {
    background: var(--surface-hover);
  }
  .totp-edit-wrap {
    display: flex;
    gap: var(--space-1);
    align-items: flex-start;
    width: 100%;
  }
  .totp-edit-wrap :global(input) {
    flex: 1;
  }
</style>
