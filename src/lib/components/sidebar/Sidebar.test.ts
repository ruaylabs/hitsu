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
  localStorage.clear();
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

  it("counts trashed entries separately and opens the recycle bin", async () => {
    vault.setEntries([...entries, { ...entries[0], id: "deleted", trashed: true }]);
    const recycleBin = render(Sidebar).getByRole("tab", { name: "Recycle Bin 1" });

    expect(screen.getByRole("tab", { name: "All items 3" })).toBeInTheDocument();
    await fireEvent.click(recycleBin);

    expect(selection.filter).toEqual({ kind: "trash" });
  });

  it("persists the collapsed tags state", async () => {
    vault.setEntries([{ ...entries[0], tags: ["work"] }]);
    const sidebar = render(Sidebar);

    const collapseButton = screen.getByRole("button", { name: "Collapse Tags" });
    expect(screen.getByRole("tab", { name: "work" })).toBeInTheDocument();
    expect(collapseButton).toHaveAttribute("aria-expanded", "true");

    await fireEvent.click(collapseButton);

    expect(screen.queryByRole("tab", { name: "work" })).not.toBeInTheDocument();
    expect(localStorage.getItem("kagi:sidebar-tags-collapsed")).toBe("true");
    sidebar.unmount();

    render(Sidebar);

    const expandButton = await screen.findByRole("button", { name: "Expand Tags" });
    expect(expandButton).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByRole("tab", { name: "work" })).not.toBeInTheDocument();

    await fireEvent.click(expandButton);

    expect(screen.getByRole("tab", { name: "work" })).toBeInTheDocument();
    expect(localStorage.getItem("kagi:sidebar-tags-collapsed")).toBe("false");
  });
});
