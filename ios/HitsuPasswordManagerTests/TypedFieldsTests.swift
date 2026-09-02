import KDBXKit
import XCTest

@testable import HitsuPasswordManager

final class TypedFieldsTests: XCTestCase {
  func testCardFields() {
    let entry = KDBX.Entry(
      uuid: UUID(),
      iconID: 0,
      strings: [
        .init(key: "card.type", value: .unprotected("visa")),
        .init(key: "card.holder", value: .unprotected("Ada Lovelace")),
        .init(key: "card.number", value: .unprotected("4242424242424242")),
        .init(key: "card.expMonth", value: .unprotected("3")),
        .init(key: "card.expYear", value: .unprotected("2027")),
        .init(key: "card.cvv", value: .unprotected("123")),
      ]
    )

    let fields = makeTypedFields(for: .card, entry: entry, protectedNames: [])

    XCTAssertEqual(
      fields.map(\.label),
      ["Type", "Holder", "Number", "Expires", "CVV"]
    )
    XCTAssertEqual(fields.first { $0.label == "Expires" }?.displayValue, "03/2027")
    XCTAssertEqual(fields.first { $0.label == "Type" }?.field, "card.type")
    XCTAssertTrue(fields.first { $0.label == "Number" }!.isProtected)
    XCTAssertTrue(fields.first { $0.label == "CVV" }!.isProtected)
    XCTAssertFalse(fields.first { $0.label == "Holder" }!.isProtected)
  }

  func testIdentityFields() {
    let entry = KDBX.Entry(
      uuid: UUID(),
      iconID: 0,
      strings: [
        .init(key: "identity.firstName", value: .unprotected("Ada")),
        .init(key: "identity.lastName", value: .unprotected("Lovelace")),
        .init(key: "identity.email", value: .unprotected("ada@example.com")),
        .init(key: "identity.phone", value: .unprotected("+1 555 0100")),
        .init(key: "identity.address", value: .unprotected("1 Analytical St")),
        .init(key: "identity.dob", value: .unprotected("1815-12-10")),
      ]
    )

    let fields = makeTypedFields(for: .identity, entry: entry, protectedNames: [])

    XCTAssertEqual(
      fields.map(\.label),
      ["First name", "Last name", "Email", "Phone", "Address", "Date of birth"]
    )
    XCTAssertFalse(fields.contains { $0.isProtected })
  }

  func testProtectedCustomFieldForcesReveal() {
    let entry = KDBX.Entry(
      uuid: UUID(),
      iconID: 0,
      strings: [
        .init(key: "identity.email", value: .protectedInMemory("ada@example.com"))
      ]
    )

    let fields = makeTypedFields(for: .identity, entry: entry, protectedNames: [])

    XCTAssertEqual(fields.first { $0.label == "Email" }?.isProtected, true)
  }

  func testSoftwareLicenseFields() {
    let entry = KDBX.Entry(
      uuid: UUID(),
      iconID: 0,
      strings: [
        .init(key: "license.version", value: .unprotected("2026")),
        .init(key: "license.key", value: .unprotected("ABCD-1234")),
        .init(key: "license.licensedTo", value: .unprotected("Ada")),
        .init(key: "license.company", value: .unprotected("Analytical Engines")),
        .init(key: "license.orderNumber", value: .unprotected("42")),
      ]
    )

    let fields = makeTypedFields(for: .softwareLicense, entry: entry, protectedNames: [])

    XCTAssertEqual(fields.first?.label, "Version")
    XCTAssertTrue(fields.first { $0.label == "License key" }!.isProtected)
    XCTAssertFalse(fields.contains { $0.field == nil })
  }

  func testPassportFields() {
    let entry = KDBX.Entry(
      uuid: UUID(),
      iconID: 0,
      strings: [
        .init(key: "passport.type", value: .unprotected("P")),
        .init(key: "passport.issuingCountry", value: .unprotected("GB")),
        .init(key: "passport.number", value: .unprotected("123456789")),
        .init(key: "passport.fullName", value: .unprotected("Ada Lovelace")),
        .init(key: "passport.expiryDate", value: .unprotected("2030-01-01")),
      ]
    )

    let fields = makeTypedFields(for: .passport, entry: entry, protectedNames: [])

    XCTAssertEqual(
      fields.map(\.label),
      ["Type", "Issuing country", "Number", "Full name", "Expiry date"]
    )
    XCTAssertTrue(fields.first { $0.label == "Number" }!.isProtected)
    XCTAssertFalse(fields.first { $0.label == "Full name" }!.isProtected)
  }

  func testPgpKeyFields() {
    let entry = KDBX.Entry(
      uuid: UUID(),
      iconID: 0,
      strings: [
        .init(key: "pgp.fingerprint", value: .unprotected("ABCD")),
        .init(key: "pgp.keyId", value: .unprotected("0xABCD")),
        .init(key: "pgp.publicKey", value: .unprotected("-----BEGIN PGP PUBLIC KEY-----")),
        .init(key: "pgp.privateKey", value: .unprotected("-----BEGIN PGP PRIVATE KEY-----")),
      ]
    )

    let fields = makeTypedFields(for: .pgpKey, entry: entry, protectedNames: [])

    XCTAssertEqual(
      fields.map(\.label),
      ["Fingerprint", "Key ID", "Public key", "Private key"]
    )
    XCTAssertTrue(fields.first { $0.label == "Private key" }!.isProtected)
    XCTAssertFalse(fields.first { $0.label == "Public key" }!.isProtected)
  }

  func testLoginHasNoTypedFields() {
    let entry = KDBX.Entry(
      uuid: UUID(),
      iconID: 0,
      strings: [
        .init(key: "UserName", value: .unprotected("ada")),
        .init(key: "Password", value: .unprotected("secret")),
      ]
    )

    for category in [VaultEntryCategory.login, .password, .note] {
      XCTAssertTrue(makeTypedFields(for: category, entry: entry, protectedNames: []).isEmpty)
    }
  }
}
