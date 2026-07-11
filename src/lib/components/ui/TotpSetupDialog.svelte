<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    oncancel,
    onconfirm,
  }: {
    oncancel: () => void;
    onconfirm: (otpauthUri: string) => void;
  } = $props();

  let seed = $state("");
  let error = $state("");

  const BASE32_RE = /^[A-Z2-7]+=*$/i;

  function normalizeSeed(raw: string): string {
    // Remove whitespace and dashes, uppercase
    return raw.replace(/[\s-]/g, "").toUpperCase();
  }

  function isValidBase32(s: string): boolean {
    return s.length > 0 && BASE32_RE.test(s);
  }

  function submit() {
    const normalized = normalizeSeed(seed);
    if (!normalized) {
      error = "Seed code is required";
      return;
    }
    if (!isValidBase32(normalized)) {
      error = "Invalid seed code (expected base32 characters: A-Z, 2-7)";
      return;
    }
    error = "";
    const uri = `otpauth://totp/entry?secret=${encodeURIComponent(normalized)}&period=30&digits=6`;
    onconfirm(uri);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      submit();
    } else if (e.key === "Escape") {
      oncancel();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
  class="dialog-overlay"
  onclick={oncancel}
  role="dialog"
  aria-label="Setup TOTP from seed"
  tabindex="-1"
>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="dialog-pane" onclick={(e) => e.stopPropagation()}>
    <header class="dialog-header">
      <h2 class="dialog-title">Setup TOTP from seed</h2>
      <button class="dialog-close" onclick={oncancel} aria-label="Cancel" title="Close">
        <Icon name="x" size={16} />
      </button>
    </header>

    <div class="dialog-body">
      <p class="dialog-message">
        Enter the TOTP secret seed code from the website. It will be converted to the standard
        otpauth:// format and saved to this entry.
      </p>
      <label class="input-label" for="totp-seed">Seed code</label>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        id="totp-seed"
        type="text"
        class="seed-input"
        placeholder="e.g. JBSWY3DPEHPK3PXP"
        autofocus
        autocomplete="off"
        autocorrect="off"
        autocapitalize="characters"
        spellcheck="false"
        bind:value={seed}
        oninput={() => { error = ""; }}
      />
      {#if error}
        <span class="input-error">{error}</span>
      {/if}
    </div>

    <footer class="dialog-footer">
      <button class="btn btn-cancel" onclick={oncancel}>Cancel</button>
      <button class="btn btn-confirm" onclick={submit} disabled={!seed.trim()}>Save</button>
    </footer>
  </div>
</div>

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: var(--backdrop);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: var(--z-dialog);
  }

  .dialog-pane {
    width: 380px;
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-card);
    overflow: hidden;
    box-shadow: var(--shadow-dialog);
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
    width: var(--icon-button-size);
    height: var(--icon-button-size);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
  }

  .dialog-close:hover {
    background: var(--border);
  }

  .dialog-body {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .dialog-message {
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.5;
    margin: 0;
  }

  .input-label {
    font-size: 11px;
    color: var(--text-muted);
  }

  .seed-input {
    width: 100%;
    padding: 8px 10px;
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 14px;
    color: var(--text-primary);
    font-family: var(--font-mono);
  }

  .seed-input:focus {
    border-color: var(--accent);
    outline: none;
  }

  .seed-input::placeholder {
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
    transition: background var(--transition-fast);
    cursor: pointer;
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
