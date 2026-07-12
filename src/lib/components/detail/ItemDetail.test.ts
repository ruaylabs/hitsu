import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Entry, EntrySummary } from "$lib/bridge/types";
import { clipboard } from "$lib/stores/clipboard.svelte";
import { saveStatus } from "$lib/stores/saveStatus.svelte";
import { selection } from "$lib/stores/selection.svelte";
import { vault } from "$lib/stores/vault.svelte";
import ItemDetail from "./ItemDetail.svelte";

const mocks = vi.hoisted(() => ({
  entryGet: vi.fn(),
  entryRevealField: vi.fn(),
  entryUpdate: vi.fn(),
  entryDiscard: vi.fn(),
  entryDelete: vi.fn(),
  entryCopyField: vi.fn(),
  clipboardCopy: vi.fn(),
  clipboardClear: vi.fn(),
  openUrl: vi.fn(),
}));

vi.mock("$lib/bridge/entries", () => ({
  entryGet: mocks.entryGet,
  entryRevealField: mocks.entryRevealField,
  entryUpdate: mocks.entryUpdate,
  entryDiscard: mocks.entryDiscard,
  entryDelete: mocks.entryDelete,
  entryCopyField: mocks.entryCopyField,
  toSummary: (entry: Entry): EntrySummary => ({
    id: entry.id,
    type: entry.type,
    title: entry.title,
    subtitle: entry.subtitle,
    url: entry.url,
    username: entry.username,
    tags: entry.tags,
    favorite: entry.favorite,
    iconHint: entry.iconHint,
  }),
}));

vi.mock("$lib/bridge/clipboard", () => ({
  clipboardCopy: mocks.clipboardCopy,
  clipboardCopyWithTimeout: vi.fn(),
  clipboardClear: mocks.clipboardClear,
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: mocks.openUrl,
}));

function passwordEntry(overrides: Partial<Entry> = {}): Entry {
  return {
    id: "password-1",
    type: "password",
    title: "Recovery password",
    subtitle: "",
    url: "https://example.com",
    hasPassword: true,
    hasTotp: false,
    notes: "Original note",
    tags: ["work"],
    favorite: false,
    attachments: [],
    customFields: [],
    modifiedAt: "2026-07-11T00:00:00Z",
    createdAt: "2026-07-11T00:00:00Z",
    historyCount: 0,
    ...overrides,
  };
}

function summary(entry: Entry): EntrySummary {
  return {
    id: entry.id,
    type: entry.type,
    title: entry.title,
    subtitle: entry.subtitle,
    url: entry.url,
    username: entry.username,
    tags: entry.tags,
    favorite: entry.favorite,
    iconHint: entry.iconHint,
  };
}

function selectEntry(entry: Entry, creating = false) {
  mocks.entryGet.mockResolvedValue(entry);
  vault.setEntries([summary(entry)]);
  vault.setCreatingId(creating ? entry.id : null);
  vault.setEditingId(creating ? entry.id : null);
  selection.selectedId = entry.id;
}

beforeEach(() => {
  vi.clearAllMocks();
  selection.selectedId = null;
  selection.search = "";
  selection.filter = { kind: "all" };
  vault.setEntries([]);
  vault.setCreatingId(null);
  vault.setEditingId(null);
  clipboard.defaultTimeoutSecs = 0;
  saveStatus.markSaved();
  mocks.entryRevealField.mockResolvedValue("stored-password");
  mocks.entryDiscard.mockResolvedValue(undefined);
  mocks.entryCopyField.mockResolvedValue(undefined);
  mocks.clipboardCopy.mockResolvedValue(undefined);
  mocks.clipboardClear.mockResolvedValue(undefined);
});

afterEach(() => {
  selection.selectedId = null;
  vault.setCreatingId(null);
  vault.setEditingId(null);
  clipboard.cancel();
});

describe("ItemDetail errors", () => {
  it("shows entry loading failures", async () => {
    const entry = passwordEntry();
    vault.setEntries([summary(entry)]);
    mocks.entryGet.mockRejectedValue(new Error("Entry unavailable"));
    selection.selectedId = entry.id;

    render(ItemDetail);

    expect(await screen.findByText("Entry unavailable")).toBeInTheDocument();
  });

  it("keeps editing and shows save failures", async () => {
    selectEntry(passwordEntry());
    mocks.entryUpdate.mockRejectedValue(new Error("Vault changed on disk"));
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Edit entry" }));
    await waitFor(() => expect(screen.getByPlaceholderText("Password")).toBeInTheDocument());
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText("Vault changed on disk")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();
    expect(saveStatus.state).toBe("error");
    consoleError.mockRestore();
  });
});

describe("password entry workflow", () => {
  it("shows password and URL editors for a new password entry", async () => {
    selectEntry(passwordEntry({ hasPassword: false, url: undefined }), true);
    render(ItemDetail);

    expect(await screen.findByPlaceholderText("Password")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("URL")).toBeInTheDocument();
    expect(screen.queryByPlaceholderText("Username")).not.toBeInTheDocument();
  });

  it("fetches an existing secret only after an explicit action", async () => {
    selectEntry(passwordEntry());
    render(ItemDetail);

    const reveal = await screen.findByRole("button", { name: "Reveal password" });
    expect(mocks.entryRevealField).not.toHaveBeenCalled();

    await fireEvent.click(reveal);
    await waitFor(() => {
      expect(mocks.entryRevealField).toHaveBeenCalledWith("password-1", "password");
    });
    expect(screen.getByText("stored-password")).toBeInTheDocument();
  });

  it("saves password, URL, notes, and tags", async () => {
    const entry = passwordEntry();
    selectEntry(entry);
    mocks.entryUpdate.mockResolvedValue(entry);
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Edit entry" }));
    await waitFor(() =>
      expect(screen.getByPlaceholderText("Password")).toHaveValue("stored-password"),
    );

    await fireEvent.input(screen.getByPlaceholderText("Password"), {
      target: { value: "updated-password" },
    });
    await fireEvent.input(screen.getByPlaceholderText("URL"), {
      target: { value: "https://updated.example.com" },
    });
    await fireEvent.input(screen.getByPlaceholderText("Notes"), {
      target: { value: "Updated note" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mocks.entryUpdate).toHaveBeenCalledOnce());
    expect(mocks.entryUpdate).toHaveBeenCalledWith(
      "password-1",
      expect.objectContaining({
        password: "updated-password",
        url: "https://updated.example.com",
        notes: "Updated note",
        tags: ["work"],
      }),
    );
  });

  it("supports password copy and opening or copying the URL", async () => {
    selectEntry(passwordEntry());
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Copy password" }));
    expect(mocks.entryCopyField).toHaveBeenCalledWith("password-1", "password", 0, undefined);

    await fireEvent.click(screen.getAllByRole("button", { name: "https://example.com" })[0]);
    expect(mocks.openUrl).toHaveBeenCalledWith("https://example.com");

    await fireEvent.click(screen.getByRole("button", { name: "Copy URL" }));
    expect(mocks.clipboardCopy).toHaveBeenCalledWith("https://example.com");
  });

  it("cancels editing without persisting changes", async () => {
    selectEntry(passwordEntry());
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Edit entry" }));
    await waitFor(() => expect(screen.getByPlaceholderText("Password")).toBeInTheDocument());
    await fireEvent.input(screen.getByPlaceholderText("Password"), {
      target: { value: "do-not-save" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));

    expect(mocks.entryUpdate).not.toHaveBeenCalled();
  });
});
