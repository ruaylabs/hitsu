export type ItemType =
  | "login"
  | "password"
  | "note"
  | "identity"
  | "card"
  | "software_license"
  | "passport";

/**
 * Detail view of an entry. Carries NO secret material: the backend reduces
 * passwords, TOTP URIs, card numbers, passport numbers, CVVs and PINs to
 * presence flags or masked values. Fetch plaintext on demand with `entryRevealField`, or copy
 * it without ever seeing it via `entryCopyField`.
 */
export interface Entry {
  id: string;
  type: ItemType;
  title: string;
  subtitle: string;
  url?: string;
  username?: string;
  hasPassword: boolean;
  hasTotp: boolean;
  notes?: string;
  tags: string[];
  favorite: boolean;
  trashed?: boolean;
  folderId?: string;
  iconHint?: string;
  identity?: IdentityFields;
  card?: CardFields;
  softwareLicense?: SoftwareLicenseFields;
  passport?: PassportFields;
  attachments: AttachmentMeta[];
  customFields: CustomField[];
  expiresAt?: string;
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

export interface PassportFields {
  type?: string;
  issuingCountry?: string;
  hasNumber: boolean;
  fullName?: string;
  sex?: string;
  nationality?: string;
  issuingAuthority?: string;
  birthDate?: string;
  birthPlace?: string;
  issueDate?: string;
  expiryDate?: string;
}

export interface SoftwareLicenseFields {
  version?: string;
  hasLicenseKey: boolean;
  licensedTo?: string;
  registeredEmail?: string;
  company?: string;
  downloadPage?: string;
  publisher?: string;
  website?: string;
  retailPrice?: string;
  supportEmail?: string;
  purchaseDate?: string;
  orderNumber?: string;
  orderTotal?: string;
}

export interface CardFields {
  holder?: string;
  /** Pre-masked by the backend, e.g. "4111 •••• 1111". */
  numberMasked?: string;
  type?: string;
  expMonth?: number;
  expYear?: number;
  hasNumber: boolean;
  hasCvv: boolean;
  hasPin: boolean;
}

/** A secret field that can be revealed or copied on demand. */
export type SecretField =
  | "password"
  | "totp"
  | "cardNumber"
  | "cardCvv"
  | "cardPin"
  | "licenseKey"
  | "passportNumber";

export interface FolderSummary {
  id: string;
  name: string;
  parentId?: string;
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
  trashed?: boolean;
  folderId?: string;
  iconHint?: string;
}

export interface VaultMeta {
  path: string;
  name: string;
  itemCount: number;
  syncProvider: "icloud" | "dropbox" | "local" | "unknown";
  /** When true, the vault's KDF memory is below 64 MiB and should be upgraded. */
  kdfNeedsUpgrade?: boolean;
  /** Entry summaries returned inline from vault_open to avoid a second round-trip. */
  entries: EntrySummary[];
  folders: FolderSummary[];
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
  | { kind: "tag"; tag: string }
  | { kind: "folder"; folderId: string };

export type AppView = "unlock" | "main" | "settings";
