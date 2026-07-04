<script lang="ts">
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import PasswordStrengthMeter from "../ui/PasswordStrengthMeter.svelte";

  let {
    label,
    password,
    showStrength = false,
  }: {
    label: string;
    password: string;
    /** Show a strength meter under the value. Only meaningful for actual
     *  passwords — CVV/pin are always short and would always read "weak". */
    showStrength?: boolean;
  } = $props();

  let revealed = $state(false);
  let revealTimer: ReturnType<typeof setTimeout> | null = null;

  let copied = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | undefined;

  function toggleReveal() {
    if (revealed) {
      revealed = false;
      if (revealTimer) clearTimeout(revealTimer);
    } else {
      revealed = true;
      revealTimer = setTimeout(() => {
        revealed = false;
      }, 30000);
    }
  }

  function copy() {
    clipboard.copy(password);
    if (copyTimer) clearTimeout(copyTimer);
    copied = true;
    copyTimer = setTimeout(() => (copied = false), 1000);
  }
</script>

<div class="field-row">
  <span class="field-label">{label}</span>
  <div class="field-main">
    <span class="field-value mono">{revealed ? password : "•".repeat(14)}</span>
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
    {#if showStrength}
      <PasswordStrengthMeter {password} />
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
