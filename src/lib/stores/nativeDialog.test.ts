import { describe, expect, it } from "vitest";
import { nativeDialog } from "./nativeDialog.svelte";

describe("nativeDialog store", () => {
  it("is open only while a wrapped task runs", async () => {
    expect(nativeDialog.open).toBe(false);
    let resolve!: (value: string) => void;
    const pending = nativeDialog.during(() => new Promise<string>((r) => (resolve = r)));
    expect(nativeDialog.open).toBe(true);
    resolve("picked");
    await expect(pending).resolves.toBe("picked");
    expect(nativeDialog.open).toBe(false);
  });

  it("closes even when the task rejects", async () => {
    await expect(nativeDialog.during(() => Promise.reject(new Error("cancelled")))).rejects.toThrow(
      "cancelled",
    );
    expect(nativeDialog.open).toBe(false);
  });

  it("counts overlapping tasks", async () => {
    let resolveFirst!: () => void;
    let resolveSecond!: () => void;
    const first = nativeDialog.during(() => new Promise<void>((r) => (resolveFirst = r)));
    const second = nativeDialog.during(() => new Promise<void>((r) => (resolveSecond = r)));
    resolveFirst();
    await first;
    expect(nativeDialog.open).toBe(true);
    resolveSecond();
    await second;
    expect(nativeDialog.open).toBe(false);
  });
});
