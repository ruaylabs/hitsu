import { invoke } from "@tauri-apps/api/core";

export interface PasswordOptions {
  length: number;
  uppercase: boolean;
  lowercase: boolean;
  digits: boolean;
  symbols: boolean;
  excludeLookalikes: boolean;
}

export async function generatePassword(opts: PasswordOptions): Promise<string> {
  return invoke<string>("generate_password", { opts });
}
