import XCTest

@testable import HitsuPasswordManager

final class PasswordStrengthTests: XCTestCase {
  func testTreatsEmptyAndShortPasswordsAsTooShort() {
    let empty = estimatePasswordStrength("")
    XCTAssertEqual(empty.level, 0)
    XCTAssertEqual(empty.fraction, 0)
    XCTAssertEqual(empty.label, "Too short")

    XCTAssertEqual(estimatePasswordStrength("Ab1!").level, 0)
  }

  func testRewardsLengthAndCharacterDiversity() {
    XCTAssertEqual(
      estimatePasswordStrength("Correct-Horse-Battery-Staple-42").level, 4
    )
  }

  func testPenalizesCommonPasswords() {
    XCTAssertEqual(estimatePasswordStrength("password").level, 0)
  }

  func testCatchesCommonPasswordsDespiteCaseLeetSwapsAndPadding() {
    XCTAssertLessThanOrEqual(estimatePasswordStrength("Password123!").level, 1)
    XCTAssertLessThanOrEqual(estimatePasswordStrength("qwerty2024").level, 1)
    XCTAssertEqual(estimatePasswordStrength("P4ssw0rd").level, 0)
    XCTAssertLessThanOrEqual(estimatePasswordStrength("!!Sunshine99!!").level, 1)
  }

  func testCapsRepeatedAndSequentialRunsAtZeroWithHonestLabel() {
    XCTAssertEqual(estimatePasswordStrength("aaaaaaaaaaaa").level, 0)

    let sequential = estimatePasswordStrength("abcdefghijkl")
    XCTAssertEqual(sequential.level, 0)
    XCTAssertEqual(sequential.label, "Very weak")

    XCTAssertEqual(estimatePasswordStrength("0123456789").level, 0)

    XCTAssertEqual(estimatePasswordStrength("iloveyou").label, "Very weak")
  }

  func testDoesNotFlagStrongPassphrasesThatMerelyContainACommonWord() {
    XCTAssertEqual(
      estimatePasswordStrength("Correct-Horse-Battery-Staple-42").level, 4
    )
    XCTAssertEqual(
      estimatePasswordStrength("sunshine-glacier-parrot-42x").level, 4
    )
  }

  func testColorsMatchDesktopBands() {
    XCTAssertEqual(strengthColor(for: 0), .red)
    XCTAssertEqual(strengthColor(for: 1), .red)
    XCTAssertEqual(strengthColor(for: 2), .orange)
    XCTAssertEqual(strengthColor(for: 3), .green)
    XCTAssertEqual(strengthColor(for: 4), .green)
  }
}
