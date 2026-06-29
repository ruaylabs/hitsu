<script lang="ts">
  import { parseOtpauthUri, computeTotp, totpRemainingSeconds } from "$lib/utils/otp";
  import { clipboard } from "$lib/stores/clipboard.svelte";

  let { totpUri }: { totpUri: string } = $props();

  let params = $derived(parseOtpauthUri(totpUri));
  let code = $state("------");
  let remaining = $state(30);
  let flash = $state(false);

  let period = $derived(params?.period ?? 30);
  let circumference = $derived(2 * Math.PI * 8); // r=8 → ~50.27
  let dashoffset = $derived(circumference - (remaining / period) * circumference);

  let prevCounter = $state(-1);

  function tick() {
    const nowCounter = Math.floor(Date.now() / 1000 / period);
    if (nowCounter !== prevCounter) {
      prevCounter = nowCounter;
      flash = true;
      setTimeout(() => (flash = false), 200);
      computeCode();
    }
    remaining = totpRemainingSeconds(period);
  }

  async function computeCode() {
    if (!params) return;
    try {
      code = await computeTotp(params);
    } catch {
      code = "------";
    }
  }

  // Recompute periodically
  let interval: ReturnType<typeof setInterval>;

  $effect(() => {
    if (!params) return;
    computeCode();
    remaining = totpRemainingSeconds(period);
    prevCounter = Math.floor(Date.now() / 1000 / period);
    interval = setInterval(tick, 250);
    return () => clearInterval(interval);
  });

  let formattedCode = $derived(code.length >= 3 ? `${code.slice(0, 3)} ${code.slice(3)}` : code);

  function copyCode() {
    clipboard.copy(code);
  }
</script>

<div class="totp-field" class:flash>
  <span class="totp-label">TOTP</span>
  <span class="totp-code">{formattedCode}</span>
  <div class="totp-ring-container">
    <svg width="20" height="20" viewBox="0 0 20 20">
      <circle cx="10" cy="10" r="8" fill="none" stroke="var(--border-strong)" stroke-width="1.5" />
      <circle
        cx="10"
        cy="10"
        r="8"
        fill="none"
        stroke="var(--success)"
        stroke-width="1.5"
        stroke-dasharray={circumference}
        stroke-dashoffset={dashoffset}
        transform="rotate(-90, 10, 10)"
        stroke-linecap="round"
      />
    </svg>
    <span class="totp-seconds">{remaining}</span>
  </div>
  <button class="totp-copy" onclick={copyCode} aria-label="Copy TOTP code">
    <i class="ti ti-copy" style="font-size: 15px"></i>
  </button>
</div>

<style>
  .totp-field {
    background: var(--surface-2);
    padding: 10px 12px;
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
    transition: border-color 0.15s;
  }

  .totp-field.flash {
    border-color: var(--success);
  }

  .totp-label {
    font-size: 11px;
    color: var(--text-muted);
    width: 70px;
    flex-shrink: 0;
  }

  .totp-code {
    font-size: 16px;
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    letter-spacing: 2px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .totp-ring-container {
    position: relative;
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .totp-seconds {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 7px;
    font-weight: 500;
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }

  .totp-copy {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    margin-left: auto;
  }

  .totp-copy:hover {
    background: var(--border);
  }
</style>
