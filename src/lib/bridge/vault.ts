import { invoke } from "@tauri-apps/api/core";
import type { VaultMeta } from "./types";

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
