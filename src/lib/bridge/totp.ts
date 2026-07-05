import { invoke } from "@tauri-apps/api/core";

export interface TotpCode {
  code: string;
  remaining: number;
  period: number;
}

/**
 * Compute the current TOTP code for an entry. The otpauth:// URI (the
 * long-lived seed) is read backend-side and never crosses IPC — only the
 * ephemeral code does.
 */
export async function totpCompute(id: string): Promise<TotpCode> {
  return invoke<TotpCode>("totp_compute", { id });
}
