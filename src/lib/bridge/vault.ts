import { invoke } from "@tauri-apps/api/core";
import type { EntrySummary, VaultMeta } from "./types";

export interface SkippedImportEntry {
  title: string;
  reason: string;
}

export interface ImportReport {
  importedItems: number;
  importedAttachments: number;
  skippedItems: number;
  skippedEntries: SkippedImportEntry[];
  entries: EntrySummary[];
}

export async function vaultOpen(path: string, password: string): Promise<VaultMeta> {
  return invoke<VaultMeta>("vault_open", { path, password });
}

export async function vaultCreate(
  path: string,
  password: string,
  name: string,
): Promise<VaultMeta> {
  return invoke<VaultMeta>("vault_create", { path, password, name });
}

export async function vaultChangePassword(oldPassword: string, newPassword: string): Promise<void> {
  return invoke<void>("vault_change_password", { oldPassword, newPassword });
}

export async function vaultLock(): Promise<void> {
  return invoke<void>("vault_lock");
}

export async function vaultUpgradeKdf(): Promise<void> {
  return invoke<void>("vault_upgrade_kdf");
}

/** Open a native picker and import a 1Password 7 .1pif export into the open vault. */
export async function vaultImport1pif(): Promise<ImportReport | null> {
  return invoke<ImportReport | null>("vault_import_1pif");
}
