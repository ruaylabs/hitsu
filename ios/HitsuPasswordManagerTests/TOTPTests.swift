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

  func testRejectsUnsupportedOTPFormats() {
    let secret = "JBSWY3DPEHPK3PXP"
    let uris = [
      "otpauth://hotp/Test?secret=\(secret)&counter=0",
      "otpauth://steam/Test?secret=\(secret)",
      "https://totp/Test?secret=\(secret)",
      "otpauth://totp/Test?secret=\(secret)&encoder=steam",
      "otpauth://totp/Test?secret=\(secret)&encoder=unknown",
      "otpauth://totp/Test?secret=\(secret)&encoder=",
      "otpauth://totp/Test?secret=\(secret)&encoder=numeric&encoder=steam",
      "otpauth://totp/Test?secret=\(secret)&Encoder=Steam",
      "otpauth://totp/Test?secret=\(secret)&counter=0",
    ]

    for uri in uris {
      XCTAssertNil(TOTPGenerator.code(from: uri))
    }
  }

  func testExplicitNumericEncoderPreservesStandardCode() {
    let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
    for encoder in ["numeric", "NUMERIC"] {
      let uri = "otpauth://totp/Test?secret=\(secret)&encoder=\(encoder)"
      XCTAssertEqual(
        TOTPGenerator.code(from: uri, at: Date(timeIntervalSince1970: 59))?.code,
        "287082"
      )
    }
  }

  func testValidatesBase32BeforeDecoding() {
    let date = Date(timeIntervalSince1970: 59)
    for secret in [
      "", "A", "AAA", "AAAAAA", "MZ", "MZ======", "MZXR", "MZXW7",
      "MZXW6YR", "MY=", "MY=====", "MY=======", "M=Y=====", "========",
      "MZXW6YTB=", "MZXW6YTB========", "M0", "M Y", "M-Y",
    ] {
      var components = URLComponents(string: "otpauth://totp/test")!
      components.queryItems = [URLQueryItem(name: "secret", value: secret)]
      XCTAssertNil(TOTPGenerator.code(from: components.string!, at: date))
      XCTAssertNil(TOTPGenerator.code(fromLegacySecret: secret, settings: nil, at: date))
    }
    for secret in [
      "MY", "MY======", "MZXQ", "MZXQ====", "MZXW6", "MZXW6===",
      "MZXW6YQ", "MZXW6YQ=", "MZXW6YTB", "mzxw6ytb",
    ] {
      var components = URLComponents(string: "otpauth://totp/test")!
      components.queryItems = [URLQueryItem(name: "secret", value: secret)]
      let code = TOTPGenerator.code(from: components.string!, at: date)
      XCTAssertNotNil(code)
      XCTAssertEqual(
        code, TOTPGenerator.code(fromLegacySecret: secret, settings: nil, at: date)
      )
    }
    XCTAssertNil(TOTPGenerator.code(from: "otpauth://totp/test?secret=MY&secret=MY"))
  }

  func testRejectsInvalidConfigurations() {
    XCTAssertNil(TOTPGenerator.code(from: "https://example.com"))
    XCTAssertNil(TOTPGenerator.code(from: "otpauth://hotp/Test?secret=JBSWY3DPEHPK3PXP"))
    XCTAssertNil(TOTPGenerator.code(from: "otpauth://totp/Test?secret=invalid!"))
    XCTAssertNil(TOTPGenerator.code(from: "otpauth://totp/Test?secret=JBSWY3DPEHPK3PXP&period=0"))
    XCTAssertNil(TOTPGenerator.code(from: "otpauth://totp/Test?secret=JBSWY3DPEHPK3PXP&digits=20"))
  }
}
