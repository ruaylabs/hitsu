//! Process-level memory hardening, applied once at startup.
//!
//! The vault is decrypted into ordinary heap memory while unlocked, so the
//! goal here is to close the cheap out-of-process read paths:
//!
//! - **Core dumps** are disabled (`RLIMIT_CORE = 0` on Unix). A crash while
//!   the vault is unlocked can no longer write decrypted entries to disk.
//! - **Linux**: the process is additionally marked non-dumpable
//!   (`prctl(PR_SET_DUMPABLE, 0)`) in release builds. Besides being a second
//!   belt for core dumps, this blocks `ptrace` attach from non-root processes
//!   and makes `/proc/<pid>/mem` unreadable to other users.
//! - **macOS**: release builds call `ptrace(PT_DENY_ATTACH)` so a debugger
//!   cannot attach to the running process.
//!
//! Both anti-attach measures are compiled out of debug builds so local
//! debugging keeps working.
//!
//! ## Known limitation: swap
//!
//! The decrypted `keepass::Database` (and Argon2's 64 MiB work area) live in
//! regular pageable memory and can be written to swap by the OS. `mlock` is
//! not applied: the keepass crate allocates internally so we cannot pin its
//! buffers selectively, and `mlockall(MCL_FUTURE)` risks hard allocation
//! failures under default `RLIMIT_MEMLOCK` limits. Encrypted swap / FileVault
//! (standard on macOS, common on modern Linux installs) is the effective
//! mitigation; secrets that pass through our own DTOs are zeroized on drop.

/// Apply all hardening measures. Failures are logged to stderr and otherwise
/// ignored — a partially hardened process is still better than refusing to
/// start, and none of these calls should fail on a normal desktop system.
pub fn apply() {
    disable_core_dumps();
    deny_attach();
}

#[cfg(unix)]
fn disable_core_dumps() {
    let limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: passing a valid pointer to an initialized rlimit struct.
    if unsafe { libc::setrlimit(libc::RLIMIT_CORE, &limit) } != 0 {
        eprintln!(
            "hardening: failed to disable core dumps: {}",
            std::io::Error::last_os_error()
        );
    }
}

#[cfg(not(unix))]
fn disable_core_dumps() {
    // Windows: there is no core-dump rlimit; crash dumps are produced by WER
    // only if the user/system opted in via registry (LocalDumps), which is
    // off by default. Nothing to do here.
}

#[cfg(all(target_os = "linux", not(debug_assertions)))]
fn deny_attach() {
    // Also forces the kernel to skip core dumps regardless of RLIMIT_CORE.
    // SAFETY: PR_SET_DUMPABLE with arg 0 has no memory-safety concerns.
    if unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0) } != 0 {
        eprintln!(
            "hardening: failed to mark process non-dumpable: {}",
            std::io::Error::last_os_error()
        );
    }
}

#[cfg(all(target_os = "macos", not(debug_assertions)))]
fn deny_attach() {
    // SAFETY: PT_DENY_ATTACH ignores the addr/data arguments.
    if unsafe { libc::ptrace(libc::PT_DENY_ATTACH, 0, std::ptr::null_mut(), 0) } != 0 {
        eprintln!(
            "hardening: failed to deny debugger attach: {}",
            std::io::Error::last_os_error()
        );
    }
}

#[cfg(not(all(any(target_os = "linux", target_os = "macos"), not(debug_assertions))))]
fn deny_attach() {
    // Debug builds and other platforms: keep the process attachable.
}
