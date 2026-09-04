use std::path::Path;

use core_foundation::base::{CFRelease, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::{CFString, CFStringRef};
use objc2_local_authentication::{LABiometryType, LAContext, LAPolicy};
use security_framework::access_control::{ProtectionMode, SecAccessControl};
use security_framework::base::Error as SecurityError;
use security_framework::passwords::{
    delete_generic_password_options, generic_password, set_generic_password_options,
    AccessControlOptions, PasswordOptions,
};
use security_framework_sys::base::{
    errSecAuthFailed as ERR_SEC_AUTH_FAILED, errSecItemNotFound as ERR_SEC_ITEM_NOT_FOUND,
    errSecSuccess as ERR_SEC_SUCCESS,
};
use security_framework_sys::item::{
    kSecAttrAccount, kSecAttrService, kSecAttrSynchronizable, kSecClass, kSecClassGenericPassword,
    kSecReturnData, kSecUseAuthenticationUI, kSecUseDataProtectionKeychain,
};
use security_framework_sys::keychain_item::SecItemCopyMatching;
use zeroize::Zeroizing;

use super::{account_for_path, BiometricError, BiometricResult};

const SERVICE: &str = "com.ruaylabs.hitsu.touchid";
const ERR_SEC_USER_CANCELED: i32 = -128;
const ERR_SEC_NOT_AVAILABLE: i32 = -25291;
const ERR_SEC_INTERACTION_NOT_ALLOWED: i32 = -25308;
const ERR_SEC_MISSING_ENTITLEMENT: i32 = -34018;

// security-framework-sys exposes the query key but not this documented value.
// It is used only for the status probe so checking whether an item exists can
// never display an authentication prompt.
extern "C" {
    static kSecUseAuthenticationUIFail: CFStringRef;
}

fn password_options(account: &str) -> PasswordOptions {
    let mut options = PasswordOptions::new_generic_password(SERVICE, account);
    options.use_protected_keychain();
    // Keep reads and deletes scoped to the local store as well as ensuring
    // newly added credentials can never synchronize through iCloud Keychain.
    options.set_access_synchronized(Some(false));
    options
}

fn map_status(status: i32) -> BiometricError {
    match status {
        ERR_SEC_ITEM_NOT_FOUND => BiometricError::NotFound,
        ERR_SEC_USER_CANCELED => BiometricError::Canceled,
        ERR_SEC_AUTH_FAILED => BiometricError::AuthenticationFailed,
        ERR_SEC_NOT_AVAILABLE | ERR_SEC_MISSING_ENTITLEMENT => BiometricError::Unavailable,
        other => BiometricError::Keychain(other),
    }
}

fn map_security_error(error: SecurityError) -> BiometricError {
    map_status(error.code())
}

pub(crate) fn is_available() -> bool {
    // SAFETY: LAContext is available at the app's 10.15 deployment target and
    // is created, queried, and released entirely within this worker thread.
    unsafe {
        let context = LAContext::new();
        context
            .canEvaluatePolicy_error(LAPolicy::DeviceOwnerAuthenticationWithBiometrics)
            .is_ok()
            && context.biometryType() == LABiometryType::TouchID
    }
}

pub(crate) fn store_password(path: &Path, password: &str) -> BiometricResult<()> {
    if !is_available() {
        return Err(BiometricError::Unavailable);
    }

    let account = account_for_path(path)?;
    delete_by_account(&account)?;

    let access_control = SecAccessControl::create_with_protection(
        Some(ProtectionMode::AccessibleWhenPasscodeSetThisDeviceOnly),
        AccessControlOptions::BIOMETRY_CURRENT_SET.bits(),
    )
    .map_err(map_security_error)?;
    let mut options = password_options(&account);
    options.set_access_control(access_control);
    set_generic_password_options(password.as_bytes(), options).map_err(map_security_error)
}

pub(crate) fn read_password(path: &Path) -> BiometricResult<Zeroizing<String>> {
    let account = account_for_path(path)?;
    let bytes =
        Zeroizing::new(generic_password(password_options(&account)).map_err(map_security_error)?);
    let password =
        std::str::from_utf8(bytes.as_slice()).map_err(|_| BiometricError::InvalidData)?;
    Ok(Zeroizing::new(password.to_owned()))
}

pub(crate) fn delete_password(path: &Path) -> BiometricResult<()> {
    let account = account_for_path(path)?;
    delete_by_account(&account)
}

fn delete_by_account(account: &str) -> BiometricResult<()> {
    match delete_generic_password_options(password_options(account)) {
        Ok(()) => Ok(()),
        Err(error) if error.code() == ERR_SEC_ITEM_NOT_FOUND => Ok(()),
        Err(error) => Err(map_security_error(error)),
    }
}

pub(crate) fn item_exists(path: &Path) -> BiometricResult<bool> {
    let account = account_for_path(path)?;

    // SAFETY: every imported kSec constant is a process-lifetime CFString;
    // from_CFType_pairs retains the owned query values for the dictionary.
    let query = unsafe {
        CFDictionary::from_CFType_pairs(&[
            (
                CFString::wrap_under_get_rule(kSecClass),
                CFString::wrap_under_get_rule(kSecClassGenericPassword).into_CFType(),
            ),
            (
                CFString::wrap_under_get_rule(kSecAttrService),
                CFString::from(SERVICE).into_CFType(),
            ),
            (
                CFString::wrap_under_get_rule(kSecAttrAccount),
                CFString::from(account.as_str()).into_CFType(),
            ),
            (
                CFString::wrap_under_get_rule(kSecUseDataProtectionKeychain),
                CFBoolean::from(true).into_CFType(),
            ),
            (
                CFString::wrap_under_get_rule(kSecAttrSynchronizable),
                CFBoolean::from(false).into_CFType(),
            ),
            (
                CFString::wrap_under_get_rule(kSecReturnData),
                CFBoolean::from(true).into_CFType(),
            ),
            (
                CFString::wrap_under_get_rule(kSecUseAuthenticationUI),
                CFString::wrap_under_get_rule(kSecUseAuthenticationUIFail).into_CFType(),
            ),
        ])
    };

    let mut result = std::ptr::null();
    // SAFETY: query is a valid CFDictionary and result is an initialized
    // out-pointer. A non-null result follows the Core Foundation create rule.
    let status = unsafe { SecItemCopyMatching(query.as_concrete_TypeRef(), &mut result) };
    if !result.is_null() {
        // SAFETY: SecItemCopyMatching returned this retained object above.
        unsafe { CFRelease(result) };
    }

    match status {
        ERR_SEC_SUCCESS | ERR_SEC_INTERACTION_NOT_ALLOWED | ERR_SEC_AUTH_FAILED => Ok(true),
        ERR_SEC_ITEM_NOT_FOUND => Ok(false),
        other => Err(map_status(other)),
    }
}
