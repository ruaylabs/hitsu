import XCTest

@testable import HitsuPasswordManager

@MainActor
final class AttachmentStagingTests: XCTestCase {
  private var staging: AttachmentPreviewStaging { .shared }

  func testStagedFileCarriesCompleteProtection() throws {
    let url = try staging.stagePreview(data: Data("payload".utf8), fileName: "photo.png")
    defer { staging.removePreview(in: url.deletingLastPathComponent()) }

    let attributes = try FileManager.default.attributesOfItem(atPath: url.path)
    guard let protection = attributes[.protectionKey] as? String else {
      // Some platforms (e.g. the simulator) do not record protection
      // attributes; the device assertion remains the real gate.
      throw XCTSkip("File protection attributes are not recorded on this platform.")
    }
    XCTAssertEqual(protection, FileProtectionType.complete.rawValue)
  }

  func testStagesUnderDedicatedRootWithSanitizedName() throws {
    let url = try staging.stagePreview(data: Data("x".utf8), fileName: "a/b.txt")
    defer { staging.removePreview(in: url.deletingLastPathComponent()) }

    XCTAssertEqual(url.lastPathComponent, "a_b.txt")
    XCTAssertEqual(
      url.deletingLastPathComponent().deletingLastPathComponent().lastPathComponent,
      "AttachmentPreviews"
    )
  }

  func testEmptyFileNameFallsBackToGenericName() throws {
    let url = try staging.stagePreview(data: Data("x".utf8), fileName: "")
    defer { staging.removePreview(in: url.deletingLastPathComponent()) }

    XCTAssertEqual(url.lastPathComponent, "attachment")
  }

  func testRemovePreviewDeletesItsDirectory() throws {
    let url = try staging.stagePreview(data: Data("x".utf8), fileName: "doc.pdf")
    let directory = url.deletingLastPathComponent()

    staging.removePreview(in: directory)

    XCTAssertFalse(FileManager.default.fileExists(atPath: url.path))
    XCTAssertFalse(FileManager.default.fileExists(atPath: directory.path))
  }

  func testPurgeKeepsActivePreviewsAndRemovesResidue() throws {
    let kept = try staging.stagePreview(data: Data("x".utf8), fileName: "keep.png")
    defer { staging.removePreview(in: kept.deletingLastPathComponent()) }

    // A staged-looking directory nobody registered stands in for one a killed
    // run left behind.
    let root = FileManager.default.temporaryDirectory
      .appendingPathComponent("AttachmentPreviews", isDirectory: true)
    let orphan = root.appendingPathComponent(UUID().uuidString, isDirectory: true)
    try FileManager.default.createDirectory(at: orphan, withIntermediateDirectories: true)

    staging.purgeStale()

    XCTAssertTrue(FileManager.default.fileExists(atPath: kept.path))
    XCTAssertFalse(FileManager.default.fileExists(atPath: orphan.path))
  }

  func testFreshProcessStatePurgesEverythingLeftBehind() throws {
    let url = try staging.stagePreview(data: Data("x".utf8), fileName: "secret.pdf")
    XCTAssertTrue(FileManager.default.fileExists(atPath: url.path))

    // A relaunched process has an empty active set; its purge removes all
    // residue from the previous run.
    let relaunched = AttachmentPreviewStaging()
    relaunched.purgeStale()

    XCTAssertFalse(FileManager.default.fileExists(atPath: url.path))
    staging.removePreview(in: url.deletingLastPathComponent())
  }
}
