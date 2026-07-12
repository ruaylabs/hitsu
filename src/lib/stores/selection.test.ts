import { describe, expect, it, vi } from "vitest";
import { selection } from "./selection.svelte";

describe("selection navigation guards", () => {
  it("defers navigation until unsaved changes are resolved", () => {
    const navigate = vi.fn();
    let resume: (() => void) | undefined;
    const removeGuard = selection.setNavigationGuard((pendingNavigation) => {
      resume = pendingNavigation;
      return false;
    });

    expect(selection.requestNavigation(navigate)).toBe(false);
    expect(navigate).not.toHaveBeenCalled();

    resume?.();
    expect(navigate).toHaveBeenCalledOnce();
    removeGuard();
  });
});
