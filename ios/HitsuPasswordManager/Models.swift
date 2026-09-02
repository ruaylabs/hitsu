import Foundation

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
}

struct VaultField: Identifiable, Hashable, Sendable {
  let name: String
  let isProtected: Bool

  var id: String { name }
}

struct VaultEntry: Identifiable, Hashable, Sendable {
  let id: UUID
  let title: String
  let username: String
  let url: String
  let groupPath: String
  let category: VaultEntryCategory
  let isFavorite: Bool
  let hasPassword: Bool
  let hasNotes: Bool
  let fields: [VaultField]
  let tags: [String]

  var displayTitle: String {
    title.isEmpty ? "Untitled entry" : title
  }

  var secondaryText: String {
    if !username.isEmpty { return username }
    if !groupPath.isEmpty { return groupPath }
    return "No username"
  }

  func matchesSearch(_ searchText: String) -> Bool {
    let query = searchText.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !query.isEmpty else { return true }
    return displayTitle.localizedCaseInsensitiveContains(query)
      || username.localizedCaseInsensitiveContains(query)
      || groupPath.localizedCaseInsensitiveContains(query)
      || category.title.localizedCaseInsensitiveContains(query)
      || tags.contains { $0.localizedCaseInsensitiveContains(query) }
  }
}
