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
