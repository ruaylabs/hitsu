import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { clipboard } from "./clipboard.svelte";

const mocks = vi.hoisted(() => ({
  copy: vi.fn(),
  copyWithTimeout: vi.fn(),
  clear: vi.fn(),
  copyField: vi.fn(),
}));

vi.mock("$lib/bridge/clipboard", () => ({
  clipboardCopy: mocks.copy,
  clipboardCopyWithTimeout: mocks.copyWithTimeout,
  clipboardClear: mocks.clear,
}));

vi.mock("$lib/bridge/entries", () => ({
  entryCopyField: mocks.copyField,
}));

beforeEach(() => {
  vi.useFakeTimers();
  vi.clearAllMocks();
  clipboard.defaultTimeoutSecs = 2;
  mocks.copyField.mockResolvedValue(undefined);
  mocks.clear.mockResolvedValue(undefined);
});

afterEach(() => {
  clipboard.cancel();
  vi.useRealTimers();
});

describe("clipboard store", () => {
  it("tracks the backend secret-clear countdown", async () => {
    await clipboard.copySecretField("entry-1", "password");

    expect(mocks.copyField).toHaveBeenCalledWith("entry-1", "password", 2, undefined);
    expect(clipboard.remainingMs).toBe(2000);

    vi.advanceTimersByTime(1000);
    expect(clipboard.remainingMs).toBe(1000);

    vi.advanceTimersByTime(1000);
    expect(clipboard.active).toBe(false);
  });

  it("cancels the countdown and clears the clipboard", async () => {
    await clipboard.copySecretField("entry-1", "password");

    clipboard.cancel();

    expect(clipboard.active).toBe(false);
    expect(mocks.clear).toHaveBeenCalledOnce();
  });
});
