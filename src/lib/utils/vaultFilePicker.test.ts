import { beforeEach, describe, expect, it, vi } from "vitest";
import { nativeDialog } from "$lib/stores/nativeDialog.svelte";
import { pickVaultToCreate, pickVaultToOpen } from "./vaultFilePicker";

const mocks = vi.hoisted(() => ({ open: vi.fn(), save: vi.fn() }));

vi.mock("@tauri-apps/plugin-dialog", () => mocks);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("vault file picker", () => {
  it("chooses an existing KDBX file through the native-dialog guard", async () => {
    mocks.open.mockResolvedValue("/vaults/main.kdbx");

    const result = pickVaultToOpen();

    expect(nativeDialog.open).toBe(true);
    await expect(result).resolves.toBe("/vaults/main.kdbx");
    expect(nativeDialog.open).toBe(false);
    expect(mocks.open).toHaveBeenCalledWith({
      multiple: false,
      filters: [{ name: "KeePass Database", extensions: ["kdbx"] }],
    });
  });

  it("chooses a new KDBX destination through the native-dialog guard", async () => {
    mocks.save.mockResolvedValue("/vaults/new.kdbx");

    const result = pickVaultToCreate();

    expect(nativeDialog.open).toBe(true);
    await expect(result).resolves.toBe("/vaults/new.kdbx");
    expect(nativeDialog.open).toBe(false);
    expect(mocks.save).toHaveBeenCalledWith({
      filters: [{ name: "KeePass Database", extensions: ["kdbx"] }],
      defaultPath: "vault.kdbx",
    });
  });
});
