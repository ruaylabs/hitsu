import type { ItemType } from "$lib/bridge/types";

export interface EntryTypeMetadata {
  type: ItemType;
  label: string;
  pluralLabel: string;
  icon: string;
  color: string;
}

export const ENTRY_TYPES = [
  {
    type: "login",
    label: "Login",
    pluralLabel: "Logins",
    icon: "key",
    color: "#378add",
  },
  {
    type: "password",
    label: "Password",
    pluralLabel: "Passwords",
    icon: "lock",
    color: "#d85a30",
  },
  {
    type: "note",
    label: "Note",
    pluralLabel: "Notes",
    icon: "notes",
    color: "#a1a09a",
  },
  {
    type: "identity",
    label: "Identity",
    pluralLabel: "Identities",
    icon: "user",
    color: "#7f77dd",
  },
  {
    type: "card",
    label: "Card",
    pluralLabel: "Cards",
    icon: "credit-card",
    color: "#1d9e75",
  },
  {
    type: "software_license",
    label: "Software License",
    pluralLabel: "Software Licenses",
    icon: "license",
    color: "#ba7517",
  },
  {
    type: "passport",
    label: "Passport",
    pluralLabel: "Passports",
    icon: "e-passport",
    color: "#0f6e56",
  },
] as const satisfies readonly EntryTypeMetadata[];

export const ENTRY_TYPE_BY_TYPE = Object.fromEntries(
  ENTRY_TYPES.map((metadata) => [metadata.type, metadata]),
) as Record<ItemType, EntryTypeMetadata>;
