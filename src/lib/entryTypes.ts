import type { ItemType } from "$lib/bridge/types";

interface EntryTypeMetadata {
  type: ItemType;
  label: string;
  pluralLabel: string;
  description: string;
  icon: string;
  color: string;
}

export const ENTRY_TYPES = [
  {
    type: "login",
    label: "Login",
    pluralLabel: "Logins",
    description: "Website account with a username and password",
    icon: "key",
    color: "#378add",
  },
  {
    type: "password",
    label: "Password",
    pluralLabel: "Passwords",
    description: "Standalone password or secret without a username",
    icon: "lock",
    color: "#d85a30",
  },
  {
    type: "note",
    label: "Note",
    pluralLabel: "Notes",
    description: "Free-form private note or reference information",
    icon: "notes",
    color: "#a1a09a",
  },
  {
    type: "identity",
    label: "Identity",
    pluralLabel: "Identities",
    description: "Personal, contact, and address information",
    icon: "user",
    color: "#7f77dd",
  },
  {
    type: "card",
    label: "Card",
    pluralLabel: "Cards",
    description: "Payment card number, expiry date, and PIN",
    icon: "credit-card",
    color: "#1d9e75",
  },
  {
    type: "software_license",
    label: "Software License",
    pluralLabel: "Software Licenses",
    description: "License key, purchase, and support details",
    icon: "license",
    color: "#ba7517",
  },
  {
    type: "passport",
    label: "Passport",
    pluralLabel: "Passports",
    description: "Passport number and travel document details",
    icon: "e-passport",
    color: "#0f6e56",
  },
  {
    type: "pgp_key",
    label: "PGP Key",
    pluralLabel: "PGP Keys",
    description: "Public and private PGP key material",
    icon: "fingerprint",
    color: "#6e3fa8",
  },
] as const satisfies readonly EntryTypeMetadata[];

export const ENTRY_TYPE_BY_TYPE = Object.fromEntries(
  ENTRY_TYPES.map((metadata) => [metadata.type, metadata]),
) as Record<ItemType, EntryTypeMetadata>;
