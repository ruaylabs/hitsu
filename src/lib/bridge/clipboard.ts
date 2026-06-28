import { invoke } from "@tauri-apps/api/core";

export async function clipboardCopy(value: string): Promise<void> {
  return invoke<void>("clipboard_copy", { value });
}

export async function clipboardCopyWithTimeout(value: string, timeoutSecs: number): Promise<void> {
  return invoke<void>("clipboard_copy_with_timeout", { value, timeoutSecs });
}

export async function clipboardClear(): Promise<void> {
  return invoke<void>("clipboard_clear");
}
