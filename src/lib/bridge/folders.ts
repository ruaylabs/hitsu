import { invoke } from "@tauri-apps/api/core";
import type { FolderSummary } from "./types";

export async function folderCreate(parentId: string | null, name: string): Promise<FolderSummary> {
  return invoke<FolderSummary>("folder_create", { parentId, name });
}

export async function folderRename(id: string, name: string): Promise<FolderSummary> {
  return invoke<FolderSummary>("folder_rename", { id, name });
}
