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
    WS_EX_TRANSPARENT, WS_POPUP, ZBID_IMMERSIVE_NOTIFICATION, ZBID_SYSTEM_TOOLS,
    ZBID_UIACCESS, GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, HTTRANSPARENT, MA_NOACTIVATE, WM_DESTROY,
    WM_MOUSEACTIVATE, WM_NCHITTEST, WM_SETCURSOR,
};

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

type PfnSetWindowBand = unsafe extern "system" fn(HWND, HWND, u32) -> i32;

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

/// A native Windows transparent overlay window.
pub struct OverlayWindow {
    hwnd: HWND,
}

impl OverlayWindow {
    /// Creates and initializes the transparent overlay window.
    /// Attempts creation in elevated Z-Bands (ZBID_UIACCESS) to render above
    /// the Windows Start Menu, Action Center notifications, and elevated windows.
    pub fn new() -> Option<Self> {
        unsafe {
            // Enable Per-Monitor DPI Awareness v2
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

            let hinstance = GetModuleHandleW(null_mut());

            let mut wc: WNDCLASSEXW = std::mem::zeroed();
            wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.lpfn_wnd_proc = Some(window_proc);
            wc.h_instance = hinstance;
            wc.lpsz_class_name = WINDOW_CLASS_NAME.as_ptr();

            RegisterClassExW(&wc);

            // Extended styles:
            // - WS_EX_TOPMOST: Stays above regular windows
            // - WS_EX_TRANSPARENT: Click-through (hit test passes through to window underneath)
            // - WS_EX_LAYERED: Enables per-pixel 32-bit ARGB alpha composition
            // - WS_EX_TOOLWINDOW: Hidden from taskbar and Alt+Tab menu
            // - WS_EX_NOACTIVATE: Does not steal focus when clicked or updated
            let ex_style = WS_EX_TOPMOST
                | WS_EX_TRANSPARENT
                | WS_EX_LAYERED
                | WS_EX_TOOLWINDOW
                | WS_EX_NOACTIVATE;

            // Dynamically query CreateWindowInBand and SetWindowBand from user32.dll
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

            // 1. First priority: Try elevated Z-Bands (ZBID_UIACCESS = 12, ZBID_SYSTEM_TOOLS = 11)
            // Windows allows this when the executable has UIAccess privilege.
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
                        break;
                    }
                }
            }

            // 2. Second priority: Standard CreateWindowExW (ZBID_TOPMOST = 2) fallback
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

            // 3. Reinforce band placement if SetWindowBand is available
            if let Some(set_band) = p_set_window_band {
                let _ = set_band(hwnd, null_mut(), ZBID_UIACCESS);
            }

            ShowWindow(hwnd, SW_SHOW);

            // Force topmost positioning: with UIAccess=true, this places the window above immersive shell bands
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );

            Some(Self { hwnd })
        }
    }

    /// Creates and initializes a transparent full-desktop overlay window
    /// designed specifically for DirectComposition and DXGI Modern Flip Model.
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

            // Extended styles for DirectComposition:
            // WS_EX_NOREDIRECTIONBITMAP tells DWM not to create a GDI bitmap,
            // delegating composition directly to the DirectX SwapChain.
            // Hit-testing is handled cleanly via WM_NCHITTEST -> HTTRANSPARENT.
            let ex_style = WS_EX_TOPMOST
                | WS_EX_TRANSPARENT
                | WS_EX_NOREDIRECTIONBITMAP
                | WS_EX_TOOLWINDOW
                | WS_EX_NOACTIVATE;

            let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let vw = (GetSystemMetrics(SM_CXVIRTUALSCREEN) as u32).max(1920);
            let vh = (GetSystemMetrics(SM_CYVIRTUALSCREEN) as u32).max(1080);

            // Dynamically query CreateWindowInBand and SetWindowBand from user32.dll
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

            // 1. Try elevated Z-Bands (ZBID_UIACCESS = 12, ZBID_SYSTEM_TOOLS = 11)
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
                        break;
                    }
                }
            }

            // 2. Standard CreateWindowExW fallback
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

            // 3. Reinforce band placement if SetWindowBand is available
            if let Some(set_band) = p_set_window_band {
                let _ = set_band(hwnd, null_mut(), ZBID_UIACCESS);
            }

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

            Some((Self { hwnd }, vx, vy, vw, vh))
        }
    }

    /// Returns the Win32 window handle.
    #[inline]
    pub fn hwnd(&self) -> HWND {
        self.hwnd
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
