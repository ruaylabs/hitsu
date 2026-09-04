import XCTest

@testable import HitsuPasswordManager

/// Clipboard ownership: lock-time clearing must only ever remove items the
/// app itself put on the pasteboard.
@MainActor
final class ClipboardManagerTests: XCTestCase {
  func testClearIfOwnedClearsOwnItem() {
    let pasteboard = UIPasteboard.general
    let clipboard = ClipboardManager()

    clipboard.copy("secret", expirationDate: Date().addingTimeInterval(30))
    XCTAssertEqual(pasteboard.string, "secret")

    clipboard.clearIfOwned()
    XCTAssertNil(pasteboard.string)
  }

  func testClearIfOwnedSkipsForeignItems() {
    let pasteboard = UIPasteboard.general
    let clipboard = ClipboardManager()

    clipboard.copy("mine", expirationDate: Date().addingTimeInterval(30))
    // Another app takes over the pasteboard: the changeCount no longer
    // matches, so clearing must be a no-op.
    pasteboard.string = "someone else's"

    clipboard.clearIfOwned()
    XCTAssertEqual(pasteboard.string, "someone else's")
  }
}
