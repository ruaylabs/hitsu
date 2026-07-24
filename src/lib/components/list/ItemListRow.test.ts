import { render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import type { EntrySummary } from "$lib/bridge/types";
import ItemListRow from "./ItemListRow.svelte";

const entry: EntrySummary = {
  id: "entry-1",
  type: "login",
  title: "Example",
  subtitle: "alice@example.com",
  tags: [],
  favorite: false,
};

describe("ItemListRow", () => {
  it("shows TOTP and attachment indicators", () => {
    render(ItemListRow, {
      entry: { ...entry, hasTotp: true, hasAttachments: true },
    });

    expect(screen.getByLabelText("Has TOTP")).toBeInTheDocument();
    expect(screen.getByLabelText("Has attachments")).toBeInTheDocument();
  });
});
