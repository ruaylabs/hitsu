<script lang="ts">
  import * as totpBridge from "$lib/bridge/totp";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import IconButton from "../ui/IconButton.svelte";
  import DetailFieldRow from "./DetailFieldRow.svelte";

  // Only the entry id: the backend reads the TOTP seed itself and returns
  // just the ephemeral code, so the otpauth:// URI never enters the webview.
  let { entryId }: { entryId: string } = $props();

  let code = $state("------");
  let period = $state(30);
  let remaining = $state(30);
  let flash = $state(false);
  let copied = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | undefined;

  let circumference = $derived(2 * Math.PI * 8); // r=8 → ~50.27
  let dashoffset = $derived(circumference - (remaining / period) * circumference);

  $effect(() => {
    const id = entryId;
    let active = true;
    let prevCounter = -1;
    let tickTimer: ReturnType<typeof setTimeout> | undefined;
    let flashTimer: ReturnType<typeof setTimeout> | undefined;

    async function computeCode() {
      try {
        const result = await totpBridge.totpCompute(id);
        if (!active) return;
        code = result.code;
        remaining = result.remaining;
        period = result.period;
      } catch {
        if (active) code = "------";
      }
    }

    function updateClock() {
      const now = Date.now();
      remaining = period - (Math.floor(now / 1000) % period);

      const counter = Math.floor(now / 1000 / period);
      if (counter !== prevCounter) {
        prevCounter = counter;
        flash = true;
        if (flashTimer) clearTimeout(flashTimer);
        flashTimer = setTimeout(() => (flash = false), 200);
        void computeCode();
      }

      // Wake just after the next wall-clock second instead of polling on
      // every animation frame. The displayed countdown has one-second
      // precision, so frame-rate updates only waste CPU.
      const delay = 1000 - (now % 1000) + 5;
      tickTimer = setTimeout(updateClock, delay);
    }

    async function start() {
      await computeCode();
      if (!active) return;
      const now = Date.now();
      remaining = period - (Math.floor(now / 1000) % period);
      prevCounter = Math.floor(now / 1000 / period);
      tickTimer = setTimeout(updateClock, 1000 - (now % 1000) + 5);
    }

    void start();
    return () => {
      active = false;
      if (tickTimer) clearTimeout(tickTimer);
      if (flashTimer) clearTimeout(flashTimer);
    };
  });

  let formattedCode = $derived(code.length >= 3 ? `${code.slice(0, 3)} ${code.slice(3)}` : code);
  let expiring = $derived(remaining <= 10);
  let fillColor = $derived(expiring ? "var(--danger)" : "var(--success)");

  function copyCode() {
    clipboard.copy(code);
    if (copyTimer) clearTimeout(copyTimer);
    copied = true;
    copyTimer = setTimeout(() => (copied = false), 1000);
  }
</script>

<DetailFieldRow
  label="TOTP"
  standalone
  status={expiring ? "danger" : flash ? "success" : "default"}
>
  <span class="totp-code" class:expiring>{formattedCode}</span>
  <div class="totp-ring-container">
    <svg width="20" height="20" viewBox="0 0 20 20">
      <circle cx="10" cy="10" r="8" fill="none" stroke="var(--border-strong)" stroke-width="1.5" />
      <circle
        cx="10"
        cy="10"
        r="8"
        fill="none"
        stroke={fillColor}
        stroke-width="1.5"
        stroke-dasharray={circumference}
        stroke-dashoffset={dashoffset}
        transform="rotate(-90, 10, 10)"
        stroke-linecap="round"
      />
    </svg>
    <span class="totp-seconds">{remaining}</span>
  </div>
  <IconButton
    icon={copied ? "check" : "copy"}
    onclick={copyCode}
    aria-label="Copy TOTP code"
    title="Copy TOTP code"
  />
</DetailFieldRow>

<style>
  .totp-code {
    font-size: 16px;
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
    letter-spacing: 2px;
    font-weight: 500;
    color: var(--text-primary);
    transition: color 0.3s;
  }

  .totp-code.expiring {
    color: var(--danger);
  }

  .totp-ring-container {
    position: relative;
    margin-left: auto;
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
</style>
