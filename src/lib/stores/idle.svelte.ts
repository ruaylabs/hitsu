import { vault } from "$lib/stores/vault.svelte";

const IDLE_TIMEOUT_MS = 5 * 60 * 1000; // 5 minutes
const SLEEP_GAP_MS = 30 * 1000; // lock if page was hidden longer than this

let lastActivity = Date.now();
let timer: ReturnType<typeof setInterval> | null = null;
let controller: AbortController | null = null;

function resetActivity() {
  lastActivity = Date.now();
}

/** Start the idle + sleep lock monitors. Stops any previous run first. */
export function startIdleTimer() {
  stopIdleTimer();

  lastActivity = Date.now();
  controller = new AbortController();
  const opts = { signal: controller.signal, passive: true } as AddEventListenerOptions;

  // Reset the idle clock on any user interaction
  document.addEventListener("mousemove", resetActivity, opts);
  document.addEventListener("keydown", resetActivity, opts);
  document.addEventListener("pointerdown", resetActivity, opts);
  document.addEventListener("wheel", resetActivity, opts);

  // When the tab becomes visible (wake from sleep, switch back to app, …),
  // lock if the user has been gone longer than SLEEP_GAP_MS.
  document.addEventListener(
    "visibilitychange",
    () => {
      if (document.hidden) return;
      if (Date.now() - lastActivity > SLEEP_GAP_MS) {
        vault.lock();
      }
    },
    opts,
  );

  // Poll every second for idle timeout
  timer = setInterval(() => {
    if (Date.now() - lastActivity >= IDLE_TIMEOUT_MS) {
      vault.lock();
    }
  }, 1000);
}

/** Stop the idle + sleep lock monitors. */
export function stopIdleTimer() {
  if (timer) {
    clearInterval(timer);
    timer = null;
  }
  if (controller) {
    controller.abort();
    controller = null;
  }
}
