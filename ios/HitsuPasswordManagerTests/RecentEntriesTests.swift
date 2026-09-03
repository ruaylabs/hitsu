import KDBXKit
import XCTest

@testable import HitsuPasswordManager

final class RecentEntriesTests: XCTestCase {
  func testReturnsTwentyNewestActiveEntries() {
    let entries =
      (0..<25).map { index in
        makeEntry(
          title: "Entry \(index)",
          lastModified: Date(timeIntervalSince1970: Double(index))
        )
      } + [
        makeEntry(
          title: "Trashed newest entry",
          lastModified: Date(timeIntervalSince1970: 100),
          isTrashed: true
        )
      ]

    let recent = recentVaultEntries(entries)

    XCTAssertEqual(recent.count, 20)
    XCTAssertEqual(recent.map(\.title), (5..<25).reversed().map { "Entry \($0)" })
    XCTAssertFalse(recent.contains { $0.isTrashed })
  }

  func testEntriesWithoutModificationDatesSortLast() {
    let entries = [
      makeEntry(title: "Undated"),
      makeEntry(title: "Old", lastModified: Date(timeIntervalSince1970: 1)),
      makeEntry(title: "New", lastModified: Date(timeIntervalSince1970: 2)),
    ]

    XCTAssertEqual(recentVaultEntries(entries).map(\.title), ["New", "Old", "Undated"])
  }

  func testSearchIncludesNotesAndTypedAndCustomFields() {
    let entry = makeEntry(title: "Account")
    let fields: [KDBX.ProtectedString] = [
      .init(key: "Notes", value: .unprotected("Buried recovery instructions")),
      .init(key: "identity.email", value: .unprotected("ada@example.com")),
      .init(key: "custom.Recovery contact", value: .unprotected("Grace")),
    ]

    XCTAssertTrue(entryMatchesSearch(entry, searchText: "recovery instructions", fields: fields))
    XCTAssertTrue(entryMatchesSearch(entry, searchText: "ada@example.com", fields: fields))
    XCTAssertTrue(entryMatchesSearch(entry, searchText: "recovery contact", fields: fields))
  }

  func testSearchExcludesProtectedFieldValues() {
    let entry = makeEntry(title: "Account")
    let fields: [KDBX.ProtectedString] = [
      .init(key: "Notes", value: .protectedInMemory("private note")),
      .init(key: "card.number", value: .protectedInMemory("4111111111111111")),
    ]

    XCTAssertFalse(entryMatchesSearch(entry, searchText: "private note", fields: fields))
    XCTAssertFalse(entryMatchesSearch(entry, searchText: "4111111111111111", fields: fields))
  }

  private func makeEntry(
    title: String,
    lastModified: Date? = nil,
    isTrashed: Bool = false
  ) -> VaultEntry {
    VaultEntry(
      id: UUID(),
      icon: VaultEntryIcon(standardID: 0, customData: nil),
      title: title,
      username: "",
      url: "",
      isTitleProtected: false,
      isUsernameProtected: false,
      isURLProtected: false,
      isNotesProtected: false,
      groupPath: "",
      category: .login,
      isFavorite: false,
      isTrashed: isTrashed,
      lastModified: lastModified,
      hasPassword: false,
      hasTOTP: false,
      hasNotes: false,
      fields: [],
      typedFields: [],
      attachments: [],
      history: [],
      tags: []
    )
  }
}
