import XCTest

@testable import HitsuPasswordManager

final class CardNumberTests: XCTestCase {
  func testFormatsGroupsOfFour() {
    XCTAssertEqual(
      formatCardNumber("4111111111111111", cardType: "visa"),
      "4111 1111 1111 1111"
    )
    XCTAssertEqual(formatCardNumber("4111-1111-1111-1111", cardType: nil), "4111 1111 1111 1111")
  }

  func testFormatsAmexByType() {
    XCTAssertEqual(
      formatCardNumber("378282246310005", cardType: "amex"),
      "3782 822463 10005"
    )
    XCTAssertEqual(
      formatCardNumber("378282246310005", cardType: "American Express"),
      "3782 822463 10005"
    )
  }

  func testFormatsAmexByPrefix() {
    XCTAssertEqual(formatCardNumber("378282246310005", cardType: nil), "3782 822463 10005")
    XCTAssertEqual(formatCardNumber("371234567890123", cardType: "visa"), "3712 3456 7890 123")
  }

  func testFormatFallsBackToRawWithoutDigits() {
    XCTAssertEqual(formatCardNumber("n/a", cardType: "visa"), "n/a")
  }

  func testMasksLongNumbers() {
    XCTAssertEqual(maskCardNumber("4111111111111111"), "4111 •••• 1111")
    XCTAssertEqual(maskCardNumber("123456789012"), "1234 •••• 9012")
  }

  func testMasksShortNumbers() {
    XCTAssertEqual(maskCardNumber("4111"), "••••")
  }

  func testMaskNilForEmpty() {
    XCTAssertNil(maskCardNumber(""))
  }

  func testBrandNames() {
    XCTAssertEqual(cardBrandName(for: "visa"), "Visa")
    XCTAssertEqual(cardBrandName(for: "AMEX"), "American Express")
    XCTAssertEqual(cardBrandName(for: "American Express"), "American Express")
    XCTAssertEqual(cardBrandName(for: "store card"), "store card")
  }
}
