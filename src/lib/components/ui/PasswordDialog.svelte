<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    title = "Enter master password",
    confirmLabel = "Unlock",
    showConfirm = true,
    showCancel = true,
    errorMessage = "",
    transparentOverlay = false,
    confirm = false,
    confirmLabel2 = "Confirm password",
    onconfirm,
    oncancel,
  }: {
    title?: string;
    confirmLabel?: string;
    showConfirm?: boolean;
    showCancel?: boolean;
    errorMessage?: string;
    transparentOverlay?: boolean;
    /** Show a second "confirm password" field that must match before submit. */
    confirm?: boolean;
    confirmLabel2?: string;
    onconfirm?: (password: string) => void;
    oncancel?: () => void;
  } = $props();

  let password = $state("");
  let confirmPassword = $state("");
  let localError = $state("");

  let displayError = $derived(localError || errorMessage);
  let canSubmit = $derived(password.length > 0 && (!confirm || confirmPassword.length > 0));

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

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      submit();
    } else if (e.key === "Escape") {
      oncancel?.();
    }
  }
</script>

<div
  class="dialog-overlay"
  class:transparent={transparentOverlay}
  onclick={oncancel}
  onkeydown={onKeydown}
  role="dialog"
  aria-label={title}
  tabindex="-1"
>
  <div
    class="dialog-pane"
    onclick={(e) => e.stopPropagation()}
    onkeydown={onKeydown}
    role="dialog"
    tabindex="-1"
  >
    <header class="dialog-header">
      <h2 class="dialog-title">{title}</h2>
      {#if showCancel}
        <button class="dialog-close" onclick={oncancel} aria-label="Cancel" title="Close">
          <Icon name="x" size={16} />
        </button>
      {/if}
    </header>

    <div class="dialog-body">
      <div class="password-field">
        <label class="input-label" for="master-pw">Master password</label>
        <!-- svelte-ignore a11y_autofocus -->
        <input
          id="master-pw"
          type="password"
          class="password-input"
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
          <span class="input-error">{displayError}</span>
        {/if}

        {#if confirm}
          <label class="input-label" for="master-pw-confirm">{confirmLabel2}</label>
          <input
            id="master-pw-confirm"
            type="password"
            class="password-input"
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
    </div>

    {#if showConfirm}
      <footer class="dialog-footer">
        {#if showCancel}
          <button class="btn btn-cancel" onclick={oncancel}>Cancel</button>
        {/if}
        <button class="btn btn-confirm" onclick={submit} disabled={!canSubmit}>
          {confirmLabel}
        </button>
      </footer>
    {/if}
  </div>
</div>

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .dialog-overlay.transparent {
    background: transparent;
    pointer-events: none;
  }

  .dialog-overlay.transparent .dialog-pane {
    pointer-events: auto;
  }

  .dialog-pane {
    width: 380px;
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-card);
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 0.5px solid var(--border);
  }

  .dialog-title {
    font-size: 15px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .dialog-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }

  .dialog-close:hover {
    background: var(--border);
  }

  .dialog-body {
    padding: 20px;
  }

  .password-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .input-label {
    font-size: 11px;
    color: var(--text-muted);
  }

  .password-input {
    width: 100%;
    padding: 8px 10px;
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 14px;
    color: var(--text-primary);
    font-family: var(--font-mono);
  }

  .password-input:focus {
    border-color: var(--accent);
    outline: none;
  }

  .password-input::placeholder {
    color: var(--text-muted);
  }

  .input-error {
    font-size: 12px;
    color: var(--danger);
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px;
    border-top: 0.5px solid var(--border);
  }

  .btn {
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    transition: background 0.1s;
  }

  .btn-cancel {
    color: var(--text-secondary);
    background: transparent;
  }

  .btn-cancel:hover {
    background: var(--border);
  }

  .btn-confirm {
    color: #fff;
    background: var(--accent);
  }

  .btn-confirm:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-confirm:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
