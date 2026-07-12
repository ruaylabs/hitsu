import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it } from "vitest";
import type { EntrySummary } from "$lib/bridge/types";
import { selection } from "$lib/stores/selection.svelte";
import { vault } from "$lib/stores/vault.svelte";
import Sidebar from "./Sidebar.svelte";

const entries: EntrySummary[] = [
  { id: "1", type: "login", title: "Login", subtitle: "", tags: [], favorite: false },
  { id: "2", type: "password", title: "Password", subtitle: "", tags: [], favorite: false },
  { id: "3", type: "password", title: "Password 2", subtitle: "", tags: [], favorite: false },
];

beforeEach(() => {
  selection.selectedId = null;
  selection.search = "";
  selection.filter = { kind: "all" };
  vault.setEntries(entries);
});

describe("Sidebar", () => {
  it("shows type counts and selects the password filter", async () => {
    const passwords = render(Sidebar).getByRole("tab", { name: "Passwords 2" });

    expect(passwords).toHaveAttribute("aria-selected", "false");
    await fireEvent.click(passwords);

    expect(selection.filter).toEqual({ kind: "type", type: "password" });
    expect(screen.getByRole("tab", { name: "Passwords 2" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });
});
