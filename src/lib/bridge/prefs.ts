import { invoke } from "@tauri-apps/api/core";

export interface Preferences {
  lastVault: string | null;
  recentVaults: string[];
  idleLockMinutes: number;
  clipboardClearSeconds: number;
}

export async function prefsGet(): Promise<Preferences> {
  return invoke<Preferences>("prefs_get");
}

export async function prefsSetLastVault(path: string): Promise<void> {
  return invoke<void>("prefs_set_last_vault", { path });
}

export async function prefsSetSecurity(
  idleLockMinutes: number,
  clipboardClearSeconds: number,
): Promise<void> {
  return invoke<void>("prefs_set_security", { idleLockMinutes, clipboardClearSeconds });
}
