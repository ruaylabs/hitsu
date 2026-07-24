<script lang="ts">
  import { security } from "$lib/stores/security.svelte";
  import IconButton from "../ui/IconButton.svelte";
  import PasswordStrengthMeter from "../ui/PasswordStrengthMeter.svelte";
  import DetailFieldRow from "./DetailFieldRow.svelte";

  let {
    label,
    reveal,
    copy: copyFn,
    showStrength = false,
    masked = "•".repeat(14),
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
    /** Placeholder shown while hidden. Pass a backend-masked value (e.g. a
     *  card number's "•••• 1234") to keep a non-secret hint visible. */
    masked?: string;
  } = $props();

  let revealed = $state(false);
  let plaintext = $state("");
  let hideRemaining = $state(0);
  let revealTimer: ReturnType<typeof setTimeout> | null = null;

  let copied = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | undefined;

  function hide() {
    revealed = false;
    plaintext = "";
    hideRemaining = 0;
    if (revealTimer) clearTimeout(revealTimer);
    revealTimer = null;
  }

  // Count down to the deadline so the auto-hide is visible instead of looking
  // like a glitch. Wakes just after each wall-clock second (same pattern as
  // TOTPField) — the display has one-second precision anyway.
  function tick(deadline: number) {
    const now = Date.now();
    if (now >= deadline) {
      hide();
      return;
    }
    hideRemaining = Math.ceil((deadline - now) / 1000);
    revealTimer = setTimeout(() => tick(deadline), 1000 - (now % 1000) + 5);
  }

  async function toggleReveal() {
    if (revealed) {
      hide();
      return;
    }
    try {
      plaintext = await reveal();
      revealed = true;
      if (security.clipboardClearSeconds > 0) {
        tick(Date.now() + security.clipboardClearSeconds * 1000);
      }
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

<DetailFieldRow {label}>
  <div class="field-main">
    <span class="field-value mono">{revealed ? plaintext : masked}</span>
    {#if revealed && security.clipboardClearSeconds > 0}
      <span class="hide-countdown">Hides in {hideRemaining}s</span>
    {/if}
    <div class="field-actions">
      <IconButton
        icon={copied ? "check" : "copy"}
        onclick={copy}
        aria-label={`Copy ${label.toLowerCase()}`}
        title={`Copy ${label.toLowerCase()}`}
      />
      <IconButton
        icon={revealed ? "eye-off" : "eye"}
        onclick={toggleReveal}
        aria-label={revealed ? `Hide ${label.toLowerCase()}` : `Reveal ${label.toLowerCase()}`}
        title={revealed
          ? `Hide ${label.toLowerCase()}`
          : security.clipboardClearSeconds > 0
            ? `Reveal ${label.toLowerCase()} (hides after ${security.clipboardClearSeconds}s)`
            : `Reveal ${label.toLowerCase()}`}
      />
    </div>
    {#if showStrength && revealed}
      <PasswordStrengthMeter password={plaintext} />
    {/if}
  </div>
</DetailFieldRow>

<style>
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

  .hide-countdown {
    font-size: 11px;
    color: var(--text-muted);
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }
</style>
