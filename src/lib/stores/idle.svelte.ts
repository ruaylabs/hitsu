import { invoke } from "@tauri-apps/api/core";
import { vault } from "$lib/stores/vault.svelte";

const DEFAULT_IDLE_MS = 5 * 60 * 1000; // 5 minutes
const SLEEP_GAP_MS = 30 * 1000; // max gap for sleep-detection lock

let lastActivity = Date.now();
let activityDirty = false;
let timer: ReturnType<typeof setInterval> | null = null;
let controller: AbortController | null = null;

/** Mark activity as seen — cheap flag toggled on every input event.
 *  The polling loop below updates `lastActivity` once per second
 *  if activity occurred, avoiding `Date.now()` in the hot path. */
function markActivity() {
  activityDirty = true;
}

/** Start the idle + sleep lock monitors. Stops any previous run first.
 *  @param idleTimeoutMs — idle timeout in ms. Pass 0 to never idle-lock. */
export function startIdleTimer(idleTimeoutMs = DEFAULT_IDLE_MS) {
  stopIdleTimer();

  lastActivity = Date.now();
  activityDirty = false;
  controller = new AbortController();
  const opts = { signal: controller.signal, passive: true } as AddEventListenerOptions;

  // Mark activity on any user interaction (throttled to polling interval)
  document.addEventListener("mousemove", markActivity, opts);
  document.addEventListener("keydown", markActivity, opts);
  document.addEventListener("pointerdown", markActivity, opts);
  document.addEventListener("wheel", markActivity, opts);

  // Effective sleep gap: never lock if idle-lock is disabled (0);
  // otherwise cap at the idle timeout so short timeouts aren't
  // undermined by a 30 s sleep-detection gap.
  const sleepGap = idleTimeoutMs > 0 ? Math.min(SLEEP_GAP_MS, idleTimeoutMs) : Infinity;

  document.addEventListener(
    "visibilitychange",
    async () => {
      if (document.hidden) return;
      if (Date.now() - lastActivity > sleepGap) {
        await vault.lock();
      }
    },
    opts,
  );

  // Poll every second for idle timeout and flush activity
  timer = setInterval(async () => {
    const now = Date.now();
    if (activityDirty) {
      lastActivity = now;
      activityDirty = false;
      // Keep the independent backend watchdog aligned with real user input.
      // If the webview hangs, these heartbeats stop and the backend still locks.
      if (idleTimeoutMs > 0) void invoke("idle_activity").catch(() => {});
    }
    if (idleTimeoutMs > 0 && now - lastActivity >= idleTimeoutMs) {
      await vault.lock();
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
