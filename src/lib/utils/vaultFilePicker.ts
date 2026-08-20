import { open, save } from "@tauri-apps/plugin-dialog";
import { nativeDialog } from "$lib/stores/nativeDialog.svelte";

const KDBX_FILTER = { name: "KeePass Database", extensions: ["kdbx"] };

/** Choose an existing KeePass database while keeping the privacy screen suppressed. */
export function pickVaultToOpen(): Promise<string | null> {
  return nativeDialog.during(() =>
    open({
      multiple: false,
      filters: [KDBX_FILTER],
    }),
  );
}

/** Choose a destination for a new KeePass database. */
export function pickVaultToCreate(): Promise<string | null> {
  return nativeDialog.during(() =>
    save({
      filters: [KDBX_FILTER],
      defaultPath: "vault.kdbx",
    }),
  );
}
