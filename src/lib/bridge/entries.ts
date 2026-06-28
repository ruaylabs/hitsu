import { invoke } from "@tauri-apps/api/core";
import type { Entry } from "./types";

export interface EntrySummary {
  id: string;
  type: "login" | "note" | "identity" | "card";
  title: string;
  subtitle: string;
  favorite: boolean;
  iconHint?: string;
}

export async function entriesList(): Promise<EntrySummary[]> {
  return invoke<EntrySummary[]>("entries_list");
}

export async function entryGet(id: string): Promise<Entry> {
  return invoke<Entry>("entry_get", { id });
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
  cardHolder?: string | null;
  cardNumber?: string | null;
  cardType?: string | null;
  cardExpMonth?: string | null;
  cardExpYear?: string | null;
  cardCvv?: string | null;
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
      cardHolder: patch.cardHolder ?? null,
      cardNumber: patch.cardNumber ?? null,
      cardType: patch.cardType ?? null,
      cardExpMonth: patch.cardExpMonth ?? null,
      cardExpYear: patch.cardExpYear ?? null,
      cardCvv: patch.cardCvv ?? null,
    },
  });
}

export async function entryDelete(id: string): Promise<void> {
  return invoke<void>("entry_delete", { id });
}
