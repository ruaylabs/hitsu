import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Entry, EntrySummary } from "$lib/bridge/types";
import { clipboard } from "$lib/stores/clipboard.svelte";
import { features } from "$lib/stores/features.svelte";
import { saveStatus } from "$lib/stores/saveStatus.svelte";
import { selection } from "$lib/stores/selection.svelte";
import { vault } from "$lib/stores/vault.svelte";
import { tagColor } from "$lib/utils/tagColor";
import ItemDetail from "./ItemDetail.svelte";

const mocks = vi.hoisted(() => ({
  entryGet: vi.fn(),
  entryEditPayload: vi.fn(),
  entryRevealField: vi.fn(),
  entryUpdate: vi.fn(),
  entryMove: vi.fn(),
  entryDiscard: vi.fn(),
  entryDelete: vi.fn(),
  entryCopyField: vi.fn(),
  entryRevealCustomField: vi.fn(),
  entryCopyCustomField: vi.fn(),
  entryAttachmentRemove: vi.fn(),
  folderCreate: vi.fn(),
  folderRename: vi.fn(),
  clipboardCopy: vi.fn(),
  clipboardClear: vi.fn(),
  openUrl: vi.fn(),
}));

vi.mock("$lib/bridge/entries", () => ({
  entryGet: mocks.entryGet,
  entryEditPayload: mocks.entryEditPayload,
  entryRevealField: mocks.entryRevealField,
  entryUpdate: mocks.entryUpdate,
  entryMove: mocks.entryMove,
  entryDiscard: mocks.entryDiscard,
  entryDelete: mocks.entryDelete,
  entryCopyField: mocks.entryCopyField,
  entryRevealCustomField: mocks.entryRevealCustomField,
  entryCopyCustomField: mocks.entryCopyCustomField,
  entryAttachmentRemove: mocks.entryAttachmentRemove,
  toSummary: (entry: Entry): EntrySummary => ({
    id: entry.id,
    type: entry.type,
    title: entry.title,
    subtitle: entry.subtitle,
    url: entry.url,
    username: entry.username,
    tags: entry.tags,
    favorite: entry.favorite,
    folderId: entry.folderId,
    iconHint: entry.iconHint,
  }),
}));

vi.mock("$lib/bridge/folders", () => ({
  folderCreate: mocks.folderCreate,
  folderRename: mocks.folderRename,
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
    folderId: entry.folderId,
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
  vault.setFolders([]);
  features.hydrate({
    lastVault: null,
    recentVaults: [],
    idleLockMinutes: 5,
    clipboardClearSeconds: 15,
    foldersEnabled: false,
    browserIntegrationEnabled: false,
    kdfUpgradeDismissedVaults: [],
  });
  vault.setCreatingId(null);
  vault.setEditingId(null);
  clipboard.defaultTimeoutSecs = 0;
  saveStatus.markSaved();
  mocks.entryEditPayload.mockResolvedValue({
    password: "stored-password",
    totp: "",
    cardNumber: "",
    cardCvv: "",
    cardPin: "",
    licenseKey: "stored-password",
    passportNumber: "stored-password",
    customFields: [],
  });
  mocks.entryRevealField.mockResolvedValue("stored-password");
  mocks.entryDiscard.mockResolvedValue(undefined);
  mocks.entryMove.mockResolvedValue(passwordEntry());
  mocks.entryCopyField.mockResolvedValue(undefined);
  mocks.entryRevealCustomField.mockResolvedValue("protected-value");
  mocks.entryCopyCustomField.mockResolvedValue(undefined);
  mocks.entryAttachmentRemove.mockResolvedValue(undefined);
  mocks.folderCreate.mockResolvedValue({ id: "new-folder", name: "New folder" });
  mocks.folderRename.mockResolvedValue({ id: "new-folder", name: "Renamed folder" });
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
    await fireEvent.input(screen.getByPlaceholderText("Notes"), {
      target: { value: "Trigger a save" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText("Vault changed on disk")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeInTheDocument();
    expect(saveStatus.state).toBe("error");
    consoleError.mockRestore();
  });
});

describe("edit actions", () => {
  it("enables Save only after the form changes", async () => {
    selectEntry(passwordEntry());
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Edit entry" }));
    const save = await screen.findByRole("button", { name: "Save" });

    expect(save).toBeDisabled();
    await fireEvent.keyDown(window, { key: "s", metaKey: true });
    expect(mocks.entryUpdate).not.toHaveBeenCalled();
    await fireEvent.input(screen.getByPlaceholderText("Title"), {
      target: { value: "Updated title" },
    });
    expect(save).toBeEnabled();
  });
});

describe("tag colors", () => {
  it("applies the stable palette color to detail badges", async () => {
    selectEntry(passwordEntry({ tags: ["finance"] }));
    render(ItemDetail);

    const badge = await screen.findByText("finance");
    expect(badge).toHaveStyle(`--tag-color: ${tagColor("finance")}`);
  });
});

describe("custom fields", () => {
  it("displays and reveals protected and unprotected fields", async () => {
    selectEntry(
      passwordEntry({
        customFields: [
          { name: "Environment", value: "Production", protected: false },
          { name: "API key", value: "", protected: true },
        ],
      }),
    );
    render(ItemDetail);

    expect(await screen.findByText("Production")).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: "Reveal api key" }));

    expect(mocks.entryRevealCustomField).toHaveBeenCalledWith("password-1", "API key");
    expect(await screen.findByText("protected-value")).toBeInTheDocument();
  });

  it("adds a protected custom field while editing", async () => {
    const entry = passwordEntry();
    selectEntry(entry);
    mocks.entryUpdate.mockResolvedValue(entry);
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Edit entry" }));
    await fireEvent.click(await screen.findByRole("button", { name: "Add field" }));
    await fireEvent.input(screen.getByRole("textbox", { name: "Custom field name" }), {
      target: { value: "API key" },
    });
    await fireEvent.input(screen.getByLabelText("Custom field value"), {
      target: { value: "secret" },
    });
    await fireEvent.click(screen.getByRole("checkbox", { name: "Protect custom field" }));
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mocks.entryUpdate).toHaveBeenCalledOnce());
    expect(mocks.entryUpdate).toHaveBeenCalledWith("password-1", {
      customFields: [{ name: "API key", value: "secret", protected: true }],
    });
  });
});

describe("entry refreshes", () => {
  it("debounces detail fetches during rapid keyboard navigation", async () => {
    vi.useFakeTimers();
    const first = passwordEntry({ id: "password-1", title: "First" });
    const second = passwordEntry({ id: "password-2", title: "Second" });
    mocks.entryGet.mockResolvedValue(second);
    vault.setEntries([summary(first), summary(second)]);
    render(ItemDetail);

    selection.select(first.id, "keyboard");
    await tick();
    selection.select(second.id, "keyboard");
    await tick();
    expect(mocks.entryGet).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(80);
    expect(mocks.entryGet).toHaveBeenCalledOnce();
    expect(mocks.entryGet).toHaveBeenCalledWith(second.id);
    vi.useRealTimers();
  });

  it("ignores an attachment refresh after selection changes", async () => {
    const first = passwordEntry({
      attachments: [{ id: "notes.txt", name: "notes.txt", sizeBytes: 12 }],
    });
    const second = passwordEntry({ id: "password-2", title: "Second password" });
    selectEntry(first);
    render(ItemDetail);

    expect(await screen.findByRole("heading", { name: first.title })).toBeInTheDocument();

    let resolveRefresh!: (entry: Entry) => void;
    const refresh = new Promise<Entry>((resolve) => {
      resolveRefresh = resolve;
    });
    mocks.entryGet.mockReturnValueOnce(refresh);

    await fireEvent.click(screen.getByRole("button", { name: "Remove notes.txt" }));
    await fireEvent.click(screen.getByRole("button", { name: /^Remove$/ }));
    await waitFor(() => expect(mocks.entryGet).toHaveBeenCalledTimes(2));

    mocks.entryGet.mockResolvedValue(second);
    vault.setEntries([summary(first), summary(second)]);
    selection.selectedId = second.id;
    expect(await screen.findByRole("heading", { name: second.title })).toBeInTheDocument();

    resolveRefresh(first);
    await waitFor(() => {
      expect(screen.getByRole("heading", { name: second.title })).toBeInTheDocument();
    });
  });
});

describe("identity entry workflow", () => {
  it("displays and saves the date of birth", async () => {
    const entry = passwordEntry({
      id: "identity-1",
      type: "identity",
      title: "Alice Example",
      url: undefined,
      hasPassword: false,
      identity: {
        firstName: "Alice",
        lastName: "Example",
        dob: "1990-01-02",
      },
    });
    const updated = {
      ...entry,
      identity: { ...entry.identity, dob: "1991-03-04" },
    };
    selectEntry(entry);
    mocks.entryUpdate.mockResolvedValue(updated);
    render(ItemDetail);

    expect(await screen.findByText("1990-01-02")).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: "Edit entry" }));

    const dob = await screen.findByLabelText("Date of birth");
    expect(dob).toHaveValue("1990-01-02");
    await fireEvent.input(dob, { target: { value: "1991-03-04" } });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mocks.entryUpdate).toHaveBeenCalledOnce());
    expect(mocks.entryUpdate).toHaveBeenCalledWith("identity-1", { dob: "1991-03-04" });
    expect(await screen.findByText("1991-03-04")).toBeInTheDocument();
  });
});

describe("software license workflow", () => {
  it("reveals and saves license fields", async () => {
    const entry = passwordEntry({
      id: "license-1",
      type: "software_license",
      title: "Editor Pro",
      url: undefined,
      hasPassword: false,
      softwareLicense: {
        version: "4.2",
        hasLicenseKey: true,
        licensedTo: "Ada",
        registeredEmail: "ada@example.com",
        purchaseDate: "2024-01-01",
      },
    });
    selectEntry(entry);
    mocks.entryUpdate.mockResolvedValue(entry);
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Reveal license key" }));
    expect(mocks.entryRevealField).toHaveBeenCalledWith("license-1", "licenseKey");

    await fireEvent.click(screen.getByRole("button", { name: "Edit entry" }));
    await waitFor(() =>
      expect(screen.getByPlaceholderText("License key")).toHaveValue("stored-password"),
    );
    const licenseKey = screen.getByPlaceholderText("License key");
    expect(licenseKey).toHaveAttribute("type", "password");
    await fireEvent.click(screen.getByRole("button", { name: "Reveal license key" }));
    expect(licenseKey).toHaveAttribute("type", "text");
    await fireEvent.input(screen.getByPlaceholderText("Version"), { target: { value: "5.0" } });
    await fireEvent.input(screen.getByPlaceholderText("License key"), {
      target: { value: "NEW-LICENSE-KEY" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mocks.entryUpdate).toHaveBeenCalledOnce());
    expect(mocks.entryUpdate).toHaveBeenCalledWith("license-1", {
      licenseVersion: "5.0",
      licenseKey: "NEW-LICENSE-KEY",
    });
  });
});

describe("passport workflow", () => {
  it("reveals and saves passport fields", async () => {
    const entry = passwordEntry({
      id: "passport-1",
      type: "passport",
      title: "US Passport",
      url: undefined,
      hasPassword: false,
      passport: {
        type: "Passport",
        issuingCountry: "United States",
        hasNumber: true,
        fullName: "Ada Lovelace",
        birthDate: "1815-12-10",
        expiryDate: "2030-01-01",
      },
    });
    selectEntry(entry);
    mocks.entryUpdate.mockResolvedValue(entry);
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Reveal number" }));
    expect(mocks.entryRevealField).toHaveBeenCalledWith("passport-1", "passportNumber");

    await fireEvent.click(screen.getByRole("button", { name: "Edit entry" }));
    await waitFor(() =>
      expect(screen.getByPlaceholderText("Passport number")).toHaveValue("stored-password"),
    );
    const passportNumber = screen.getByPlaceholderText("Passport number");
    expect(passportNumber).toHaveAttribute("type", "password");
    await fireEvent.click(screen.getByRole("button", { name: "Reveal passport number" }));
    expect(passportNumber).toHaveAttribute("type", "text");
    await fireEvent.input(screen.getByPlaceholderText("Passport number"), {
      target: { value: "NEW-PASSPORT-NUMBER" },
    });
    await fireEvent.input(screen.getByLabelText("Passport expiry date"), {
      target: { value: "2035-01-01" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mocks.entryUpdate).toHaveBeenCalledOnce());
    expect(mocks.entryUpdate).toHaveBeenCalledWith("passport-1", {
      passportNumber: "NEW-PASSPORT-NUMBER",
      passportExpiryDate: "2035-01-01",
    });
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
    const password = screen.getByPlaceholderText("Password");
    expect(password).toHaveAttribute("type", "password");
    await fireEvent.click(screen.getByRole("button", { name: "Reveal password" }));
    expect(password).toHaveAttribute("type", "text");

    await fireEvent.input(password, { target: { value: "updated-password" } });
    await fireEvent.input(screen.getByPlaceholderText("URL"), {
      target: { value: "https://updated.example.com" },
    });
    await fireEvent.input(screen.getByPlaceholderText("Notes"), {
      target: { value: "Updated note" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mocks.entryUpdate).toHaveBeenCalledOnce());
    expect(mocks.entryUpdate).toHaveBeenCalledWith("password-1", {
      password: "updated-password",
      url: "https://updated.example.com",
      notes: "Updated note",
    });
  });

  it("omits unchanged fields and secrets from the update patch", async () => {
    const entry = passwordEntry();
    selectEntry(entry);
    mocks.entryUpdate.mockResolvedValue({ ...entry, title: "Updated title" });
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Edit entry" }));
    await waitFor(() =>
      expect(screen.getByPlaceholderText("Password")).toHaveValue("stored-password"),
    );
    expect(mocks.entryEditPayload).toHaveBeenCalledOnce();
    expect(mocks.entryEditPayload).toHaveBeenCalledWith("password-1");
    expect(mocks.entryRevealField).not.toHaveBeenCalled();
    await fireEvent.input(screen.getByPlaceholderText("Title"), {
      target: { value: "Updated title" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mocks.entryUpdate).toHaveBeenCalledOnce());
    expect(mocks.entryUpdate).toHaveBeenCalledWith("password-1", { title: "Updated title" });
  });

  it("sends an empty secret only when the user clears it", async () => {
    const entry = passwordEntry();
    selectEntry(entry);
    mocks.entryUpdate.mockResolvedValue({ ...entry, hasPassword: false });
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Edit entry" }));
    const password = await screen.findByPlaceholderText("Password");
    await waitFor(() => expect(password).toHaveValue("stored-password"));
    await fireEvent.input(password, { target: { value: "" } });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mocks.entryUpdate).toHaveBeenCalledOnce());
    expect(mocks.entryUpdate).toHaveBeenCalledWith("password-1", { password: "" });
  });

  it("closes an unchanged edit without sending an update", async () => {
    selectEntry(passwordEntry());
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Edit entry" }));
    await waitFor(() =>
      expect(screen.getByPlaceholderText("Password")).toHaveValue("stored-password"),
    );
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(mocks.entryUpdate).not.toHaveBeenCalled();
    expect(await screen.findByRole("button", { name: "Edit entry" })).toBeInTheDocument();
  });

  it("saves a general entry expiration date", async () => {
    const entry = passwordEntry({ expiresAt: undefined });
    const updated = { ...entry, expiresAt: "2030-05-20" };
    selectEntry(entry);
    mocks.entryUpdate.mockResolvedValue(updated);
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Edit entry" }));
    const expiration = await screen.findByLabelText("Entry expiration date");
    await fireEvent.input(expiration, { target: { value: "2030-05-20" } });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mocks.entryUpdate).toHaveBeenCalledOnce());
    expect(mocks.entryUpdate).toHaveBeenCalledWith("password-1", {
      expiresAt: "2030-05-20",
    });
    expect(await screen.findByText(/Expires on/)).toBeInTheDocument();
  });

  it("shows a warning when an entry expiration is due", async () => {
    selectEntry(passwordEntry({ expiresAt: "2000-01-01" }));
    render(ItemDetail);

    const warning = await screen.findByText(/Expired on/);
    expect(warning.closest('[role="status"]')).toHaveClass("due");
  });

  it("moves an entry through the folder dialog when folders are enabled", async () => {
    features.hydrate({
      lastVault: null,
      recentVaults: [],
      idleLockMinutes: 5,
      clipboardClearSeconds: 15,
      foldersEnabled: true,
      browserIntegrationEnabled: false,
      kdfUpgradeDismissedVaults: [],
    });
    vault.setFolders([
      { id: "personal", name: "Personal" },
      { id: "work", name: "Work" },
    ]);
    const entry = passwordEntry({ folderId: "personal" });
    const moved = { ...entry, folderId: "work" };
    mocks.entryMove.mockResolvedValue(moved);
    selectEntry(entry);
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Move entry" }));
    const destination = screen.getByLabelText("Destination");
    await fireEvent.change(destination, { target: { value: "work" } });
    await fireEvent.click(screen.getByRole("button", { name: "Move" }));

    await waitFor(() => expect(mocks.entryMove).toHaveBeenCalledWith("password-1", "work"));
    expect(vault.entries[0].folderId).toBe("work");
    expect(screen.queryByRole("dialog", { name: "Move entry" })).not.toBeInTheDocument();
  });

  it("creates a nested destination from the move dialog", async () => {
    features.hydrate({
      lastVault: null,
      recentVaults: [],
      idleLockMinutes: 5,
      clipboardClearSeconds: 15,
      foldersEnabled: true,
      browserIntegrationEnabled: false,
      kdfUpgradeDismissedVaults: [],
    });
    vault.setFolders([{ id: "work", name: "Work" }]);
    selectEntry(passwordEntry());
    mocks.folderCreate.mockResolvedValue({ id: "clients", name: "Clients", parentId: "work" });
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Move entry" }));
    await fireEvent.change(screen.getByLabelText("Destination"), { target: { value: "work" } });
    await fireEvent.click(screen.getByRole("button", { name: "New folder" }));
    await fireEvent.input(screen.getByPlaceholderText("Folder name"), {
      target: { value: "Clients" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Create" }));

    await waitFor(() => expect(mocks.folderCreate).toHaveBeenCalledWith("work", "Clients"));
    expect(screen.getByLabelText("Destination")).toHaveValue("clients");
    expect(vault.folders).toContainEqual({ id: "clients", name: "Clients", parentId: "work" });
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

  it("shows a URL in its field but not in the entry header", async () => {
    selectEntry(passwordEntry());
    render(ItemDetail);

    expect(await screen.findAllByRole("button", { name: "https://example.com" })).toHaveLength(1);
  });

  it("prompts before Escape discards unsaved changes", async () => {
    selectEntry(passwordEntry());
    render(ItemDetail);

    await fireEvent.click(await screen.findByRole("button", { name: "Edit entry" }));
    const password = await screen.findByPlaceholderText("Password");
    await fireEvent.input(password, { target: { value: "do-not-discard" } });
    await fireEvent.keyDown(window, { key: "Escape" });

    expect(await screen.findByRole("dialog", { name: "Save changes?" })).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Password")).toHaveValue("do-not-discard");
    expect(mocks.entryUpdate).not.toHaveBeenCalled();
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
