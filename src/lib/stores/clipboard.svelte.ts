import * as clipboardBridge from "$lib/bridge/clipboard";

let remainingMs = $state(0);
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
  /** Copy a protected value (password, CVV) with auto-clear countdown */
  async copy(value: string, timeoutSecs = 15) {
    stop();
    try {
      await clipboardBridge.clipboardCopyWithTimeout(value, timeoutSecs);
    } catch {
      await navigator.clipboard.writeText(value);
    }
    remainingMs = timeoutSecs * 1000;
    timer = setInterval(() => {
      remainingMs = Math.max(0, remainingMs - 1000);
      if (remainingMs <= 0) {
        stop();
      }
    }, 1000);
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
