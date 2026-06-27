<script lang="ts">
  let { totpUri }: { totpUri: string } = $props();

  // Static dummy code for M1 mock — live TOTP in M4
  let code = $state("482591");
  let formattedCode = $derived(`${code.slice(0, 3)} ${code.slice(3)}`);

  // Static: ring at ~75% progress
  const circumference = 2 * Math.PI * 8; // ~50.27
  let dashoffset = $state(12.57); // 75% remaining
  let secondsRemaining = $state(22);
</script>

<div class="totp-field">
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
    <span class="totp-seconds">{secondsRemaining}</span>
  </div>
  <button class="totp-copy" aria-label="Copy TOTP code">
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
