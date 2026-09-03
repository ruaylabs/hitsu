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
  /// Raw KDBX field key, retained for looking up the value in the store.
  let name: String
  let isProtected: Bool

  /// User-facing custom field name without the storage prefix.
  var displayName: String {
    name.hasPrefix("custom.") ? String(name.dropFirst("custom.".count)) : name
  }

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
  /// Card-number rows show a masked preview before reveal and group the
  /// revealed digits for display.
  let isCardNumber: Bool

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

private let cardBrandNames = [
  "visa": "Visa",
  "mastercard": "Mastercard",
  "amex": "American Express",
  "discover": "Discover",
  "diners": "Diners Club",
  "jcb": "JCB",
  "unionpay": "UnionPay",
  "maestro": "Maestro",
]

/// Display name for a stored card type. Accepts either a canonical key
/// ("amex") or a legacy full name ("American Express"); unknown values are
/// returned unchanged so nothing is silently lost.
func cardBrandName(for type: String) -> String {
  cardBrandNames[type.lowercased()] ?? type
}

/// Groups a card number for display: American Express (by type or 34/37
/// prefix) as 4-6-5, everything else in groups of 4. Mirrors the desktop
/// formatter.
func formatCardNumber(_ raw: String, cardType: String?) -> String {
  let digits = String(raw.unicodeScalars.filter { (48...57).contains($0.value) })
  guard !digits.isEmpty else { return raw }

  let type = cardType?.lowercased() ?? ""
  let prefix = String(digits.prefix(2))
  let isAmex =
    type == "amex" || type == "american express"
    || (type.isEmpty && digits.count >= 2 && (prefix == "34" || prefix == "37"))

  let chars = Array(digits)
  if isAmex {
    var parts: [String] = []
    if chars.count > 0 { parts.append(String(chars[0..<min(4, chars.count)])) }
    if chars.count > 4 { parts.append(String(chars[4..<min(10, chars.count)])) }
    if chars.count > 10 { parts.append(String(chars[10..<min(15, chars.count)])) }
    return parts.joined(separator: " ")
  }

  var groups: [String] = []
  var start = digits.startIndex
  while start < digits.endIndex {
    let end = digits.index(start, offsetBy: 4, limitedBy: digits.endIndex) ?? digits.endIndex
    groups.append(String(digits[start..<end]))
    start = end
  }
  return groups.joined(separator: " ")
}

/// Masked preview for an unrevealed card number: first and last 4 digits for
/// typical lengths, otherwise a fixed bullet mask. Mirrors the desktop mask.
func maskCardNumber(_ value: String) -> String? {
  guard !value.isEmpty else { return nil }
  if value.count >= 12 && value.allSatisfy(\.isASCII) {
    return "\(value.prefix(4)) •••• \(value.suffix(4))"
  }
  return "••••"
}

/// One prior version of an entry. Field values stay in the vault store and
/// are only revealed on demand.
struct VaultHistoryItem: Identifiable, Hashable, Sendable {
  /// Position within the entry's history (oldest first); the store keys
  /// version lookups by it.
  let index: Int
  let lastModified: Date?

  var id: Int { index }
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
  /// True when the entry lives in the recycle bin (or a group nested inside
  /// it). Trashed entries stay out of the Favorites and Categories lists and
  /// are shown only in the read-only Trash tab.
  let isTrashed: Bool
  /// The last time the entry was modified, used to populate the Recent view.
  let lastModified: Date?
  /// The expiration date when KeePass expiration is enabled; nil otherwise.
  let expirationDate: Date?
  let hasPassword: Bool
  let hasTOTP: Bool
  let hasNotes: Bool
  let fields: [VaultField]
  let typedFields: [VaultTypedField]
  let attachments: [VaultAttachment]
  let history: [VaultHistoryItem]
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

  var isExpired: Bool {
    guard let expirationDate else { return false }
    let calendar = Calendar.current
    return calendar.startOfDay(for: expirationDate) <= calendar.startOfDay(for: Date())
  }

  func matchesSearch(_ searchText: String) -> Bool {
    let query = searchText.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !query.isEmpty else { return true }
    return displayTitle.localizedCaseInsensitiveContains(query)
      || (!isUsernameProtected && username.localizedCaseInsensitiveContains(query))
      || (!isURLProtected && url.localizedCaseInsensitiveContains(query))
      || groupPath.localizedCaseInsensitiveContains(query)
      || category.title.localizedCaseInsensitiveContains(query)
      || tags.contains { $0.localizedCaseInsensitiveContains(query) }
  }
}

/// Returns active entries newest-first, limited to the 20 most recent entries.
func recentVaultEntries(_ entries: [VaultEntry], limit: Int = 20) -> [VaultEntry] {
  guard limit > 0 else { return [] }

  return
    entries
    .filter { !$0.isTrashed }
    .sorted { left, right in
      switch (left.lastModified, right.lastModified) {
      case (let leftDate?, let rightDate?) where leftDate != rightDate:
        return leftDate > rightDate
      case (_?, nil):
        return true
      case (nil, _?):
        return false
      default:
        let titleOrder = left.displayTitle.localizedCaseInsensitiveCompare(right.displayTitle)
        if titleOrder != .orderedSame {
          return titleOrder == .orderedAscending
        }
        return left.id.uuidString < right.id.uuidString
      }
    }
    .prefix(limit)
    .map { $0 }
}
