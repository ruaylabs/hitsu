import * as clipboardBridge from "$lib/bridge/clipboard";
import * as entriesBridge from "$lib/bridge/entries";
import type { SecretField } from "$lib/bridge/types";

let remainingMs = $state(0);
let defaultTimeoutSecs = $state(15);
let timer: ReturnType<typeof setInterval> | null = null;

function stop() {
  if (timer !== null) clearInterval(timer);
  timer = null;
  remainingMs = 0;
}

function startCountdown(secs: number) {
  if (secs <= 0) return;
  remainingMs = secs * 1000;
  timer = setInterval(() => {
    remainingMs = Math.max(0, remainingMs - 1000);
    if (remainingMs <= 0) stop();
  }, 1000);
}

async function copyAndTrack(copy: () => Promise<void>, timeoutSecs = 0) {
  stop();
  await copy();
  startCountdown(timeoutSecs);
}

export const clipboard = {
  get remainingMs() {
    return remainingMs;
  },
  get active() {
    return remainingMs > 0;
  },
  get defaultTimeoutSecs() {
    return defaultTimeoutSecs;
  },
  set defaultTimeoutSecs(v: number) {
    defaultTimeoutSecs = v;
  },
  /** Copy a protected value (password, CVV) with auto-clear countdown.
   *  Pass timeoutSecs=0 or "Never" to skip auto-clear entirely.
   *  Does NOT fall back to the WebView clipboard API — if the Rust command
   *  fails the error is surfaced and the copy does not proceed silently. */
  async copy(value: string, timeoutSecs?: number) {
    const secs = timeoutSecs ?? defaultTimeoutSecs;
    await copyAndTrack(
      () =>
        secs > 0
          ? clipboardBridge.clipboardCopyWithTimeout(value, secs)
          : clipboardBridge.clipboardCopy(value),
      secs,
    );
  },
  /** Copy an entry's secret field (password, CVV, card number, …) with the
   *  auto-clear countdown. The plaintext is read and copied inside the Rust
   *  backend — it never crosses IPC to the webview. Pass `version` to copy
   *  from a history revision. */
  async copySecretField(id: string, field: SecretField, version?: number) {
    const secs = defaultTimeoutSecs;
    await copyAndTrack(() => entriesBridge.entryCopyField(id, field, secs, version), secs);
  },
  /** Copy a protected custom field without exposing it to the webview. */
  async copyCustomField(id: string, name: string) {
    const secs = defaultTimeoutSecs;
    await copyAndTrack(() => entriesBridge.entryCopyCustomField(id, name, secs), secs);
  },
  /** Copy a plain value (username, URL) without auto-clear.
   *  Does NOT fall back to the WebView clipboard API — if the Rust command
   *  fails the error is surfaced and the copy does not proceed silently. */
  async copyPlain(value: string) {
    await copyAndTrack(() => clipboardBridge.clipboardCopy(value));
  },
  cancel() {
    stop();
    clipboardBridge.clipboardClear().catch(() => {});
  },
};
