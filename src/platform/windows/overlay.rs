//! Win32 Transparent Topmost Layered Overlay Window.
//!
//! Creates a click-through, non-activating topmost window with per-monitor DPI v2 awareness.

use std::ffi::c_void;
use std::ptr::null_mut;
use super::renderer::CANVAS_SIZE;
use super::sys::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetModuleHandleW,
    GetProcAddress, HMENU, HINSTANCE, PeekMessageW, RegisterClassExW, SetProcessDpiAwarenessContext,
    SetWindowPos, ShowWindow, TranslateMessage, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, HWND,
    HWND_TOPMOST, LPARAM, LRESULT, MSG, PM_REMOVE, SW_SHOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOOWNERZORDER, SWP_SHOWWINDOW, WNDCLASSEXW, WM_QUIT, WPARAM,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_EX_TRANSPARENT, WS_POPUP, GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, HTTRANSPARENT, MA_NOACTIVATE, WM_DESTROY,
    WM_MOUSEACTIVATE, WM_NCHITTEST, SetLayeredWindowAttributes, LWA_ALPHA,
};



const WINDOW_CLASS_NAME: &[u16] = &[
    b'T' as u16, b'i' as u16, b'n' as u16, b'y' as u16,
    b'C' as u16, b'u' as u16, b'r' as u16, b's' as u16,
    b'o' as u16, b'r' as u16, b'O' as u16, b'v' as u16,
    b'e' as u16, b'r' as u16, b'l' as u16, b'a' as u16,
    b'y' as u16, 0,
];

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        // HTTRANSPARENT (-1) tells the Windows Input Manager that this window is 100%
        // non-existent for hit-testing, routing all clicks, touches, pen, and gestures
        // immediately to whatever window or desktop element is beneath it.
        WM_NCHITTEST => HTTRANSPARENT,
        // MA_NOACTIVATE (3) ensures clicks never steal focus or activate the overlay.
        WM_MOUSEACTIVATE => MA_NOACTIVATE,
        WM_DESTROY => {
            super::sys::PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

pub const ZBID_DEFAULT: u32 = 0;
pub const ZBID_DESKTOP: u32 = 1;
pub const ZBID_UIACCESS: u32 = 12;
pub const ZBID_IMMERSIVE_NOTIFICATION: u32 = 6;
pub const ZBID_IMMERSIVE_ACTIVEMOBODY: u32 = 8;
pub const ZBID_SYSTEM_TOOLS: u32 = 11;

type PfnCreateWindowInBand = unsafe extern "system" fn(
    u32,
    *const u16,
    *const u16,
    u32,
    i32,
    i32,
    i32,
    i32,
    HWND,
    HMENU,
    HINSTANCE,
    *mut c_void,
    u32,
) -> HWND;

type PfnSetWindowBand = unsafe extern "system" fn(HWND, HWND, u32) -> super::sys::BOOL;

/// A native Windows transparent overlay window.
pub struct OverlayWindow {
    hwnd: HWND,
    is_in_band: bool,
}

impl OverlayWindow {
    /// Creates and initializes the transparent overlay window.
    /// Dual-Tier Architecture: Attempts elevated Z-Bands (ZBID_UIACCESS / ZBID_SYSTEM_TOOLS)
    /// with graceful, seamless fallback to standard Desktop Topmost.
    pub fn new() -> Option<Self> {
        unsafe {
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            let hinstance = GetModuleHandleW(null_mut());

            let mut wc: WNDCLASSEXW = std::mem::zeroed();
            wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.lpfn_wnd_proc = Some(window_proc);
            wc.h_instance = hinstance;
            wc.lpsz_class_name = WINDOW_CLASS_NAME.as_ptr();

            RegisterClassExW(&wc);

            let ex_style = WS_EX_TOPMOST
                | WS_EX_TRANSPARENT
                | WS_EX_LAYERED
                | WS_EX_TOOLWINDOW
                | WS_EX_NOACTIVATE;

            let user32_name: &[u16] = &[
                b'u' as u16, b's' as u16, b'e' as u16, b'r' as u16,
                b'3' as u16, b'2' as u16, b'.' as u16, b'd' as u16,
                b'l' as u16, b'l' as u16, 0,
            ];
            let user32_mod = GetModuleHandleW(user32_name.as_ptr());
            let mut p_create_in_band: Option<PfnCreateWindowInBand> = None;
            let mut p_set_window_band: Option<PfnSetWindowBand> = None;

            if !user32_mod.is_null() {
                let p_create = GetProcAddress(user32_mod, b"CreateWindowInBand\0".as_ptr());
                if !p_create.is_null() {
                    p_create_in_band = Some(std::mem::transmute(p_create));
                }
                let p_set = GetProcAddress(user32_mod, b"SetWindowBand\0".as_ptr());
                if !p_set.is_null() {
                    p_set_window_band = Some(std::mem::transmute(p_set));
                }
            }

            let mut hwnd: HWND = null_mut();
            let mut is_in_band = false;

            // Tier 1: Try elevated Z-Bands (ZBID_UIACCESS = 12, ZBID_SYSTEM_TOOLS = 11)
            if let Some(create_in_band) = p_create_in_band {
                for &target_band in &[ZBID_UIACCESS, ZBID_SYSTEM_TOOLS, ZBID_IMMERSIVE_NOTIFICATION] {
                    hwnd = create_in_band(
                        ex_style,
                        WINDOW_CLASS_NAME.as_ptr(),
                        WINDOW_CLASS_NAME.as_ptr(),
                        WS_POPUP,
                        -1000,
                        -1000,
                        CANVAS_SIZE,
                        CANVAS_SIZE,
                        null_mut(),
                        null_mut(),
                        hinstance,
                        null_mut(),
                        target_band,
                    );
                    if !hwnd.is_null() {
                        is_in_band = true;
                        if let Some(set_band) = p_set_window_band {
                            let _ = set_band(hwnd, null_mut(), target_band);
                        }
                        break;
                    }
                }
            }

            // Tier 2: Standard Desktop Topmost fallback
            if hwnd.is_null() {
                hwnd = CreateWindowExW(
                    ex_style,
                    WINDOW_CLASS_NAME.as_ptr(),
                    WINDOW_CLASS_NAME.as_ptr(),
                    WS_POPUP,
                    -1000,
                    -1000,
                    CANVAS_SIZE,
                    CANVAS_SIZE,
                    null_mut(),
                    null_mut(),
                    hinstance,
                    null_mut(),
                );
            }

            if hwnd.is_null() {
                return None;
            }

            SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);
            ShowWindow(hwnd, SW_SHOW);
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );

            Some(Self { hwnd, is_in_band })
        }
    }

    /// Creates and initializes a transparent full-desktop overlay window
    /// designed specifically for DirectComposition and DXGI Modern Flip Model.
    /// Dual-Tier Architecture: Attempts elevated Z-Bands (ZBID_UIACCESS / ZBID_SYSTEM_TOOLS)
    /// with graceful, seamless fallback to standard Desktop Topmost.
    /// Returns (OverlayWindow, vx, vy, width, height) covering all active displays.
    pub fn new_direct_composition() -> Option<(Self, i32, i32, u32, u32)> {
        unsafe {
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            let hinstance = GetModuleHandleW(null_mut());

            let mut wc: WNDCLASSEXW = std::mem::zeroed();
            wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.lpfn_wnd_proc = Some(window_proc);
            wc.h_instance = hinstance;
            wc.lpsz_class_name = WINDOW_CLASS_NAME.as_ptr();

            RegisterClassExW(&wc);

            // Extended styles for DirectComposition overlay:
            // WS_EX_LAYERED + WS_EX_TRANSPARENT ensures the Windows User32 input manager
            // completely bypasses this window for all mouse, touch, and pen hit-testing,
            // passing all clicks to whatever window or control is beneath it.
            // WS_EX_NOREDIRECTIONBITMAP delegates composition directly to the DirectX SwapChain.
            let ex_style = WS_EX_TOPMOST
                | WS_EX_TRANSPARENT
                | WS_EX_LAYERED
                | WS_EX_NOREDIRECTIONBITMAP
                | WS_EX_TOOLWINDOW
                | WS_EX_NOACTIVATE;

            let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let vw = (GetSystemMetrics(SM_CXVIRTUALSCREEN) as u32).max(1920);
            let vh = (GetSystemMetrics(SM_CYVIRTUALSCREEN) as u32).max(1080);

            let user32_name: &[u16] = &[
                b'u' as u16, b's' as u16, b'e' as u16, b'r' as u16,
                b'3' as u16, b'2' as u16, b'.' as u16, b'd' as u16,
                b'l' as u16, b'l' as u16, 0,
            ];
            let user32_mod = GetModuleHandleW(user32_name.as_ptr());
            let mut p_create_in_band: Option<PfnCreateWindowInBand> = None;
            let mut p_set_window_band: Option<PfnSetWindowBand> = None;

            if !user32_mod.is_null() {
                let p_create = GetProcAddress(user32_mod, b"CreateWindowInBand\0".as_ptr());
                if !p_create.is_null() {
                    p_create_in_band = Some(std::mem::transmute(p_create));
                }
                let p_set = GetProcAddress(user32_mod, b"SetWindowBand\0".as_ptr());
                if !p_set.is_null() {
                    p_set_window_band = Some(std::mem::transmute(p_set));
                }
            }

            let mut hwnd: HWND = null_mut();
            let mut is_in_band = false;

            // Tier 1: Try elevated Z-Bands (ZBID_UIACCESS = 12, ZBID_SYSTEM_TOOLS = 11)
            if let Some(create_in_band) = p_create_in_band {
                for &target_band in &[ZBID_UIACCESS, ZBID_SYSTEM_TOOLS, ZBID_IMMERSIVE_NOTIFICATION] {
                    hwnd = create_in_band(
                        ex_style,
                        WINDOW_CLASS_NAME.as_ptr(),
                        WINDOW_CLASS_NAME.as_ptr(),
                        WS_POPUP,
                        vx,
                        vy,
                        vw as i32,
                        vh as i32,
                        null_mut(),
                        null_mut(),
                        hinstance,
                        null_mut(),
                        target_band,
                    );
                    if !hwnd.is_null() {
                        is_in_band = true;
                        if let Some(set_band) = p_set_window_band {
                            let _ = set_band(hwnd, null_mut(), target_band);
                        }
                        break;
                    }
                }
            }

            // Tier 2: Standard Desktop Topmost fallback
            if hwnd.is_null() {
                hwnd = CreateWindowExW(
                    ex_style,
                    WINDOW_CLASS_NAME.as_ptr(),
                    WINDOW_CLASS_NAME.as_ptr(),
                    WS_POPUP,
                    vx,
                    vy,
                    vw as i32,
                    vh as i32,
                    null_mut(),
                    null_mut(),
                    hinstance,
                    null_mut(),
                );
            }

            if hwnd.is_null() {
                return None;
            }

            // Initialize layered window state with full opacity (255) so DirectComposition can draw
            // and User32 hit-testing bypasses this window cleanly.
            SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);

            ShowWindow(hwnd, SW_SHOW);
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                vx,
                vy,
                vw as i32,
                vh as i32,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );

            Some((Self { hwnd, is_in_band }, vx, vy, vw, vh))
        }
    }

    /// Returns the Win32 window handle.
    #[inline]
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Returns true if the window is successfully hosted within an elevated DWM Z-Band.
    #[inline]
    pub fn is_in_band(&self) -> bool {
        self.is_in_band
    }

    /// Continuously reinforces HWND_TOPMOST priority so the cursor stays above
    /// the Windows Taskbar, Start Menu, and all full-screen or foreground windows.
    #[inline]
    pub fn reinforce_topmost(&self) {
        unsafe {
            if !self.hwnd.is_null() {
                SetWindowPos(
                    self.hwnd,
                    HWND_TOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_SHOWWINDOW,
                );
            }
        }
    }

    /// Pumps pending Windows messages. Returns `false` if WM_QUIT was encountered.
    pub fn poll_events(&self) -> bool {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    return false;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        true
    }
}

impl Drop for OverlayWindow {
    fn drop(&mut self) {
        unsafe {
            if !self.hwnd.is_null() {
                DestroyWindow(self.hwnd);
            }
        }
    }
}
