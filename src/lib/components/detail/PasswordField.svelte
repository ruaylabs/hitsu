<script lang="ts">
  import PasswordStrengthMeter from "../ui/PasswordStrengthMeter.svelte";

  let {
    label,
    reveal,
    copy: copyFn,
    showStrength = false,
  }: {
    label: string;
    /** Fetch the plaintext on demand (explicit reveal only — the secret is
     *  not in the webview until the user asks for it). */
    reveal: () => Promise<string>;
    /** Copy backend-side; the secret never crosses IPC for this path. */
    copy: () => Promise<void>;
    /** Show a strength meter under the value. Only meaningful for actual
     *  passwords — CVV/pin are always short and would always read "weak".
     *  Shown only while revealed, since the plaintext isn't held otherwise. */
    showStrength?: boolean;
  } = $props();

  let revealed = $state(false);
  let plaintext = $state("");
  let revealTimer: ReturnType<typeof setTimeout> | null = null;

  let copied = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | undefined;

  function hide() {
    revealed = false;
    plaintext = "";
    if (revealTimer) clearTimeout(revealTimer);
    revealTimer = null;
  }

  async function toggleReveal() {
    if (revealed) {
      hide();
      return;
    }
    try {
      plaintext = await reveal();
      revealed = true;
      revealTimer = setTimeout(hide, 30000);
    } catch (e) {
      console.error("Failed to reveal secret", e);
    }
  }

  // Drop the plaintext when the component unmounts or switches entries.
  $effect(() => () => hide());

  async function copy() {
    try {
      await copyFn();
    } catch (e) {
      console.error("Failed to copy secret", e);
      return;
    }
    if (copyTimer) clearTimeout(copyTimer);
    copied = true;
    copyTimer = setTimeout(() => (copied = false), 1000);
  }
</script>

<div class="field-row">
  <span class="field-label">{label}</span>
  <div class="field-main">
    <span class="field-value mono">{revealed ? plaintext : "•".repeat(14)}</span>
    <div class="field-actions">
      <button class="field-action" onclick={copy} aria-label="Copy password" title="Copy password">
        <i class="ti ti-{copied ? 'check' : 'copy'}" style="font-size: 15px"></i>
      </button>
      <button
        class="field-action"
        onclick={toggleReveal}
        aria-label={revealed ? "Hide password" : "Reveal password"}
        title={revealed ? "Hide password" : "Reveal password"}
      >
        <i class="ti ti-{revealed ? 'eye-off' : 'eye'}" style="font-size: 15px"></i>
      </button>
    </div>
    {#if showStrength && revealed}
      <PasswordStrengthMeter password={plaintext} />
    {/if}
  </div>
</div>

<style>
  .field-row {
    background: var(--surface-2);
    padding: 10px 12px;
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 38px;
  }

  .field-label {
    font-size: 11px;
    color: var(--text-muted);
    width: 70px;
    flex-shrink: 0;
  }

  .field-value {
    font-size: 13.5px;
    color: var(--text-primary);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .field-value.mono {
    font-family: var(--font-mono);
  }

  .field-main {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .field-actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .field-action {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition: background 0.1s;
  }

  .field-action:hover {
    background: var(--border);
  }
</style>
