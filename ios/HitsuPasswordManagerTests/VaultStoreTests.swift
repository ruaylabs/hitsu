import KDBXKit
import XCTest

@testable import HitsuPasswordManager

/// Security behaviors around open/lock: the lock must clear all state, and
/// on-demand attachment resolution must never outlive it.
@MainActor
final class VaultStoreTests: XCTestCase {
  private static let masterPassword = "correct horse battery staple"
  private static let attachmentPayload = Data("attachment payload".utf8)

  /// Writes a 4.x vault with one entry and one pool-referenced attachment to a
  /// temporary file. Uses a cheap KDF (8 MiB, 1 iteration) instead of the
  /// 64 MiB default so the tests don't pay real Argon2 cost per open.
  private func makeVaultFile() throws -> URL {
    var content = KDBXContent.makeEmpty(
      databaseName: "Test vault",
      kdf: .argon2id(
        .init(
          version: .v1_3,
          salt: Data(repeating: 7, count: 32),
          iterations: 1,
          memory: 8 * 1024 * 1024,
          parallelism: 1
        ),
        additional: [:]
      )
    )
    content.innerHeader.binaryContent = [
      .init(shouldBeProtected: false, data: Self.attachmentPayload)
    ]
    content.database.root.group.entries = [
      KDBX.Entry(
        uuid: UUID(),
        strings: [
          KDBX.ProtectedString(key: "Title", value: .regular(SecureBytes("Test entry".utf8)))
        ],
        binaries: [.init(key: "f.bin", value: .ref(0))]
      )
    ]

    let url = FileManager.default.temporaryDirectory
      .appendingPathComponent("test-\(UUID().uuidString).kdbx")
    guard let stream = OutputStream(url: url, append: false) else {
      throw NSError(domain: "VaultStoreTests", code: 1)
    }
    stream.open()
    defer { stream.close() }
    try KDBXWriter(to: stream).write(
      content,
      unlockData: UnlockData(masterPassword: Self.masterPassword)
    )
    return url
  }

  /// `open` is fire-and-forget; wait for the detached parse to settle.
  private func waitForOpen(_ store: VaultStore) async throws {
    let deadline = Date().addingTimeInterval(10)
    while store.isLoading {
      if Date() > deadline {
        XCTFail("Timed out waiting for the vault to open")
        return
      }
      try await Task.sleep(for: .milliseconds(20))
    }
  }

  func testOpenUnlocksAndLockClearsState() async throws {
    let url = try makeVaultFile()
    defer { try? FileManager.default.removeItem(at: url) }
    let store = VaultStore()

    store.open(url: url, unlockData: UnlockData(masterPassword: Self.masterPassword))
    try await waitForOpen(store)

    XCTAssertTrue(store.isUnlocked)
    XCTAssertEqual(store.entries.first?.title, "Test entry")

    store.lock()
    XCTAssertFalse(store.isUnlocked)
    XCTAssertTrue(store.entries.isEmpty)
    XCTAssertNil(store.errorMessage)
  }

  func testWrongPasswordFailsWithoutUnlocking() async throws {
    let url = try makeVaultFile()
    defer { try? FileManager.default.removeItem(at: url) }
    let store = VaultStore()

    store.open(url: url, unlockData: UnlockData(masterPassword: "wrong password"))
    try await waitForOpen(store)

    XCTAssertFalse(store.isUnlocked)
    XCTAssertEqual(store.errorMessage, "That password did not unlock this database.")
  }

  func testAttachmentResolvesOnDemandAndDiesOnLock() async throws {
    let url = try makeVaultFile()
    defer { try? FileManager.default.removeItem(at: url) }
    let store = VaultStore()

    store.open(url: url, unlockData: UnlockData(masterPassword: Self.masterPassword))
    try await waitForOpen(store)
    let entry = try XCTUnwrap(store.entries.first)

    let data = await store.attachmentData(for: entry.id, index: 0)
    XCTAssertEqual(data, Self.attachmentPayload)

    store.lock()
    let afterLock = await store.attachmentData(for: entry.id, index: 0)
    XCTAssertNil(afterLock)
  }

  func testResolutionStartedBeforeLockIsDiscarded() async throws {
    let url = try makeVaultFile()
    defer { try? FileManager.default.removeItem(at: url) }
    let store = VaultStore()

    store.open(url: url, unlockData: UnlockData(masterPassword: Self.masterPassword))
    try await waitForOpen(store)
    let entry = try XCTUnwrap(store.entries.first)

    // The task cannot start until the main actor yields, so `lock()` runs
    // first either way: the resolution must come back discarded, never with
    // payload from a locked vault.
    let task = Task { await store.attachmentData(for: entry.id, index: 0) }
    store.lock()
    let data = await task.value

    XCTAssertNil(data)
  }
}
