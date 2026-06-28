<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    title = "Enter master password",
    confirmLabel = "Unlock",
    showConfirm = true,
    onconfirm,
    oncancel,
  }: {
    title?: string;
    confirmLabel?: string;
    showConfirm?: boolean;
    onconfirm?: (password: string) => void;
    oncancel?: () => void;
  } = $props();

  let password = $state("");
  let error = $state("");

  function submit() {
    if (!password) {
      error = "Password is required";
      return;
    }
    error = "";
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
      <button class="dialog-close" onclick={oncancel} aria-label="Cancel">
        <Icon name="x" size={16} />
      </button>
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
          bind:value={password}
          oninput={() => (error = "")}
        />
        {#if error}
          <span class="input-error">{error}</span>
        {/if}
      </div>
    </div>

    {#if showConfirm}
      <footer class="dialog-footer">
        <button class="btn btn-cancel" onclick={oncancel}>Cancel</button>
        <button class="btn btn-confirm" onclick={submit} disabled={!password}>
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
