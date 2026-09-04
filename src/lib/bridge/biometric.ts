import { invoke } from "@tauri-apps/api/core";
import type { VaultMeta } from "./types";

export interface BiometricStatus {
  available: boolean;
  enabled: boolean;
}

export async function biometricStatus(path: string): Promise<BiometricStatus> {
  return invoke<BiometricStatus>("biometric_status", { path });
}

export async function biometricEnable(path: string, password: string): Promise<void> {
  return invoke<void>("biometric_enable", { path, password });
}

export async function biometricDisable(path: string): Promise<void> {
  return invoke<void>("biometric_disable", { path });
}

export async function biometricUnlock(path: string): Promise<VaultMeta> {
  return invoke<VaultMeta>("biometric_unlock", { path });
}
