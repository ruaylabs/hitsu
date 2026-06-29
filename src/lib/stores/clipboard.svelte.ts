import * as clipboardBridge from "$lib/bridge/clipboard";

let remainingMs = $state(0);
let defaultTimeoutSecs = $state(15);
let timer: ReturnType<typeof setInterval> | null = null;

function stop() {
  if (timer !== null) clearInterval(timer);
  timer = null;
  remainingMs = 0;
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
   *  Pass timeoutSecs=0 or "Never" to skip auto-clear entirely. */
  async copy(value: string, timeoutSecs?: number) {
    const secs = timeoutSecs ?? defaultTimeoutSecs;
    stop();
    if (secs > 0) {
      try {
        await clipboardBridge.clipboardCopyWithTimeout(value, secs);
      } catch {
        await navigator.clipboard.writeText(value);
      }
      remainingMs = secs * 1000;
      timer = setInterval(() => {
        remainingMs = Math.max(0, remainingMs - 1000);
        if (remainingMs <= 0) {
          stop();
        }
      }, 1000);
    } else {
      // No auto-clear — just copy directly
      try {
        await clipboardBridge.clipboardCopy(value);
      } catch {
        await navigator.clipboard.writeText(value);
      }
    }
  },
  /** Copy a plain value (username, URL) without auto-clear */
  async copyPlain(value: string) {
    stop();
    try {
      await clipboardBridge.clipboardCopy(value);
    } catch {
      await navigator.clipboard.writeText(value);
    }
  },
  cancel() {
    stop();
    clipboardBridge.clipboardClear().catch(() => {});
  },
};
