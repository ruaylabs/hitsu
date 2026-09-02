import Foundation
import SwiftUI

enum VaultEntryCategory: String, CaseIterable, Hashable, Sendable {
  case login
  case password
  case note
  case identity
  case card
  case softwareLicense = "software_license"
  case passport
  case pgpKey = "pgp_key"

  init(databaseValue: String?) {
    self = databaseValue.flatMap(Self.init(rawValue:)) ?? .login
  }

  var title: String {
    switch self {
    case .login: "Logins"
    case .password: "Passwords"
    case .note: "Notes"
    case .identity: "Identities"
    case .card: "Cards"
    case .softwareLicense: "Software Licenses"
    case .passport: "Passports"
    case .pgpKey: "PGP Keys"
    }
  }

  var symbolName: String {
    switch self {
    case .login: "globe"
    case .password: "key.fill"
    case .note: "note.text"
    case .identity: "person.crop.circle.fill"
    case .card: "creditcard.fill"
    case .softwareLicense: "opticaldisc.fill"
    case .passport: "airplane"
    case .pgpKey: "lock.doc.fill"
    }
  }

  var tint: Color {
    switch self {
    case .login: .blue
    case .password: .orange
    case .note: .teal
    case .identity: .indigo
    case .card: .green
    case .softwareLicense: .purple
    case .passport: .cyan
    case .pgpKey: .pink
    }
  }
}

struct VaultField: Identifiable, Hashable, Sendable {
  let name: String
  let isProtected: Bool

  var id: String { name }
}

/// One row in a category-specific detail section (card, identity, passport,
/// license, PGP key). Mirrors the desktop app's typed detail layouts: values
/// are looked up on demand from the vault store, and secret rows (card number,
/// CVV, PIN, license key, passport number, PGP private key) always require an
/// explicit reveal, matching the desktop's masked-by-default behavior.
struct VaultTypedField: Identifiable, Hashable, Sendable {
  let label: String
  /// KDBX key backing the row; nil when `displayValue` carries the value.
  let field: String?
  let isProtected: Bool
  /// Pre-rendered value for composite rows (e.g. card expiry "03/2027").
  let displayValue: String?

  var id: String { field ?? label }
}

/// One file attached to an entry. Payload bytes stay in the vault store and
/// are materialized to a temp file only when the user opens a preview.
struct VaultAttachment: Identifiable, Hashable, Sendable {
  /// Position within the entry's resolved attachment list; the store keys
  /// payload lookups by it.
  let index: Int
  let name: String
  let byteCount: Int

  var id: Int { index }
}

func formatAttachmentSize(_ bytes: Int) -> String {
  if bytes < 1024 { return "\(bytes) B" }
  if bytes < 1024 * 1024 { return String(format: "%.1f KB", Double(bytes) / 1024) }
  return String(format: "%.1f MB", Double(bytes) / (1024 * 1024))
}

struct VaultEntryIcon: Hashable, Sendable {
  let standardID: UInt32
  let customData: Data?
}

struct VaultEntry: Identifiable, Hashable, Sendable {
  let id: UUID
  let icon: VaultEntryIcon
  let title: String
  let username: String
  let url: String
  let isTitleProtected: Bool
  let isUsernameProtected: Bool
  let isURLProtected: Bool
  let isNotesProtected: Bool
  let groupPath: String
  let category: VaultEntryCategory
  let isFavorite: Bool
  let hasPassword: Bool
  let hasTOTP: Bool
  let hasNotes: Bool
  let fields: [VaultField]
  let typedFields: [VaultTypedField]
  let attachments: [VaultAttachment]
  let tags: [String]

  var displayTitle: String {
    if isTitleProtected { return "Protected entry" }
    return title.isEmpty ? "Untitled entry" : title
  }

  var secondaryText: String {
    if !username.isEmpty { return username }
    if isUsernameProtected { return "Protected username" }
    if !groupPath.isEmpty { return groupPath }
    return "No username"
  }

  func matchesSearch(_ searchText: String) -> Bool {
    let query = searchText.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !query.isEmpty else { return true }
    return displayTitle.localizedCaseInsensitiveContains(query)
      || (!isUsernameProtected && username.localizedCaseInsensitiveContains(query))
      || groupPath.localizedCaseInsensitiveContains(query)
      || category.title.localizedCaseInsensitiveContains(query)
      || tags.contains { $0.localizedCaseInsensitiveContains(query) }
  }
}
