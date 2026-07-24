import type { Toast } from "$lib/bridge/types";

let toasts = $state<Toast[]>([]);

function push(kind: Toast["kind"], message: string, durationMs: number, action?: Toast["action"]) {
  const id = crypto.randomUUID();
  toasts = [...toasts, { id, kind, message, durationMs, action }];
  if (durationMs > 0) {
    setTimeout(() => toast.dismiss(id), durationMs);
  }
}

/** App-wide notifications, rendered by ToastStack in the root layout.
 *  Use for failures the user would otherwise never see (background saves,
 *  deletes, clipboard) — errors with a natural inline home (a form) should
 *  stay inline. */
export const toast = {
  get all() {
    return toasts;
  },
  info(message: string, durationMs = 4000, action?: Toast["action"]) {
    push("info", message, durationMs, action);
  },
  success(message: string, durationMs = 3000) {
    push("success", message, durationMs);
  },
  /** Errors stay longer: the user may need to read and act on them. */
  error(message: string, durationMs = 8000) {
    push("danger", message, durationMs);
  },
  dismiss(id: string) {
    toasts = toasts.filter((t) => t.id !== id);
  },
};
