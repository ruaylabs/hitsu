import Foundation
import XCTest

@testable import HitsuPasswordManager

final class TOTPTests: XCTestCase {
  func testGeneratesRFC6238SHA1Codes() {
    let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
    let fixtures: [(timestamp: TimeInterval, code: String)] = [
      (59, "94287082"),
      (1_111_111_109, "07081804"),
      (1_111_111_111, "14050471"),
      (1_234_567_890, "89005924"),
      (2_000_000_000, "69279037"),
      (20_000_000_000, "65353130"),
    ]

    for fixture in fixtures {
      let uri = "otpauth://totp/Test?secret=\(secret)&period=30&digits=8&algorithm=SHA1"
      let result = TOTPGenerator.code(
        from: uri,
        at: Date(timeIntervalSince1970: fixture.timestamp)
      )
      XCTAssertEqual(result?.code, fixture.code)
    }
  }

  func testSupportsSHA256AndSHA512() {
    let fixtures = [
      (
        secret: "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZA====",
        algorithm: "SHA256",
        code: "46119246"
      ),
      (
        secret: "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
          + "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNA=",
        algorithm: "SHA512",
        code: "90693936"
      ),
    ]

    for fixture in fixtures {
      let uri =
        "otpauth://totp/Test?secret=\(fixture.secret)&digits=8"
        + "&algorithm=\(fixture.algorithm)"
      XCTAssertEqual(
        TOTPGenerator.code(from: uri, at: Date(timeIntervalSince1970: 59))?.code,
        fixture.code
      )
    }
  }

  func testReportsRemainingLifetime() {
    let uri = "otpauth://totp/Test?secret=JBSWY3DPEHPK3PXP&period=30&digits=6"

    XCTAssertEqual(
      TOTPGenerator.code(from: uri, at: Date(timeIntervalSince1970: 30))?.remaining,
      30
    )
    XCTAssertEqual(
      TOTPGenerator.code(from: uri, at: Date(timeIntervalSince1970: 59))?.remaining,
      1
    )
  }

  func testSupportsLegacyKeePassFields() {
    let result = TOTPGenerator.code(
      fromLegacySecret: "JBSWY3DPEHPK3PXP",
      settings: "30;6",
      at: Date(timeIntervalSince1970: 59)
    )

    XCTAssertEqual(result?.code, "996554")
    XCTAssertEqual(result?.period, 30)
    XCTAssertEqual(result?.remaining, 1)
  }

  func testRejectsInvalidConfigurations() {
    XCTAssertNil(TOTPGenerator.code(from: "https://example.com"))
    XCTAssertNil(TOTPGenerator.code(from: "otpauth://hotp/Test?secret=JBSWY3DPEHPK3PXP"))
    XCTAssertNil(TOTPGenerator.code(from: "otpauth://totp/Test?secret=invalid!"))
    XCTAssertNil(TOTPGenerator.code(from: "otpauth://totp/Test?secret=JBSWY3DPEHPK3PXP&period=0"))
    XCTAssertNil(TOTPGenerator.code(from: "otpauth://totp/Test?secret=JBSWY3DPEHPK3PXP&digits=20"))
  }
}
