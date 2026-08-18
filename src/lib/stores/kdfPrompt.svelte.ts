/** Visibility state for the per-vault KDF-upgrade prompt.
 *
 * `update` recomputes visibility from vault state (call it from an effect
 * watching meta, lock state, and the dismissed-paths list) so switching to
 * another vault — healthy or not — always lands on the right answer.
 * `dismiss` hides the prompt until the next recompute, e.g. Esc-close stays
 * hidden until a lock/unlock cycle re-runs the effect; only a persisted
 * dismissal (fed back via `dismissedVaults`) keeps it gone for good. */
export function createKdfPrompt() {
  let visible = $state(false);

  return {
    get visible() {
      return visible;
    },
    update(
      meta: { path?: string; kdfNeedsUpgrade?: boolean } | null,
      locked: boolean,
      dismissedVaults: string[],
    ) {
      const path = meta?.path ?? null;
      const dismissed = path ? dismissedVaults.includes(path) : false;
      visible = Boolean(meta?.kdfNeedsUpgrade) && !locked && !dismissed;
    },
    dismiss() {
      visible = false;
    },
  };
}
