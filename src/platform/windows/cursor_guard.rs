//! Fail-Safe System Cursor Management for Windows.
//!
//! Replaces standard system cursors with a transparent cursor while active,
//! and guarantees cursor restoration on clean exit, unhandled panics, or emergency triggers.

use std::sync::atomic::{AtomicBool, Ordering};
use super::sys::{
    CopyIcon, CreateCursor, DestroyCursor, SetSystemCursor, SystemParametersInfoW,
    SPI_SETCURSORS,
};

/// All standard Windows system cursor resource IDs.
const SYSTEM_CURSOR_IDS: &[u32] = &[
    32512, // OCR_NORMAL
    32513, // OCR_IBEAM
    32514, // OCR_WAIT
    32515, // OCR_CROSS
    32516, // OCR_UP
    32640, // OCR_SIZE
    32641, // OCR_ICON
    32642, // OCR_SIZENWSE
    32643, // OCR_SIZENESW
    32644, // OCR_SIZEWE
    32645, // OCR_SIZENS
    32646, // OCR_SIZEALL
    32648, // OCR_NO
    32649, // OCR_HAND
    32650, // OCR_APPSTARTING
];

static CURSOR_IS_HIDDEN: AtomicBool = AtomicBool::new(false);

/// Restores the default Windows system cursors immediately.
/// Safe to call from any thread or panic hook.
pub fn restore_system_cursors() {
    if CURSOR_IS_HIDDEN.swap(false, Ordering::SeqCst) {
        unsafe {
            // SPI_SETCURSORS (0x0057) forces Windows to reload default cursors
            SystemParametersInfoW(SPI_SETCURSORS, 0, std::ptr::null_mut(), 0);
        }
    }
}

/// Checks if the system cursors are currently hidden.
pub fn is_cursor_hidden() -> bool {
    CURSOR_IS_HIDDEN.load(Ordering::Relaxed)
}

/// RAII Guard that hides system cursors on creation and restores them when dropped.
pub struct SystemCursorGuard {
    _private: (),
}

impl SystemCursorGuard {
    /// Creates a transparent 32x32 cursor and replaces all active system cursors.
    /// Also installs crash filters, panic hooks, and spawns a background watchdog to guarantee cursor restoration.
    pub fn hide() -> Option<Self> {
        install_crash_handlers();
        spawn_watchdog_process();

        // 32x32 1-bit cursor masks:
        // AND mask = 1 (preserve background)
        // XOR mask = 0 (no color inversion)
        // Result: 100% invisible cursor across any background.
        let and_mask = [0xFFu8; 32 * 4];
        let xor_mask = [0x00u8; 32 * 4];

        unsafe {
            let blank_cursor = CreateCursor(
                std::ptr::null_mut(),
                0,
                0,
                32,
                32,
                and_mask.as_ptr(),
                xor_mask.as_ptr(),
            );

            if blank_cursor.is_null() {
                return None;
            }

            for &cursor_id in SYSTEM_CURSOR_IDS {
                // SetSystemCursor takes ownership of the cursor handle and destroys it when replaced.
                // Therefore, we must pass a freshly cloned icon handle for each cursor ID.
                let cursor_copy = CopyIcon(blank_cursor);
                if !cursor_copy.is_null() {
                    SetSystemCursor(cursor_copy, cursor_id);
                }
            }

            DestroyCursor(blank_cursor);
        }

        CURSOR_IS_HIDDEN.store(true, Ordering::SeqCst);

        Some(Self { _private: () })
    }
}

impl Drop for SystemCursorGuard {
    fn drop(&mut self) {
        restore_system_cursors();
    }
}

unsafe extern "system" fn unhandled_crash_callback(_: *mut std::ffi::c_void) -> i32 {
    unsafe {
        SystemParametersInfoW(SPI_SETCURSORS, 0, std::ptr::null_mut(), 0);
    }
    0 // EXCEPTION_CONTINUE_SEARCH
}

/// Installs both SEH crash filters and Rust panic hooks to ensure immediate restoration on abnormal crash.
fn install_crash_handlers() {
    static HOOK_SET: AtomicBool = AtomicBool::new(false);
    if !HOOK_SET.swap(true, Ordering::SeqCst) {
        unsafe {
            super::sys::SetUnhandledExceptionFilter(unhandled_crash_callback as *const _);
        }
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_system_cursors();
            prev_hook(info);
        }));
    }
}

/// Spawns an independent, detached watchdog process that waits for the parent PID handle.
/// If the main process is killed forcibly by Task Manager or an unrecoverable hard crash,
/// the watchdog immediately restores standard Windows cursors.
fn spawn_watchdog_process() {
    static WATCHDOG_STARTED: AtomicBool = AtomicBool::new(false);
    if !WATCHDOG_STARTED.swap(true, Ordering::SeqCst) {
        if let Ok(exe_path) = std::env::current_exe() {
            let my_pid = unsafe { super::sys::GetCurrentProcessId() };
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            const DETACHED_PROCESS: u32 = 0x00000008;

            let _ = std::process::Command::new(exe_path)
                .arg("--watchdog")
                .arg(my_pid.to_string())
                .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
                .spawn();
        }
    }
}
