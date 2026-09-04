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

/// Belt-and-braces on top of KDBXKit's own decompressed-payload cap; the 3.x
/// eager fallback reads the whole file.
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

  /// 4.x streaming open: everything but attachment bytes, so payloads resolve
  /// on demand. nil in eager (3.x) mode.
  private var lazyContent: LazyKDBXContent?

  /// How each attachment resolves its bytes; index-aligned with
  /// `VaultAttachment.index`.
  private var attachmentSourcesByID: [UUID: [AttachmentSource]] = [:]

  /// Security scope held for the session so previews can re-read the file.
  private var securityScopedURL: URL?

  /// Prior entry versions (oldest first), retained so history fields can be
  /// revealed on demand without keeping the whole parsed database alive.
  private var historyByID: [UUID: [KDBX.Entry]] = [:]

  func clearError() {
    errorMessage = nil
  }

  func showError(_ message: String) {
    errorMessage = message
  }

  /// Accepts `UnlockData` rather than the cleartext password so the master
  /// password never crosses actor boundaries as a `String`. KDBXKit discards
  /// the cleartext at `UnlockData` init and keeps only the mlock'd pre-hash.
  func open(
    url: URL,
    unlockData: UnlockData,
    onComplete: ((Bool) -> Void)? = nil
  ) {
    guard !isLoading else { return }

    let generation = lockGeneration
    isLoading = true
    errorMessage = nil
    // Access is held until lock(): previews re-read the file.
    let hasSecurityScope = url.startAccessingSecurityScopedResource()

    Task { [weak self] in
      let result = await Task.detached(priority: .userInitiated) {
        do {
          let fileSize = (try? url.resourceValues(forKeys: [.fileSizeKey]))?.fileSize
          if let fileSize, fileSize > maxVaultFileSize {
            return Result<VaultOpenResult, VaultOpenFailure>.failure(
              .message("The database is too large to open on this device.")
            )
          }
          do {
            // 4.x: streaming open keeps only the XML resident.
            return Result<VaultOpenResult, VaultOpenFailure>.success(
              VaultOpenResult.lazy(
                try KDBXReader.openMetadataStreaming(
                  from: .file(url),
                  unlockData: unlockData,
                  kdfLimits: vaultKDFLimits
                )
              )
            )
          } catch KDBXReader.Error.unsupportedFormatVersion(let major, _) where major == 3 {
            // 3.x binaries live inline in the XML body — no pool to stream past.
            let data = try Data(contentsOf: url, options: [.mappedIfSafe])
            return Result<VaultOpenResult, VaultOpenFailure>.success(
              VaultOpenResult.eager(
                try KDBXReader.parse(data, unlockData: unlockData, kdfLimits: vaultKDFLimits)
              )
            )
          }
        } catch let error as KDBXReader.Error {
          return Result<VaultOpenResult, VaultOpenFailure>.failure(
            .message(userMessage(for: error))
          )
        } catch {
          return Result<VaultOpenResult, VaultOpenFailure>.failure(
            .message("The selected file could not be read.")
          )
        }
      }.value

      guard let self else {
        if hasSecurityScope {
          url.stopAccessingSecurityScopedResource()
        }
        return
      }
      isLoading = false
      guard lockGeneration == generation else {
        if hasSecurityScope {
          url.stopAccessingSecurityScopedResource()
        }
        return
      }

      switch result {
      case .success(let opened):
        let projection: VaultProjection
        switch opened {
        case .lazy(let lazy):
          projection = makeVaultProjection(
            from: lazy.database,
            binaries: .poolMetadata(lazy.binaries)
          )
          lazyContent = lazy
        case .eager(let content):
          projection = makeVaultProjection(
            from: content.database,
            binaries: .residentPool(content.innerHeader.binaryContent)
          )
          lazyContent = nil
        }
        entries = projection.entries
        entryStringsByID = projection.entryStringsByID
        attachmentSourcesByID = projection.attachmentSourcesByID
        historyByID = projection.historyByID
        stopSecurityScope()
        securityScopedURL = hasSecurityScope ? url : nil
        isUnlocked = true
        errorMessage = nil
        onComplete?(true)
      case .failure(.message(let message)):
        stopSecurityScope()
        entries = []
        entryStringsByID = [:]
        attachmentSourcesByID = [:]
        historyByID = [:]
        isUnlocked = false
        errorMessage = message
        onComplete?(false)
      }
    }
  }

  /// Releases the session's security-scope access.
  private func stopSecurityScope() {
    guard let url = securityScopedURL else { return }
    securityScopedURL = nil
    url.stopAccessingSecurityScopedResource()
  }

  func lock() {
    lockGeneration += 1
    entries = []
    entryStringsByID = [:]
    attachmentSourcesByID = [:]
    historyByID = [:]
    lazyContent = nil
    stopSecurityScope()
    isUnlocked = false
    errorMessage = nil
  }

  /// Returns one attachment payload, re-streamed from the file on demand.
  /// nil when unknown, the vault was locked mid-read, or the re-stream
  /// fails (an error message is set).
  func attachmentData(for entryID: UUID, index: Int) async -> Data? {
    guard let sources = attachmentSourcesByID[entryID],
      sources.indices.contains(index)
    else { return nil }

    switch sources[index] {
    case .inline(let data):
      return data
    case .pool(let poolIndex):
      guard let lazy = lazyContent else { return nil }
      let generation = lockGeneration
      let capacityHint =
        lazy.binaries.indices.contains(Int(poolIndex))
        ? lazy.binaries[Int(poolIndex)].sizeBytes : 0
      let result = await Task.detached(priority: .userInitiated) {
        do {
          var sink = DataSink(capacityHint: capacityHint)
          try KDBXReader.streamBinary(from: lazy, at: Int(poolIndex), into: &sink)
          return Result<Data, KDBXReader.Error>.success(sink.data)
        } catch let error as KDBXReader.Error {
          return Result<Data, KDBXReader.Error>.failure(error)
        } catch {
          return Result<Data, KDBXReader.Error>.failure(
            .corruptedInnerHeader(reason: "Attachment re-stream failed")
          )
        }
      }.value
      guard lockGeneration == generation else { return nil }
      switch result {
      case .success(let data):
        return data
      case .failure(let error):
        errorMessage = userMessage(for: error)
        return nil
      }
    }
  }

  /// Reveals one field of a stored history version.
  func historyValue(for entryID: UUID, index: Int, field name: String) -> String? {
    guard let versions = historyByID[entryID], versions.indices.contains(index) else {
      return nil
    }
    return versions[index].strings.first(where: { $0.key == name })?.value.revealedString
  }

  /// Protection state of a history field; nil when the field is absent from
  /// that version.
  func historyFieldIsProtected(for entryID: UUID, index: Int, field name: String) -> Bool? {
    guard let versions = historyByID[entryID], versions.indices.contains(index),
      let field = versions[index].strings.first(where: { $0.key == name })
    else { return nil }
    switch field.value {
    case .lazyInnerCipher(_, _, _), .protectedInMemory(_):
      return true
    case .regular(_), .unprotected(_):
      return false
    }
  }

  /// Custom field names carried by a history version (standard names excluded).
  func historyFieldNames(for entryID: UUID, index: Int) -> [String] {
    guard let versions = historyByID[entryID], versions.indices.contains(index) else {
      return []
    }
    let hidden = Set([
      "title", "username", "url", "password", "notes", "otp", "totp seed", "totp settings",
    ])
    return versions[index].strings
      .map(\.key)
      .filter { !hidden.contains($0.lowercased()) }
  }

  /// Returns one field only when the detail view asks for it. The store never
  /// keeps a second copy of the revealed value.
  func value(for entryID: UUID, field name: String) -> String? {
    entryStringsByID[entryID]?
      .first(where: { $0.key == name })?
      .value.revealedString
  }

  /// Searches the entry's non-protected field values without retaining a
  /// second plaintext copy in the entry list model.
  func matchesSearch(_ entry: VaultEntry, searchText: String) -> Bool {
    entryMatchesSearch(
      entry,
      searchText: searchText,
      fields: entryStringsByID[entry.id] ?? []
    )
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

/// Matches the summary fields plus custom field names and unprotected field
/// values. Protected values are never materialized just to satisfy a search.
func entryMatchesSearch(
  _ entry: VaultEntry,
  searchText: String,
  fields: [KDBX.ProtectedString]
) -> Bool {
  let query = searchText.trimmingCharacters(in: .whitespacesAndNewlines)
  guard !query.isEmpty else { return true }
  if entry.matchesSearch(query) { return true }

  let standardNames = Set([
    "title", "username", "url", "password", "notes", "otp", "totp seed", "totp settings",
  ])
  return fields.contains { field in
    let fieldName = field.key.lowercased()
    if !standardNames.contains(fieldName)
      && field.key.localizedCaseInsensitiveContains(query)
    {
      return true
    }

    switch field.value {
    case .regular(_), .unprotected(_):
      return field.value.withRevealedString {
        $0.localizedCaseInsensitiveContains(query)
      }
    case .lazyInnerCipher(_, _, _), .protectedInMemory(_):
      return false
    }
  }
}

/// Streaming (4.x) or eager (3.x) open result.
private enum VaultOpenResult: Sendable {
  case lazy(LazyKDBXContent)
  case eager(KDBXContent)
}

/// How the store resolves one attachment's bytes on demand.
enum AttachmentSource: Sendable {
  /// 4.x pool reference; bytes are re-streamed on demand.
  case pool(UInt32)
  /// Bytes already resident (3.x or a rare inline 4.x binary).
  case inline(Data)
}

/// Where `makeVaultProjection` resolves `.ref` attachment bytes from.
private enum VaultBinarySource {
  /// 4.x streaming open: per-binary metadata only.
  case poolMetadata([BinaryMetadata])
  /// 3.x eager open: the full pool is resident.
  case residentPool([InnerHeader.BinaryContent])
}

private struct VaultProjection {
  let entries: [VaultEntry]
  let entryStringsByID: [UUID: [KDBX.ProtectedString]]
  let attachmentSourcesByID: [UUID: [AttachmentSource]]
  let historyByID: [UUID: [KDBX.Entry]]
}

private func makeVaultProjection(
  from database: KDBX,
  binaries: VaultBinarySource
) -> VaultProjection {
  let trashedIDs = trashedGroupIDs(
    in: database.root.group,
    recycleBinID: database.meta.recycleBinUUID
  )
  var result: [VaultEntry] = []
  var entryStringsByID: [UUID: [KDBX.ProtectedString]] = [:]
  var attachmentSourcesByID: [UUID: [AttachmentSource]] = [:]
  var historyByID: [UUID: [KDBX.Entry]] = [:]
  var customIconsByID: [UUID: Data] = [:]
  for customIcon in database.meta.customIcons {
    customIconsByID[customIcon.uuid] = customIcon.data
  }

  func appendEntries(in group: KDBX.Group, path: String) {
    let isTrashed = trashedIDs.contains(group.uuid)
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
      let category = VaultEntryCategory(databaseValue: categoryValue)
      let typedFields = makeTypedFields(for: category, entry: entry, protectedNames: protectedNames)
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
      let typedKeyPrefixes = ["card.", "identity.", "license.", "passport.", "pgp."]
      let fieldNames = entry.strings
        .map(\.key)
        .filter { !hiddenFieldNames.contains($0.lowercased()) }
        .filter { key in !typedKeyPrefixes.contains { key.lowercased().hasPrefix($0) } }
        .map { VaultField(name: $0, isProtected: protectedNames.contains($0)) }

      var attachments: [VaultAttachment] = []
      var sources: [AttachmentSource] = []
      for binary in entry.binaries {
        switch binary.value {
        case .inline(let inlineData, _):
          attachments.append(
            VaultAttachment(
              index: sources.count,
              name: binary.key,
              byteCount: inlineData.count
            )
          )
          sources.append(.inline(inlineData))
        case .ref(let poolIndex):
          switch binaries {
          case .poolMetadata(let metadata):
            guard Int(poolIndex) < metadata.count else { continue }
            attachments.append(
              VaultAttachment(
                index: sources.count,
                name: binary.key,
                byteCount: metadata[Int(poolIndex)].sizeBytes
              )
            )
            sources.append(.pool(poolIndex))
          case .residentPool(let pool):
            guard Int(poolIndex) < pool.count else { continue }
            let data = pool[Int(poolIndex)].data
            attachments.append(
              VaultAttachment(
                index: sources.count,
                name: binary.key,
                byteCount: data.count
              )
            )
            sources.append(.inline(data))
          }
        }
      }
      if !sources.isEmpty {
        attachmentSourcesByID[entry.uuid] = sources
      }

      let historyItems = entry.history.enumerated().map { index, version in
        VaultHistoryItem(index: index, lastModified: version.times?.lastModificationTime)
      }
      if !entry.history.isEmpty {
        historyByID[entry.uuid] = entry.history
      }

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
          category: category,
          isFavorite: favoriteValue == "true",
          isTrashed: isTrashed,
          lastModified: entry.times?.lastModificationTime,
          expirationDate: entry.times?.expires == true ? entry.times?.expiryTime : nil,
          hasPassword: hasPassword,
          hasTOTP: hasTOTP,
          hasNotes: hasNotes,
          fields: fieldNames,
          typedFields: typedFields,
          attachments: attachments,
          history: historyItems,
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
  return VaultProjection(
    entries: entries,
    entryStringsByID: entryStringsByID,
    attachmentSourcesByID: attachmentSourcesByID,
    historyByID: historyByID
  )
}

/// The all-zero UUID that KDBX uses as the "recycle bin not created yet"
/// marker (Meta.recycleBinUUID is zero until a bin group exists). Internal so
/// the test target can assert on it.
let kdbxZeroUUID = UUID(uuid: (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0))

/// UUIDs of the recycle-bin group and every group nested inside it. An entry
/// counts as trashed when its containing group appears here, mirroring the
/// desktop's `entry_is_trashed` ancestry walk. A nil or zero UUID (KeePass's
/// "create the bin when needed" marker) matches nothing.
func trashedGroupIDs(in root: KDBX.Group, recycleBinID: UUID?) -> Set<UUID> {
  guard let recycleBinID, recycleBinID != kdbxZeroUUID else { return [] }
  var ids: Set<UUID> = []
  func walk(_ group: KDBX.Group, isTrashed: Bool) {
    let trashed = isTrashed || group.uuid == recycleBinID
    if trashed {
      ids.insert(group.uuid)
    }
    for child in group.groups {
      walk(child, isTrashed: trashed)
    }
  }
  walk(root, isTrashed: false)
  return ids
}

private func unprotectedValue(
  in entry: KDBX.Entry,
  named name: String,
  protectedNames: Set<String>
) -> String? {
  guard !protectedNames.contains(name) else { return nil }
  return entry.strings.first(where: { $0.key == name })?.value.revealedString
}

/// Category-specific detail rows mirroring the desktop app's typed layouts.
/// Secret rows are always reveal-gated; unprotected rows resolve their value
/// on demand from `entryStringsByID`.
func makeTypedFields(
  for category: VaultEntryCategory,
  entry: KDBX.Entry,
  protectedNames: Set<String>
) -> [VaultTypedField] {
  func field(
    _ label: String,
    _ key: String,
    secret: Bool = false,
    isCardNumber: Bool = false
  ) -> VaultTypedField? {
    guard let stored = entry.strings.first(where: { $0.key == key }) else { return nil }
    // Trust the value itself in addition to the caller-supplied protection
    // set, so a protected field is never materialized as a plain row.
    let valueProtected: Bool
    switch stored.value {
    case .lazyInnerCipher(_, _, _), .protectedInMemory(_):
      valueProtected = true
    case .regular(_), .unprotected(_):
      valueProtected = false
    }
    return VaultTypedField(
      label: label,
      field: key,
      isProtected: secret || valueProtected || protectedNames.contains(key),
      displayValue: nil,
      isCardNumber: isCardNumber
    )
  }
  func display(_ label: String, _ value: String) -> VaultTypedField {
    VaultTypedField(
      label: label,
      field: nil,
      isProtected: false,
      displayValue: value,
      isCardNumber: false
    )
  }

  switch category {
  case .card:
    var fields = [
      field("Type", "card.type"),
      field("Holder", "card.holder"),
      field("Number", "card.number", secret: true, isCardNumber: true),
    ].compactMap { $0 }
    let month =
      unprotectedValue(in: entry, named: "card.expMonth", protectedNames: protectedNames) ?? ""
    let year =
      unprotectedValue(in: entry, named: "card.expYear", protectedNames: protectedNames) ?? ""
    if !month.isEmpty || !year.isEmpty {
      let monthPart = month.count == 1 ? "0" + month : month
      let value = [monthPart, year].filter { !$0.isEmpty }.joined(separator: "/")
      fields.append(display("Expires", value))
    }
    fields += [
      field("CVV", "card.cvv", secret: true),
      field("PIN", "card.pin", secret: true),
    ].compactMap { $0 }
    return fields
  case .identity:
    return [
      field("First name", "identity.firstName"),
      field("Last name", "identity.lastName"),
      field("Email", "identity.email"),
      field("Phone", "identity.phone"),
      field("Address", "identity.address"),
      field("Date of birth", "identity.dob"),
    ].compactMap { $0 }
  case .softwareLicense:
    return [
      field("Version", "license.version"),
      field("License key", "license.key", secret: true),
      field("Licensed to", "license.licensedTo"),
      field("Registered email", "license.registeredEmail"),
      field("Company", "license.company"),
      field("Download page", "license.downloadPage"),
      field("Publisher", "license.publisher"),
      field("Website", "license.website"),
      field("Retail price", "license.retailPrice"),
      field("Support email", "license.supportEmail"),
      field("Purchase date", "license.purchaseDate"),
      field("Order number", "license.orderNumber"),
      field("Order total", "license.orderTotal"),
    ].compactMap { $0 }
  case .passport:
    return [
      field("Type", "passport.type"),
      field("Issuing country", "passport.issuingCountry"),
      field("Number", "passport.number", secret: true),
      field("Full name", "passport.fullName"),
      field("Sex", "passport.sex"),
      field("Nationality", "passport.nationality"),
      field("Issuing authority", "passport.issuingAuthority"),
      field("Date of birth", "passport.birthDate"),
      field("Place of birth", "passport.birthPlace"),
      field("Issued on", "passport.issueDate"),
      field("Expiry date", "passport.expiryDate"),
    ].compactMap { $0 }
  case .pgpKey:
    return [
      field("Fingerprint", "pgp.fingerprint"),
      field("Key ID", "pgp.keyId"),
      field("User IDs", "pgp.userIds"),
      field("Algorithm", "pgp.algorithm"),
      field("Expires", "pgp.expiresAt"),
      field("Public key", "pgp.publicKey"),
      field("Private key", "pgp.privateKey", secret: true),
    ].compactMap { $0 }
  case .login, .password, .note:
    return []
  }
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
