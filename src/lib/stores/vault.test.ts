import { beforeEach, describe, expect, it } from "vitest";
import type { EntrySummary, VaultMeta } from "$lib/bridge/types";
import { selection } from "./selection.svelte";
import { vault } from "./vault.svelte";

const firstEntry: EntrySummary = {
  id: "first",
  type: "password",
  title: "First",
  subtitle: "",
  tags: [],
  favorite: false,
};

const secondEntry: EntrySummary = {
  id: "second",
  type: "login",
  title: "Second",
  subtitle: "alice",
  tags: [],
  favorite: false,
};

beforeEach(() => {
  selection.selectedId = null;
  selection.search = "";
  selection.filter = { kind: "all" };
  vault.setEntries([]);
  vault.setCreatingId(null);
  vault.setEditingId(null);
});

describe("vault store", () => {
  it("resets selection when opening a vault and replaces its entries", () => {
    selection.selectedId = "stale-entry";
    selection.search = "stale search";
    selection.filter = { kind: "favorites" };
    const meta: VaultMeta = {
      path: "/tmp/test.kdbx",
      name: "Test vault",
      itemCount: 1,
      syncProvider: "local",
      entries: [firstEntry],
    };

    vault.openVault(meta);

    expect(vault.meta).toEqual(meta);
    expect(vault.entries).toEqual([firstEntry]);
    expect(selection.selectedId).toBeNull();
    expect(selection.search).toBe("");
    expect(selection.filter).toEqual({ kind: "all" });

    vault.setEntries([secondEntry]);
    expect(vault.entries).toEqual([secondEntry]);
  });
});
