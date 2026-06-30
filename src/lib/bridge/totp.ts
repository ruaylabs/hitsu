import { invoke } from "@tauri-apps/api/core";

export interface TotpCode {
  code: string;
  remaining: number;
  period: number;
}

export async function totpCompute(uri: string): Promise<TotpCode> {
  return invoke<TotpCode>("totp_compute", { uri });
}
