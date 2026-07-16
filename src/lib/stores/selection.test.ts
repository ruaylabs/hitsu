import { describe, expect, it, vi } from "vitest";
import { selection } from "./selection.svelte";

describe("selection detail fetch mode", () => {
  it("marks keyboard selections and resets for direct assignments", () => {
    selection.select("entry-1", "keyboard");
    expect(selection.selectedId).toBe("entry-1");
    expect(selection.detailFetchMode).toBe("keyboard");

    selection.selectedId = "entry-2";
    expect(selection.detailFetchMode).toBe("immediate");
  });
});

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
