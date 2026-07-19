<script lang="ts">
  import { estimateStrength } from "$lib/utils/passwordStrength";
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";
  import PasswordStrengthMeter from "./PasswordStrengthMeter.svelte";

  let {
    title = "Enter master password",
    vaultPath = "",
    confirmLabel = "Unlock",
    showConfirm = true,
    showCancel = true,
    errorMessage = "",
    transparentOverlay = false,
    confirm = false,
    confirmLabel2 = "Confirm password",
    showStrength = false,
    /** Minimum strength level (0–4) required to enable the confirm button.
     *  Only applies when `showStrength` is true. See passwordStrength.ts levels. */
    minStrength = 1,
    onconfirm,
    oncancel,
  }: {
    title?: string;
    /** Path of the vault being unlocked, shown above the password field. */
    vaultPath?: string;
    confirmLabel?: string;
    showConfirm?: boolean;
    showCancel?: boolean;
    errorMessage?: string;
    transparentOverlay?: boolean;
    /** Show a second "confirm password" field that must match before submit. */
    confirm?: boolean;
    confirmLabel2?: string;
    /** Show a strength meter under the password field. */
    showStrength?: boolean;
    /** Minimum strength level (0–4) required to enable the confirm button.
     *  Only applies when `showStrength` is true. See passwordStrength.ts levels. */
    minStrength?: number;
    onconfirm?: (password: string) => void;
    oncancel?: () => void;
  } = $props();

  let password = $state("");
  let confirmPassword = $state("");
  let localError = $state("");

  let displayError = $derived(localError || errorMessage);
  let strengthOk = $derived(!showStrength || estimateStrength(password).level >= minStrength);
  let canSubmit = $derived(
    password.length > 0 && (!confirm || confirmPassword.length > 0) && strengthOk,
  );

  function submit() {
    if (!password) {
      localError = "Password is required";
      return;
    }
    if (confirm && confirmPassword !== password) {
      localError = "Passwords do not match";
      return;
    }
    localError = "";
    onconfirm?.(password);
  }
</script>

<Dialog
  {title}
  onclose={showCancel ? oncancel : undefined}
  onconfirm={submit}
  size="md"
  transparent={transparentOverlay}
  showFooter={showConfirm}
  closeLabel="Cancel"
>
  {#snippet children()}
    <div class="password-field">
      {#if vaultPath}
        <!-- The &lrm; keeps the RTL truncation below from visually moving the
             path's leading "/" to the end. -->
        <span class="vault-path" title={vaultPath}>&lrm;{vaultPath}</span>
      {/if}
      <label class="control-label" for="master-pw">Master password</label>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        id="master-pw"
        type="password"
        class="control control--mono"
        aria-invalid={Boolean(displayError)}
        placeholder="Enter master password"
        autofocus
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        bind:value={password}
        oninput={() => { localError = ""; }}
      />
      {#if displayError}
        <span class="control-error">{displayError}</span>
      {/if}

      {#if showStrength}
        <PasswordStrengthMeter {password} showWhenEmpty />
      {/if}

      {#if confirm}
        <label class="control-label" for="master-pw-confirm">{confirmLabel2}</label>
        <input
          id="master-pw-confirm"
          type="password"
          class="control control--mono"
          aria-invalid={Boolean(displayError)}
          placeholder="Re-enter password"
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
          bind:value={confirmPassword}
          oninput={() => { localError = ""; }}
        />
      {/if}
    </div>
  {/snippet}

  {#snippet footer()}
    {#if showCancel}
      <Button onclick={oncancel}>Cancel</Button>
    {/if}
    <Button variant="primary" onclick={submit} disabled={!canSubmit}>{confirmLabel}</Button>
  {/snippet}
</Dialog>

<style>
  .password-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .vault-path {
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--text-muted);
    /* Long paths truncate from the start so the filename stays visible. */
    direction: rtl;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-bottom: 4px;
  }
</style>
