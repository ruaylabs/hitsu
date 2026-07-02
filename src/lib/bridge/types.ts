export type ItemType = "login" | "note" | "identity" | "card";

export interface Entry {
  id: string;
  type: ItemType;
  title: string;
  subtitle: string;
  url?: string;
  username?: string;
  password?: string;
  totp?: string;
  notes?: string;
  tags: string[];
  favorite: boolean;
  iconHint?: string;
  identity?: IdentityFields;
  card?: CardFields;
  attachments: AttachmentMeta[];
  customFields: CustomField[];
  modifiedAt: string;
  createdAt: string;
  historyCount: number;
}

export interface CustomField {
  name: string;
  value: string;
  protected: boolean;
}

export interface AttachmentMeta {
  id: string;
  name: string;
  sizeBytes: number;
}

export interface IdentityFields {
  firstName?: string;
  lastName?: string;
  email?: string;
  phone?: string;
  address?: string;
  dob?: string;
}

export interface CardFields {
  holder?: string;
  number?: string;
  type?: string;
  expMonth?: number;
  expYear?: number;
  cvv?: string;
  pin?: string;
}

export interface EntrySummary {
  id: string;
  type: ItemType;
  title: string;
  subtitle: string;
  url?: string;
  username?: string;
  tags: string[];
  favorite: boolean;
  iconHint?: string;
}

export interface VaultMeta {
  path: string;
  name: string;
  itemCount: number;
  syncProvider: "icloud" | "dropbox" | "local" | "unknown";
  /** When true, the vault's KDF memory is below 64 MiB and should be upgraded. */
  kdfNeedsUpgrade?: boolean;
}

export interface Toast {
  id: string;
  kind: "info" | "success" | "warning" | "danger";
  message: string;
  durationMs: number;
}

export type SidebarFilter =
  | { kind: "all" }
  | { kind: "favorites" }
  | { kind: "trash" }
  | { kind: "type"; type: ItemType }
  | { kind: "tag"; tag: string };

export type AppView = "unlock" | "main" | "settings";
