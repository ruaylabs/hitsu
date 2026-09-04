import CryptoKit
import Foundation
import KDBXKit
import LocalAuthentication
import Security

enum BiometricKind: Sendable {
  case faceID
  case touchID

  var title: String {
    switch self {
    case .faceID: "Face ID"
    case .touchID: "Touch ID"
    }
  }

  var systemImage: String {
    switch self {
    case .faceID: "faceid"
    case .touchID: "touchid"
    }
  }
}

enum BiometricUnlockError: LocalizedError, Sendable {
  case unavailable
  case cancelled
  case failed
  case credentialUnavailable
  case invalidCredential

  var errorDescription: String? {
    switch self {
    case .unavailable:
      "Biometric unlock is not available on this device."
    case .cancelled:
      "Biometric unlock was cancelled."
    case .failed:
      "Biometric unlock failed."
    case .credentialUnavailable:
      "Biometric unlock is no longer available. Unlock with your password to set it up again."
    case .invalidCredential:
      "The saved biometric credential is invalid. Unlock with your password to set it up again."
    }
  }
}

@MainActor
enum BiometricAuthenticator {
  static var availableBiometric: BiometricKind? {
    let context = LAContext()
    var error: NSError?
    guard
      context.canEvaluatePolicy(
        .deviceOwnerAuthenticationWithBiometrics, error: &error
      )
    else {
      return nil
    }
    switch context.biometryType {
    case .faceID:
      return .faceID
    case .touchID:
      return .touchID
    default:
      return nil
    }
  }

  /// Authenticates with the device biometric and rehydrates the vault unlock
  /// from a biometric-protected Keychain item. Per KDBXKit's documented
  /// biometric pattern (``UnlockData/init(rawKeyData:)``,
  /// ``UnlockData/keyDataBytes``), what is stored is the 32-byte KDBX
  /// pre-hash, not the master password — it grants the same unlock authority
  /// for this vault without keeping a reusable password in the Keychain.
  static func retrieveUnlockData(
    for bookmark: Data,
    kind: BiometricKind
  ) async throws -> UnlockData {
    let context = LAContext()
    var availabilityError: NSError?
    guard
      context.canEvaluatePolicy(
        .deviceOwnerAuthenticationWithBiometrics, error: &availabilityError
      )
    else {
      throw BiometricUnlockError.unavailable
    }

    do {
      guard
        try await context.evaluatePolicy(
          .deviceOwnerAuthenticationWithBiometrics,
          localizedReason: "Unlock your Hitsu vault with \(kind.title)."
        )
      else {
        throw BiometricUnlockError.failed
      }
    } catch let error as LAError {
      switch error.code {
      case .userCancel, .systemCancel, .appCancel:
        throw BiometricUnlockError.cancelled
      default:
        throw BiometricUnlockError.failed
      }
    }

    let data: Data
    do {
      data = try BiometricCredentialStore.load(for: bookmark, context: context)
    } catch {
      throw BiometricUnlockError.credentialUnavailable
    }
    // UnlockData.init(rawKeyData:) traps on anything but 32 bytes; validate
    // first so a corrupted item surfaces as an error instead of a crash.
    guard data.count == 32 else {
      throw BiometricUnlockError.invalidCredential
    }
    return UnlockData(rawKeyData: data)
  }
}

/// Stores one vault unlock pre-hash (KDBXKit `UnlockData.keyDataBytes`, 32
/// bytes — not the master password) per bookmark in a Keychain item protected
/// by the currently enrolled biometric set. A non-secret UserDefaults marker
/// only tells the UI whether to offer the biometric button without prompting.
enum BiometricCredentialStore {
  private static let service = "com.ruaylabs.hitsu.biometricVaultPassword"
  private static let markerPrefix = "biometricCredential."

  static func save(_ keyData: Data, for bookmark: Data) -> Bool {
    let account = account(for: bookmark)
    var accessControlError: Unmanaged<CFError>?
    guard
      let accessControl = SecAccessControlCreateWithFlags(
        nil,
        // Not WhenPasscodeSetThisDeviceOnly: that class is documented as
        // available only when a passcode is set, which the simulator never has —
        // the add would fail there. WhenUnlockedThisDeviceOnly still keeps the
        // item out of backups while .biometryCurrentSet gates every read.
        kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
        .biometryCurrentSet,
        &accessControlError
      )
    else {
      return false
    }

    SecItemDelete(baseQuery(account: account) as CFDictionary)
    var query = baseQuery(account: account)
    query[kSecValueData as String] = keyData
    query[kSecAttrAccessControl as String] = accessControl
    guard SecItemAdd(query as CFDictionary, nil) == errSecSuccess else { return false }
    UserDefaults.standard.set(true, forKey: markerKey(account: account))
    return true
  }

  static func load(for bookmark: Data, context: LAContext) throws -> Data {
    let account = account(for: bookmark)
    var query = baseQuery(account: account)
    query[kSecReturnData as String] = true
    query[kSecMatchLimit as String] = kSecMatchLimitOne
    query[kSecUseAuthenticationContext as String] = context
    var result: AnyObject?
    guard SecItemCopyMatching(query as CFDictionary, &result) == errSecSuccess else {
      throw BiometricUnlockError.credentialUnavailable
    }
    guard let data = result as? Data else {
      throw BiometricUnlockError.invalidCredential
    }
    return data
  }

  static func remove(for bookmark: Data) {
    let account = account(for: bookmark)
    SecItemDelete(baseQuery(account: account) as CFDictionary)
    UserDefaults.standard.removeObject(forKey: markerKey(account: account))
  }

  static func hasSavedCredential(for bookmark: Data) -> Bool {
    UserDefaults.standard.bool(forKey: markerKey(account: account(for: bookmark)))
  }

  private static func account(for bookmark: Data) -> String {
    SHA256.hash(data: bookmark).map { String(format: "%02x", $0) }.joined()
  }

  private static func markerKey(account: String) -> String {
    markerPrefix + account
  }

  private static func baseQuery(account: String) -> [String: Any] {
    [
      kSecClass as String: kSecClassGenericPassword,
      kSecAttrService as String: service,
      kSecAttrAccount as String: account,
    ]
  }
}
