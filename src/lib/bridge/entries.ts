import { invoke } from "@tauri-apps/api/core";
import type { AttachmentMeta, CustomField, Entry, EntrySummary, SecretField } from "./types";

/** Convert a full Entry to its safe summary (for the list store). */
export function toSummary(entry: Entry): EntrySummary {
  return {
    id: entry.id,
    type: entry.type,
    title: entry.title,
    subtitle: entry.subtitle,
    url: entry.url,
    username: entry.username,
    tags: entry.tags,
    favorite: entry.favorite,
    trashed: entry.trashed,
    iconHint: entry.iconHint,
  };
}

export async function entryGet(id: string): Promise<Entry> {
  return invoke<Entry>("entry_get", { id });
}

/**
 * Fetch a secret field's plaintext. Only call on explicit user action
 * (reveal button, populating the edit form) — never eagerly.
 * Pass `version` to read from a history revision.
 */
export async function entryRevealField(
  id: string,
  field: SecretField,
  version?: number,
): Promise<string> {
  return invoke<string>("entry_reveal_field", { id, field, version: version ?? null });
}

/**
 * Copy a secret field to the clipboard entirely inside the Rust backend —
 * the plaintext never reaches the webview. `timeoutSecs = 0` disables
 * auto-clear. Pass `version` to copy from a history revision.
 */
export async function entryCopyField(
  id: string,
  field: SecretField,
  timeoutSecs: number,
  version?: number,
): Promise<void> {
  return invoke<void>("entry_copy_field", { id, field, timeoutSecs, version: version ?? null });
}

export interface EntryDraft {
  title: string;
  username?: string | null;
  password?: string | null;
  url?: string | null;
  notes?: string | null;
  totp?: string | null;
}

export async function entryCreate(itemType: string, draft: EntryDraft): Promise<Entry> {
  return invoke<Entry>("entry_create", {
    itemType,
    draft: {
      title: draft.title,
      username: draft.username ?? null,
      password: draft.password ?? null,
      url: draft.url ?? null,
      notes: draft.notes ?? null,
      totp: draft.totp ?? null,
    },
  });
}

export interface EntryPatch {
  title?: string;
  username?: string | null;
  password?: string | null;
  url?: string | null;
  notes?: string | null;
  totp?: string | null;
  tags?: string[];
  favorite?: boolean;
  firstName?: string | null;
  lastName?: string | null;
  email?: string | null;
  phone?: string | null;
  address?: string | null;
  dob?: string | null;
  cardHolder?: string | null;
  cardNumber?: string | null;
  cardType?: string | null;
  cardExpMonth?: string | null;
  cardExpYear?: string | null;
  cardCvv?: string | null;
  cardPin?: string | null;
  licenseVersion?: string | null;
  licenseKey?: string | null;
  licenseLicensedTo?: string | null;
  licenseRegisteredEmail?: string | null;
  licenseCompany?: string | null;
  licenseDownloadPage?: string | null;
  licensePublisher?: string | null;
  licenseWebsite?: string | null;
  licenseRetailPrice?: string | null;
  licenseSupportEmail?: string | null;
  licensePurchaseDate?: string | null;
  licenseOrderNumber?: string | null;
  licenseOrderTotal?: string | null;
  passportType?: string | null;
  passportIssuingCountry?: string | null;
  passportNumber?: string | null;
  passportFullName?: string | null;
  passportSex?: string | null;
  passportNationality?: string | null;
  passportIssuingAuthority?: string | null;
  passportBirthDate?: string | null;
  passportBirthPlace?: string | null;
  passportIssueDate?: string | null;
  passportExpiryDate?: string | null;
  customFields?: CustomField[];
}

export async function entryUpdate(id: string, patch: EntryPatch): Promise<Entry> {
  return invoke<Entry>("entry_update", {
    id,
    patch: {
      title: patch.title ?? null,
      username: patch.username ?? null,
      password: patch.password ?? null,
      url: patch.url ?? null,
      notes: patch.notes ?? null,
      totp: patch.totp ?? null,
      tags: patch.tags ?? null,
      favorite: patch.favorite ?? null,
      firstName: patch.firstName ?? null,
      lastName: patch.lastName ?? null,
      email: patch.email ?? null,
      phone: patch.phone ?? null,
      address: patch.address ?? null,
      dob: patch.dob ?? null,
      cardHolder: patch.cardHolder ?? null,
      cardNumber: patch.cardNumber ?? null,
      cardType: patch.cardType ?? null,
      cardExpMonth: patch.cardExpMonth ?? null,
      cardExpYear: patch.cardExpYear ?? null,
      cardCvv: patch.cardCvv ?? null,
      cardPin: patch.cardPin ?? null,
      licenseVersion: patch.licenseVersion ?? null,
      licenseKey: patch.licenseKey ?? null,
      licenseLicensedTo: patch.licenseLicensedTo ?? null,
      licenseRegisteredEmail: patch.licenseRegisteredEmail ?? null,
      licenseCompany: patch.licenseCompany ?? null,
      licenseDownloadPage: patch.licenseDownloadPage ?? null,
      licensePublisher: patch.licensePublisher ?? null,
      licenseWebsite: patch.licenseWebsite ?? null,
      licenseRetailPrice: patch.licenseRetailPrice ?? null,
      licenseSupportEmail: patch.licenseSupportEmail ?? null,
      licensePurchaseDate: patch.licensePurchaseDate ?? null,
      licenseOrderNumber: patch.licenseOrderNumber ?? null,
      licenseOrderTotal: patch.licenseOrderTotal ?? null,
      passportType: patch.passportType ?? null,
      passportIssuingCountry: patch.passportIssuingCountry ?? null,
      passportNumber: patch.passportNumber ?? null,
      passportFullName: patch.passportFullName ?? null,
      passportSex: patch.passportSex ?? null,
      passportNationality: patch.passportNationality ?? null,
      passportIssuingAuthority: patch.passportIssuingAuthority ?? null,
      passportBirthDate: patch.passportBirthDate ?? null,
      passportBirthPlace: patch.passportBirthPlace ?? null,
      passportIssueDate: patch.passportIssueDate ?? null,
      passportExpiryDate: patch.passportExpiryDate ?? null,
      customFields: patch.customFields ?? null,
    },
  });
}

export interface HistoryEntrySummary {
  version: number;
  modifiedAt: string;
  title: string;
}

export async function entryHistoryList(id: string): Promise<HistoryEntrySummary[]> {
  return invoke<HistoryEntrySummary[]>("entry_history_list", { id });
}

export async function entryHistoryGet(id: string, version: number): Promise<Entry> {
  return invoke<Entry>("entry_history_get", { id, version });
}

export async function entryRevealCustomField(id: string, name: string): Promise<string> {
  return invoke<string>("entry_reveal_custom_field", { id, name });
}

export async function entryCopyCustomField(
  id: string,
  name: string,
  timeoutSecs: number,
): Promise<void> {
  return invoke<void>("entry_copy_custom_field", { id, name, timeoutSecs });
}

export async function entryDelete(id: string): Promise<void> {
  return invoke<void>("entry_delete", { id });
}

export async function entryRestore(id: string): Promise<void> {
  return invoke<void>("entry_restore", { id });
}

export async function entryDeletePermanent(id: string): Promise<void> {
  return invoke<void>("entry_delete_permanent", { id });
}

/** Drop a brand-new, never-persisted entry from memory without touching disk. */
export async function entryDiscard(id: string): Promise<void> {
  return invoke<void>("entry_discard", { id });
}

/** Ask the Rust backend to choose a destination and save an attachment.
 * Returns bytes written, or null when the native dialog is cancelled. */
export async function entryAttachmentSave(id: string, name: string): Promise<number | null> {
  return invoke<number | null>("entry_attachment_save", { id, name });
}

/** Add an attachment to an entry. `dataB64` is base64-encoded binary data. */
export async function entryAttachmentAdd(
  id: string,
  name: string,
  dataB64: string,
): Promise<AttachmentMeta> {
  return invoke<AttachmentMeta>("entry_attachment_add", { id, name, dataB64 });
}

/** Remove an attachment from an entry. */
export async function entryAttachmentRemove(id: string, name: string): Promise<void> {
  return invoke<void>("entry_attachment_remove", { id, name });
}
