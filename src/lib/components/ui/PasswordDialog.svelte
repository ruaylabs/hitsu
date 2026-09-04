<script lang="ts">
  import { tick } from "svelte";
  import { errorMessage as getErrorMessage } from "$lib/utils/errorMessage";
  import { estimateStrength } from "$lib/utils/passwordStrength";
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";
  import Icon from "./Icon.svelte";
  import PasswordStrengthMeter from "./PasswordStrengthMeter.svelte";

  let {
    title = "Enter master password",
    vaultPath = "",
    confirmLabel = "Unlock",
    pendingLabel,
    alternateLabel,
    alternatePendingLabel,
    alternateIcon,
    onalternate,
    showConfirm = true,
    showCancel = true,
    errorMessage = "",
    transparentOverlay = false,
    confirm = false,
    confirmLabel2 = "Confirm password",
    showStrength = false,
    showRecoveryWarning = false,
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
    pendingLabel?: string;
    /** Optional action shown above the password field, such as biometric unlock. */
    alternateLabel?: string;
    alternatePendingLabel?: string;
    alternateIcon?: string;
    onalternate?: () => void | Promise<void>;
    showConfirm?: boolean;
    showCancel?: boolean;
    errorMessage?: string;
    transparentOverlay?: boolean;
    /** Show a second "confirm password" field that must match before submit. */
    confirm?: boolean;
    confirmLabel2?: string;
    /** Show a strength meter under the password field. */
    showStrength?: boolean;
    /** Explain that the master password cannot be recovered. */
    showRecoveryWarning?: boolean;
    /** Minimum strength level (0–4) required to enable the confirm button.
     *  Only applies when `showStrength` is true. See passwordStrength.ts levels. */
    minStrength?: number;
    onconfirm?: (password: string) => void | Promise<void>;
    oncancel?: () => void;
  } = $props();

  let password = $state("");
  let confirmPassword = $state("");
  let localError = $state("");
  let showPassword = $state(false);
  let showConfirmPassword = $state(false);
  let capsLockOn = $state(false);
  let focusedField = $state<"password" | "confirm" | null>(null);
  let busy = $state(false);
  let alternateBusy = $state(false);
  let passwordInput: HTMLInputElement | undefined;

  const DEFAULT_PENDING_LABELS: Record<string, string> = {
    Unlock: "Unlocking…",
    Open: "Opening…",
    Create: "Creating…",
    Change: "Changing…",
  };
  const STRENGTH_LEVEL_LABELS = ["Very weak", "Weak", "Fair", "Good", "Strong"];

  let displayError = $derived(localError || errorMessage);
  let submitLabel = $derived(
    busy ? (pendingLabel ?? DEFAULT_PENDING_LABELS[confirmLabel] ?? "Working…") : confirmLabel,
  );
  let strengthOk = $derived(!showStrength || estimateStrength(password).level >= minStrength);
  let strengthHelp = $derived(
    strengthOk
      ? "Use a long, unique passphrase and store it in a separate secure location."
      : `${confirmLabel} is disabled until password strength is ${STRENGTH_LEVEL_LABELS[minStrength]} or better. Try a longer, unique passphrase.`,
  );
  let working = $derived(busy || alternateBusy);
  let canSubmit = $derived(
    !working && password.length > 0 && (!confirm || confirmPassword.length > 0) && strengthOk,
  );

  function updateCapsLock(event: KeyboardEvent) {
    capsLockOn = event.getModifierState("CapsLock");
  }

  async function submit() {
    if (working) return;
    if (!password) {
      localError = "Password is required";
      return;
    }
    if (confirm && confirmPassword !== password) {
      localError = "Passwords do not match";
      return;
    }
    localError = "";
    busy = true;
    try {
      await onconfirm?.(password);
    } catch (error) {
      localError = getErrorMessage(error);
    } finally {
      busy = false;
    }
  }

  async function runAlternate() {
    if (working || !onalternate) return;
    localError = "";
    alternateBusy = true;
    try {
      await onalternate();
    } catch (error) {
      localError = getErrorMessage(error);
    } finally {
      alternateBusy = false;
      await tick();
      passwordInput?.focus();
    }
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
  closeDisabled={working}
>
  {#snippet children()}
    <div class="password-field" aria-busy={working}>
      {#if vaultPath}
        <!-- The &lrm; keeps the RTL truncation below from visually moving the
             path's leading "/" to the end. -->
        <span class="vault-path" title={vaultPath}>&lrm;{vaultPath}</span>
      {/if}
      {#if showRecoveryWarning}
        <div class="recovery-warning" role="note">
          <Icon name="alert-triangle" size={16} />
          <span>
            <strong>Hitsu cannot recover this password if you forget it.</strong>
            Store a backup in another secure location.
          </span>
        </div>
      {/if}
      {#if alternateLabel && onalternate}
        <Button type="button" variant="primary" size="md" disabled={working} onclick={runAlternate}>
          {#if alternateBusy}
            <span class="busy-spinner" aria-hidden="true"></span>
          {:else if alternateIcon}
            <Icon name={alternateIcon} size={18} />
          {/if}
          {alternateBusy ? (alternatePendingLabel ?? "Working…") : alternateLabel}
        </Button>
        <div class="alternate-divider"><span>or use your master password</span></div>
      {/if}
      <label class="control-label" for="master-pw">Master password</label>
      <div class="password-input-wrap">
        <!-- svelte-ignore a11y_autofocus -->
        <input
          id="master-pw"
          type={showPassword ? "text" : "password"}
          class="control control--mono"
          aria-invalid={Boolean(displayError)}
          placeholder="Enter master password"
          autofocus
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
          disabled={working}
          bind:this={passwordInput}
          bind:value={password}
          oninput={() => { localError = ""; }}
          onkeydown={updateCapsLock}
          onkeyup={updateCapsLock}
          onfocus={() => (focusedField = "password")}
          onblur={() => (focusedField = null)}
        />
        <button
          type="button"
          class="reveal-button"
          aria-label={showPassword ? "Hide master password" : "Show master password"}
          aria-pressed={showPassword}
          disabled={working}
          onclick={() => (showPassword = !showPassword)}
        >
          <Icon name={showPassword ? "eye-off" : "eye"} size={16} />
        </button>
      </div>
      {#if capsLockOn && focusedField === "password"}
        <span class="caps-lock-warning" role="status">
          <Icon name="alert-triangle" size={13} />
          Caps Lock is on
        </span>
      {/if}
      {#if displayError}
        <span class="control-error">{displayError}</span>
      {/if}

      {#if showStrength}
        <PasswordStrengthMeter {password} showWhenEmpty />
        <p class="strength-help" aria-live="polite">{strengthHelp}</p>
      {/if}

      {#if confirm}
        <label class="control-label" for="master-pw-confirm">{confirmLabel2}</label>
        <div class="password-input-wrap">
          <input
            id="master-pw-confirm"
            type={showConfirmPassword ? "text" : "password"}
            class="control control--mono"
            aria-invalid={Boolean(displayError)}
            placeholder="Re-enter password"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            disabled={working}
            bind:value={confirmPassword}
            oninput={() => { localError = ""; }}
            onkeydown={updateCapsLock}
            onkeyup={updateCapsLock}
            onfocus={() => (focusedField = "confirm")}
            onblur={() => (focusedField = null)}
          />
          <button
            type="button"
            class="reveal-button"
            aria-label={showConfirmPassword ? "Hide confirmation password" : "Show confirmation password"}
            aria-pressed={showConfirmPassword}
            disabled={working}
            onclick={() => (showConfirmPassword = !showConfirmPassword)}
          >
            <Icon name={showConfirmPassword ? "eye-off" : "eye"} size={16} />
          </button>
        </div>
        {#if capsLockOn && focusedField === "confirm"}
          <span class="caps-lock-warning" role="status">
            <Icon name="alert-triangle" size={13} />
            Caps Lock is on
          </span>
        {/if}
      {/if}
    </div>
  {/snippet}

  {#snippet footer()}
    {#if showCancel}
      <Button onclick={oncancel} disabled={working}>Cancel</Button>
    {/if}
    <Button variant="primary" onclick={submit} disabled={!canSubmit}>
      {#if busy}
        <span class="busy-spinner" aria-hidden="true"></span>
      {/if}
      {submitLabel}
    </Button>
  {/snippet}
</Dialog>

<style>
  .busy-spinner {
    width: 12px;
    height: 12px;
    border: 1.5px solid currentColor;
    border-right-color: transparent;
    border-radius: 50%;
    animation: busy-spin 0.7s linear infinite;
  }

  @keyframes busy-spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .busy-spinner {
      animation: none;
    }
  }

  .alternate-divider {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 4px 0;
    color: var(--text-muted);
    font-size: var(--text-sm);
  }

  .alternate-divider::before,
  .alternate-divider::after {
    height: 0.5px;
    background: var(--border);
    content: "";
    flex: 1;
  }

  .recovery-warning {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin-bottom: 6px;
    padding: 10px 12px;
    color: var(--text-secondary);
    background: var(--surface-1);
    border: 0.5px solid var(--warning);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    line-height: 1.45;
  }

  .recovery-warning :global(.ti) {
    flex-shrink: 0;
    margin-top: 1px;
    color: var(--warning);
  }

  .recovery-warning strong {
    display: block;
    color: var(--text-primary);
    font-weight: 600;
  }

  .strength-help {
    color: var(--text-muted);
    font-size: var(--text-sm);
    line-height: 1.4;
  }

  .password-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .password-input-wrap {
    position: relative;
  }

  .password-input-wrap .control {
    padding-right: 38px;
  }

  .reveal-button {
    position: absolute;
    top: 50%;
    right: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    transform: translateY(-50%);
  }

  .reveal-button:hover,
  .reveal-button:focus-visible {
    color: var(--text-primary);
    background: var(--border);
  }

  .reveal-button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .caps-lock-warning {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--warning);
    font-size: var(--text-sm);
  }

  .vault-path {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
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
