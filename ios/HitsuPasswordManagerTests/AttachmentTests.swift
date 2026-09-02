import XCTest

@testable import HitsuPasswordManager

final class AttachmentTests: XCTestCase {
  func testFormatsBytes() {
    XCTAssertEqual(formatAttachmentSize(0), "0 B")
    XCTAssertEqual(formatAttachmentSize(512), "512 B")
    XCTAssertEqual(formatAttachmentSize(1023), "1023 B")
  }

  func testFormatsKilobytes() {
    XCTAssertEqual(formatAttachmentSize(1024), "1.0 KB")
    XCTAssertEqual(formatAttachmentSize(1536), "1.5 KB")
  }

  func testFormatsMegabytes() {
    XCTAssertEqual(formatAttachmentSize(1024 * 1024), "1.0 MB")
    XCTAssertEqual(formatAttachmentSize(256 * 1024 * 1024), "256.0 MB")
  }
}
