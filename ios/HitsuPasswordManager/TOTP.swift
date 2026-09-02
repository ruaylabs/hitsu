import CryptoKit
import Foundation

struct TOTPCode: Equatable, Sendable {
  let code: String
  let remaining: Int
  let period: Int
}

enum TOTPGenerator {
  private enum Algorithm {
    case sha1
    case sha256
    case sha512
  }

  static func code(from uri: String, at date: Date = Date()) -> TOTPCode? {
    guard
      let components = URLComponents(string: uri),
      components.scheme?.lowercased() == "otpauth",
      components.host?.lowercased() == "totp"
    else { return nil }

    let queryItems = components.queryItems ?? []
    guard let secret = value(named: "secret", in: queryItems) else { return nil }

    let period = value(named: "period", in: queryItems).flatMap(Int.init) ?? 30
    let digits = value(named: "digits", in: queryItems).flatMap(Int.init) ?? 6
    let algorithm: Algorithm
    switch value(named: "algorithm", in: queryItems)?.uppercased() ?? "SHA1" {
    case "SHA1", "SHA-1": algorithm = .sha1
    case "SHA256", "SHA-256": algorithm = .sha256
    case "SHA512", "SHA-512": algorithm = .sha512
    default: return nil
    }

    return code(secret: secret, period: period, digits: digits, algorithm: algorithm, at: date)
  }

  static func code(
    fromLegacySecret secret: String,
    settings: String?,
    at date: Date = Date()
  ) -> TOTPCode? {
    let values = settings?.split(separator: ";", omittingEmptySubsequences: false) ?? []
    let period = values.first.flatMap { Int($0) } ?? 30
    let digits = values.dropFirst().first.flatMap { Int($0) } ?? 6
    return code(secret: secret, period: period, digits: digits, algorithm: .sha1, at: date)
  }

  private static func code(
    secret: String,
    period: Int,
    digits: Int,
    algorithm: Algorithm,
    at date: Date
  ) -> TOTPCode? {
    guard period > 0, (1...10).contains(digits), let keyData = decodeBase32(secret) else {
      return nil
    }

    let timestamp = max(0, Int64(date.timeIntervalSince1970.rounded(.down)))
    var counter = UInt64(timestamp / Int64(period)).bigEndian
    let counterData = withUnsafeBytes(of: &counter) { Data($0) }
    let key = SymmetricKey(data: keyData)
    let authenticationCode: Data

    switch algorithm {
    case .sha1:
      authenticationCode = Data(
        HMAC<Insecure.SHA1>.authenticationCode(for: counterData, using: key)
      )
    case .sha256:
      authenticationCode = Data(
        HMAC<SHA256>.authenticationCode(for: counterData, using: key)
      )
    case .sha512:
      authenticationCode = Data(
        HMAC<SHA512>.authenticationCode(for: counterData, using: key)
      )
    }

    guard let lastByte = authenticationCode.last else { return nil }
    let offset = Int(lastByte & 0x0F)
    guard offset + 3 < authenticationCode.count else { return nil }

    let value =
      (UInt32(authenticationCode[offset] & 0x7F) << 24)
      | (UInt32(authenticationCode[offset + 1]) << 16)
      | (UInt32(authenticationCode[offset + 2]) << 8)
      | UInt32(authenticationCode[offset + 3])
    let divisor = (0..<digits).reduce(UInt64(1)) { result, _ in result * 10 }
    let rawCode = String(UInt64(value) % divisor)
    let paddedCode = String(repeating: "0", count: digits - rawCode.count) + rawCode
    let elapsed = Int(timestamp % Int64(period))

    return TOTPCode(code: paddedCode, remaining: period - elapsed, period: period)
  }

  private static func value(named name: String, in items: [URLQueryItem]) -> String? {
    items.first { $0.name.caseInsensitiveCompare(name) == .orderedSame }?.value
  }

  private static func decodeBase32(_ encoded: String) -> Data? {
    let alphabet = Array("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567")
    let normalized = encoded.uppercased().filter {
      !$0.isWhitespace && $0 != "-" && $0 != "="
    }
    guard !normalized.isEmpty else { return nil }

    var result = Data()
    var buffer: UInt64 = 0
    var bitCount = 0

    for character in normalized {
      guard let value = alphabet.firstIndex(of: character) else { return nil }
      buffer = (buffer << 5) | UInt64(value)
      bitCount += 5

      if bitCount >= 8 {
        bitCount -= 8
        result.append(UInt8((buffer >> bitCount) & 0xFF))
        buffer = bitCount == 0 ? 0 : buffer & ((1 << bitCount) - 1)
      }
    }

    return result.isEmpty ? nil : result
  }
}
