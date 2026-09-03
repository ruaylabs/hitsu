import XCTest

@testable import HitsuPasswordManager

final class MetadataTests: XCTestCase {
  func testDecodesBinaryMetadata() {
    let fixtures = [
      "bG9naW4=": "login",
      "cGFzc3dvcmQ=": "password",
      "bm90ZQ==": "note",
      "aWRlbnRpdHk=": "identity",
      "Y2FyZA==": "card",
      "c29mdHdhcmVfbGljZW5zZQ==": "software_license",
      "cGFzc3BvcnQ=": "passport",
      "cGdwX2tleQ==": "pgp_key",
      "dHJ1ZQ==": "true",
    ]

    for (encoded, expected) in fixtures {
      XCTAssertEqual(
        resolvedHitsuMetadataValue(currentValue: encoded, legacyValue: nil),
        expected
      )
    }
  }

  func testPreservesPlainAndMalformedMetadata() {
    XCTAssertEqual(
      resolvedHitsuMetadataValue(currentValue: "software_license", legacyValue: nil),
      "software_license"
    )
    XCTAssertEqual(
      resolvedHitsuMetadataValue(currentValue: "not-base64!", legacyValue: nil),
      "not-base64!"
    )
  }

  func testUsesLegacyMetadataOnlyAsFallback() {
    XCTAssertEqual(
      resolvedHitsuMetadataValue(currentValue: nil, legacyValue: "bm90ZQ=="),
      "note"
    )
    XCTAssertEqual(
      resolvedHitsuMetadataValue(currentValue: "Y2FyZA==", legacyValue: "bm90ZQ=="),
      "card"
    )
  }

  func testMissingMetadataReturnsNil() {
    XCTAssertNil(resolvedHitsuMetadataValue(currentValue: nil, legacyValue: nil))
  }

  func testUnknownCategoryDefaultsToLogin() {
    XCTAssertEqual(VaultEntryCategory(databaseValue: "unknown"), .login)
    XCTAssertEqual(VaultEntryCategory(databaseValue: nil), .login)
  }

  func testCustomFieldDisplayNameRemovesStoragePrefix() {
    XCTAssertEqual(
      VaultField(name: "custom.API key", isProtected: true).displayName,
      "API key"
    )
    XCTAssertEqual(
      VaultField(name: "PluginData", isProtected: false).displayName,
      "PluginData"
    )
  }

  func testValidatedURLAllowsOnlyHTTPAndHTTPS() {
    XCTAssertEqual(
      validatedHTTPURL("example.com")?.absoluteString,
      "https://example.com"
    )
    XCTAssertEqual(
      validatedHTTPURL("HTTPS://example.com/login")?.absoluteString,
      "HTTPS://example.com/login"
    )
    XCTAssertNil(validatedHTTPURL("javascript://alert"))
    XCTAssertNil(validatedHTTPURL("custom-scheme://value"))
    XCTAssertNil(validatedHTTPURL("not a valid host"))
  }
}
