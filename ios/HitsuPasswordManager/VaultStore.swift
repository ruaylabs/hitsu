import Foundation
import KDBXKit
import Observation

private enum VaultOpenFailure: Error, Sendable {
  case message(String)
}

/// iOS unlock policy: bounds the KDF work a crafted or misconfigured vault can
/// force onto the device. Much tighter than KDBXKit's `.default` (1 GiB memory,
/// 1000 iterations, 1024 lanes, 100M rounds), which would let a hostile file
/// wedge the app or trip jetsam mid-unlock. Values sit well above what KeePass
/// and Hitsu write by default (64 MiB, a handful of iterations, 6M AES rounds)
/// so legitimate vaults open unchanged; out-of-policy files fail with the
/// existing "too expensive for this device" message.
private let vaultKDFLimits = KDFParameterLimits(
  maxArgon2Memory: 256 * 1024 * 1024,
  maxArgon2Iterations: 256,
  maxArgon2Parallelism: 8,
  maxAESKDFRounds: 10_000_000
)

/// Rejects oversized files before reading them: the eager parse path copies the
/// whole payload into memory, so the encrypted file size drives peak footprint.
private let maxVaultFileSize = 256 * 1024 * 1024

@MainActor
@Observable
final class VaultStore {
  private(set) var entries: [VaultEntry] = []
  private(set) var isLoading = false
  private(set) var isUnlocked = false
  var errorMessage: String?

  /// Bumped on every lock; in-flight opens discard their results if a lock
  /// (e.g. the auto-lock on backgrounding) happened while they were parsing.
  private var lockGeneration = 0

  private var entryStringsByID: [UUID: [KDBX.ProtectedString]] = [:]

  func clearError() {
    errorMessage = nil
  }

  func showError(_ message: String) {
    errorMessage = message
  }

  func open(url: URL, password: String) {
    guard !password.isEmpty, !isLoading else { return }

    let generation = lockGeneration
    isLoading = true
    errorMessage = nil
    let hasSecurityScope = url.startAccessingSecurityScopedResource()

    Task { [weak self] in
      let result = await Task.detached(priority: .userInitiated) {
        do {
          let fileSize = (try? url.resourceValues(forKeys: [.fileSizeKey]))?.fileSize
          if let fileSize, fileSize > maxVaultFileSize {
            return Result<KDBXContent, VaultOpenFailure>.failure(
              .message("The database is too large to open on this device.")
            )
          }
          let data = try Data(contentsOf: url, options: [.mappedIfSafe])
          let unlockData = UnlockData(masterPassword: password)
          return Result<KDBXContent, VaultOpenFailure>.success(
            try KDBXReader.parse(data, unlockData: unlockData, kdfLimits: vaultKDFLimits)
          )
        } catch let error as KDBXReader.Error {
          return Result<KDBXContent, VaultOpenFailure>.failure(
            .message(userMessage(for: error))
          )
        } catch {
          return Result<KDBXContent, VaultOpenFailure>.failure(
            .message("The selected file could not be read.")
          )
        }
      }.value

      if hasSecurityScope {
        url.stopAccessingSecurityScopedResource()
      }

      guard let self else { return }
      isLoading = false
      guard lockGeneration == generation else { return }

      switch result {
      case .success(let content):
        let projection = makeVaultProjection(from: content.database)
        entries = projection.entries
        entryStringsByID = projection.entryStringsByID
        isUnlocked = true
        errorMessage = nil
      case .failure(.message(let message)):
        entries = []
        entryStringsByID = [:]
        isUnlocked = false
        errorMessage = message
      }
    }
  }

  func lock() {
    lockGeneration += 1
    entries = []
    entryStringsByID = [:]
    isUnlocked = false
    errorMessage = nil
  }

  /// Returns one field only when the detail view asks for it. The store never
  /// keeps a second copy of the revealed value.
  func value(for entryID: UUID, field name: String) -> String? {
    entryStringsByID[entryID]?
      .first(where: { $0.key == name })?
      .value.revealedString
  }

  func totpCode(for entryID: UUID, at date: Date = Date()) -> TOTPCode? {
    guard let fields = entryStringsByID[entryID] else { return nil }

    if let otp = fields.first(where: { $0.key.caseInsensitiveCompare("otp") == .orderedSame }) {
      return otp.value.withRevealedString { TOTPGenerator.code(from: $0, at: date) }
    }

    guard
      let seed = fields.first(where: {
        $0.key.caseInsensitiveCompare("TOTP Seed") == .orderedSame
      })
    else { return nil }

    if let settings = fields.first(where: {
      $0.key.caseInsensitiveCompare("TOTP Settings") == .orderedSame
    }) {
      return settings.value.withRevealedString { settingsValue in
        seed.value.withRevealedString {
          TOTPGenerator.code(fromLegacySecret: $0, settings: settingsValue, at: date)
        }
      }
    }

    return seed.value.withRevealedString {
      TOTPGenerator.code(fromLegacySecret: $0, settings: nil, at: date)
    }
  }
}

private struct VaultProjection {
  let entries: [VaultEntry]
  let entryStringsByID: [UUID: [KDBX.ProtectedString]]
}

private func makeVaultProjection(from database: KDBX) -> VaultProjection {
  var result: [VaultEntry] = []
  var entryStringsByID: [UUID: [KDBX.ProtectedString]] = [:]
  var customIconsByID: [UUID: Data] = [:]
  for customIcon in database.meta.customIcons {
    customIconsByID[customIcon.uuid] = customIcon.data
  }

  func appendEntries(in group: KDBX.Group, path: String) {
    for entry in group.entries {
      entryStringsByID[entry.uuid] = entry.strings
      let protectedNames = Set(
        entry.strings.compactMap { field in
          switch field.value {
          case .lazyInnerCipher(_, _, _), .protectedInMemory(_):
            return field.key
          case .regular(_), .unprotected(_):
            return nil
          }
        }
      )
      let title = unprotectedValue(in: entry, named: "Title", protectedNames: protectedNames) ?? ""
      let username =
        unprotectedValue(in: entry, named: "UserName", protectedNames: protectedNames) ?? ""
      let url = unprotectedValue(in: entry, named: "URL", protectedNames: protectedNames) ?? ""
      let categoryValue = customDataValue(
        in: entry,
        currentKey: "hitsu.itemType",
        legacyKey: "kagi.itemType"
      )
      let favoriteValue = customDataValue(
        in: entry,
        currentKey: "hitsu.favorite",
        legacyKey: "kagi.favorite"
      )
      let hasPassword =
        entry.strings.first(where: { $0.key == "Password" })?.value
        .withRevealedString { !$0.isEmpty } ?? false
      let hasTOTP = entry.strings.contains {
        ["otp", "totp seed"].contains($0.key.lowercased())
      }
      let hasNotes =
        entry.strings.first(where: { $0.key == "Notes" })?.value
        .withRevealedString { !$0.isEmpty } ?? false
      let hiddenFieldNames = Set([
        "title", "username", "url", "password", "notes", "otp", "totp seed", "totp settings",
      ])
      let fieldNames = entry.strings
        .map(\.key)
        .filter { !hiddenFieldNames.contains($0.lowercased()) }
        .map { VaultField(name: $0, isProtected: protectedNames.contains($0)) }

      result.append(
        VaultEntry(
          id: entry.uuid,
          icon: VaultEntryIcon(
            standardID: entry.iconID,
            customData: entry.customIconUUID.flatMap { customIconsByID[$0] }
          ),
          title: title,
          username: username,
          url: url,
          isTitleProtected: protectedNames.contains("Title"),
          isUsernameProtected: protectedNames.contains("UserName"),
          isURLProtected: protectedNames.contains("URL"),
          isNotesProtected: protectedNames.contains("Notes"),
          groupPath: path,
          category: VaultEntryCategory(databaseValue: categoryValue),
          isFavorite: favoriteValue == "true",
          hasPassword: hasPassword,
          hasTOTP: hasTOTP,
          hasNotes: hasNotes,
          fields: fieldNames,
          tags: entry.tags
        )
      )
    }

    for child in group.groups {
      let childName = child.name?.trimmingCharacters(in: .whitespacesAndNewlines)
      let nextPath = [path, childName].compactMap { value in
        guard let value, !value.isEmpty else { return nil }
        return value
      }.joined(separator: " / ")
      appendEntries(in: child, path: nextPath)
    }
  }

  appendEntries(in: database.root.group, path: "")
  let entries = result.sorted {
    $0.displayTitle.localizedCaseInsensitiveCompare($1.displayTitle) == .orderedAscending
  }
  return VaultProjection(entries: entries, entryStringsByID: entryStringsByID)
}

private func unprotectedValue(
  in entry: KDBX.Entry,
  named name: String,
  protectedNames: Set<String>
) -> String? {
  guard !protectedNames.contains(name) else { return nil }
  return entry.strings.first(where: { $0.key == name })?.value.revealedString
}

private func customDataValue(
  in entry: KDBX.Entry,
  currentKey: String,
  legacyKey: String
) -> String? {
  let currentValue = entry.customData.first(where: { $0.key == currentKey })?.value
  let legacyValue = entry.customData.first(where: { $0.key == legacyKey })?.value
  return resolvedHitsuMetadataValue(currentValue: currentValue, legacyValue: legacyValue)
}

func resolvedHitsuMetadataValue(currentValue: String?, legacyValue: String?) -> String? {
  guard let storedValue = currentValue ?? legacyValue else { return nil }
  guard let data = Data(base64Encoded: storedValue),
    let decodedValue = String(data: data, encoding: .utf8)
  else {
    return storedValue
  }
  return decodedValue
}

private func userMessage(for error: KDBXReader.Error) -> String {
  if case .wrongCredentials = error {
    return "That password did not unlock this database."
  }
  if case .invalidFileSignature = error {
    return "This is not a KeePass .kdbx database."
  }
  if case .unsupportedFormatVersion = error {
    return "This KeePass database format is not supported."
  }
  if case .unsupportedEncryption(let cipher) = error {
    return "This database uses an unsupported encryption method (\(cipher.uuidString))."
  }
  if case .unsupportedCompression(let compression) = error {
    return "This database uses an unsupported compression method (\(compression))."
  }
  if case .unsupportedKDF(let kdf) = error {
    return "This database uses an unsupported key derivation method (\(kdf.uuidString))."
  }
  if case .kdfParametersOutOfRange = error {
    return "This database uses settings that are too expensive for this device."
  }
  if case .corruptedHeaderDigest = error {
    return "The database header is corrupted."
  }
  if case .corruptedHeader(let reason) = error {
    return "The database header is invalid: \(reason)"
  }
  if case .corruptedHMAC(let reason) = error {
    return "The database integrity check failed: \(reason)"
  }
  if case .corruptedInnerHeader(let reason) = error {
    return "The encrypted database header is invalid: \(reason)"
  }
  if case .corruptedXML(let reason) = error {
    return "The database contents are invalid: \(reason)"
  }
  if case .decompressedPayloadTooLarge(let limit) = error {
    return "The database is too large to open (limit: \(limit) bytes)."
  }
  if case .unexpectedEOF = error {
    return "The database file is incomplete."
  }
  return "The database is damaged or could not be opened."
}
