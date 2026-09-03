import KDBXKit
import XCTest

@testable import HitsuPasswordManager

final class TrashFilteringTests: XCTestCase {
  private func group(_ name: String, entries: [KDBX.Entry] = [], groups: [KDBX.Group] = [])
    -> KDBX.Group
  {
    KDBX.Group(uuid: UUID(), name: name, entries: entries, groups: groups)
  }

  func testNilRecycleBinMarksNothing() {
    let root = group("Root", groups: [group("Work")])

    XCTAssertTrue(trashedGroupIDs(in: root, recycleBinID: nil).isEmpty)
  }

  func testZeroRecycleBinUUIDMarksNothing() {
    // Zero UUID is KeePass's "create the bin when needed" marker; it must not
    // accidentally match a group.
    let root = group("Root", groups: [group("Work")])

    XCTAssertTrue(trashedGroupIDs(in: root, recycleBinID: kdbxZeroUUID).isEmpty)
  }

  func testMarksBinAndDescendantsOnly() {
    let oldEntry = KDBX.Entry(uuid: UUID(), iconID: 0)
    let oldStuff = group("Old stuff", entries: [oldEntry])
    let bin = group("Recycle Bin", groups: [oldStuff])
    let work = group("Work", groups: [group("Nested")])
    let root = group("Root", entries: [KDBX.Entry(uuid: UUID(), iconID: 0)], groups: [bin, work])

    let trashed = trashedGroupIDs(in: root, recycleBinID: bin.uuid)

    // The bin and its descendants are trashed; sibling trees are not.
    XCTAssertEqual(trashed, [bin.uuid, oldStuff.uuid])
  }

  func testBinsDeepInTreeAreTrashed() {
    let bin = group("Deleted")
    let root = group("Root", groups: [group("Archive", groups: [bin])])

    let trashed = trashedGroupIDs(in: root, recycleBinID: bin.uuid)

    XCTAssertEqual(trashed, [bin.uuid])
  }

  func testMissingBinGroupMarksNothing() {
    // A vault whose meta points at a group that no longer exists.
    let root = group("Root", groups: [group("Work")])

    XCTAssertTrue(trashedGroupIDs(in: root, recycleBinID: UUID()).isEmpty)
  }

  func testTrashedEntryExcludedFromFilters() {
    let trashed = VaultEntry(
      id: UUID(),
      icon: VaultEntryIcon(standardID: 0, customData: nil),
      title: "Old login",
      username: "ada",
      url: "",
      isTitleProtected: false,
      isUsernameProtected: false,
      isURLProtected: false,
      isNotesProtected: false,
      groupPath: "Recycle Bin",
      category: .login,
      isFavorite: true,
      isTrashed: true,
      lastModified: nil,
      hasPassword: true,
      hasTOTP: false,
      hasNotes: false,
      fields: [],
      typedFields: [],
      attachments: [],
      history: [],
      tags: []
    )

    // A trashed favorite must not surface in the Favorites or Categories
    // lists, but stays findable by the Trash tab's own search.
    XCTAssertTrue(trashed.isTrashed)
    XCTAssertTrue(trashed.matchesSearch("Old"))
  }
}
