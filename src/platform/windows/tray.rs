//! Windows System Tray (Taskbar Notification Area) Integration.
//!
//! Provides a background tray icon with a right-click context menu to pause/resume,
//! toggle click behaviors, and exit cleanly without needing a console window.

use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use super::assets::ICON_ICO_BYTES;
use super::sys::{
    AppendMenuW, CreateIconFromResourceEx, CreatePopupMenu, CreateWindowExW,
    DefWindowProcW, DestroyIcon, DestroyMenu, DestroyWindow, GetCursorPos,
    GetModuleHandleW, PostQuitMessage, RegisterClassExW, SetForegroundWindow,
    Shell_NotifyIconW, TrackPopupMenu, HICON, HMENU, HWND, LPARAM, LRESULT,
    MF_CHECKED, MF_DISABLED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED,
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
    POINT, TPM_BOTTOMALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_APP,
    WM_COMMAND, WM_LBUTTONUP, WM_RBUTTONUP, WNDCLASSEXW, WPARAM,
};

pub const WM_TRAY_CALLBACK: u32 = WM_APP + 101;

const IDM_TITLE: usize = 1000;
const IDM_PAUSE: usize = 1001;
const IDM_CLICK_HAND: usize = 1002;
const IDM_THEME_AUTO: usize = 1003;
const IDM_THEME_WHITE: usize = 1004;
const IDM_THEME_BLACK: usize = 1005;
const IDM_EXIT: usize = 1006;

const IDM_DESIGN_AUTO: usize = 1010;
const IDM_DESIGN_ZOOM_IN: usize = 1011;
const IDM_DESIGN_ZOOM_OUT: usize = 1012;
const IDM_DESIGN_RESIZE_NS: usize = 1013;
const IDM_DESIGN_RESIZE_WE: usize = 1014;
const IDM_DESIGN_MOVE: usize = 1015;
const IDM_DESIGN_IBEAM: usize = 1016;
const IDM_DESIGN_CROSSHAIR: usize = 1017;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Auto = 0,
    AlwaysWhite = 1,
    AlwaysBlack = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorDesignOverride {
    Auto = 0,
    Arrow = 1,
    Hand = 2,
    IBeam = 3,
    ResizeNS = 4,
    ResizeWE = 5,
    Move = 6,
    ZoomIn = 7,
    ZoomOut = 8,
    Crosshair = 9,
}

static IS_PAUSED: AtomicBool = AtomicBool::new(false);
static HAND_ON_CLICK: AtomicBool = AtomicBool::new(true);
static THEME_MODE: AtomicU32 = AtomicU32::new(0); // 0 = Auto, 1 = AlwaysWhite, 2 = AlwaysBlack
static DESIGN_OVERRIDE: AtomicU32 = AtomicU32::new(0); // 0 = Auto
static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);

const TRAY_WINDOW_CLASS: &[u16] = &[
    b'T' as u16, b'i' as u16, b'n' as u16, b'y' as u16,
    b'C' as u16, b'u' as u16, b'r' as u16, b's' as u16,
    b'o' as u16, b'r' as u16, b'T' as u16, b'r' as u16,
    b'a' as u16, b'y' as u16, 0,
];

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_TRAY_CALLBACK {
        let event = lparam as u32;
        if event == WM_RBUTTONUP || event == WM_LBUTTONUP {
            let mut pt = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            SetForegroundWindow(hwnd);

            let menu = CreatePopupMenu();
            let to_wide = |s: &str| -> Vec<u16> {
                s.encode_utf16().chain(std::iter::once(0)).collect()
            };

            let title_str = to_wide("TinyCursor v0.1.0");
            AppendMenuW(menu, MF_STRING | MF_DISABLED, IDM_TITLE, title_str.as_ptr());
            AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

            let is_paused = IS_PAUSED.load(Ordering::Relaxed);
            let pause_text = to_wide(if is_paused { "▶ Reanudar Smooth Cursor" } else { "⏸ Pausar Smooth Cursor" });
            let pause_flags = MF_STRING | if is_paused { MF_CHECKED } else { MF_UNCHECKED };
            AppendMenuW(menu, pause_flags, IDM_PAUSE, pause_text.as_ptr());

            let hand_active = HAND_ON_CLICK.load(Ordering::Relaxed);
            let hand_text = to_wide("👆 Mano en Clic");
            let hand_flags = MF_STRING | if hand_active { MF_CHECKED } else { MF_UNCHECKED };
            AppendMenuW(menu, hand_flags, IDM_CLICK_HAND, hand_text.as_ptr());

            AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

            let current_theme = THEME_MODE.load(Ordering::Relaxed);

            let auto_text = to_wide("🌓 Tema: Automático (Adaptativo)");
            let auto_flags = MF_STRING | if current_theme == 0 { MF_CHECKED } else { MF_UNCHECKED };
            AppendMenuW(menu, auto_flags, IDM_THEME_AUTO, auto_text.as_ptr());

            let white_text = to_wide("⚪ Tema: Siempre Blanco");
            let white_flags = MF_STRING | if current_theme == 1 { MF_CHECKED } else { MF_UNCHECKED };
            AppendMenuW(menu, white_flags, IDM_THEME_WHITE, white_text.as_ptr());

            let black_text = to_wide("⚫ Tema: Siempre Negro");
            let black_flags = MF_STRING | if current_theme == 2 { MF_CHECKED } else { MF_UNCHECKED };
            AppendMenuW(menu, black_flags, IDM_THEME_BLACK, black_text.as_ptr());

            AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());

            let current_design = DESIGN_OVERRIDE.load(Ordering::Relaxed);
            let d_auto = to_wide("🎯 Modo Cursor: Automático (Detectar OS)");
            AppendMenuW(menu, MF_STRING | if current_design == 0 { MF_CHECKED } else { MF_UNCHECKED }, IDM_DESIGN_AUTO, d_auto.as_ptr());

            let d_zoom_in = to_wide("🔍 Vista: Lupa Zoom +");
            AppendMenuW(menu, MF_STRING | if current_design == 7 { MF_CHECKED } else { MF_UNCHECKED }, IDM_DESIGN_ZOOM_IN, d_zoom_in.as_ptr());

            let d_zoom_out = to_wide("🔎 Vista: Lupa Zoom -");
            AppendMenuW(menu, MF_STRING | if current_design == 8 { MF_CHECKED } else { MF_UNCHECKED }, IDM_DESIGN_ZOOM_OUT, d_zoom_out.as_ptr());

            let d_resize = to_wide("↕ Vista: Redimensionar");
            AppendMenuW(menu, MF_STRING | if current_design == 4 { MF_CHECKED } else { MF_UNCHECKED }, IDM_DESIGN_RESIZE_NS, d_resize.as_ptr());

            let d_move = to_wide("✥ Vista: Mover");
            AppendMenuW(menu, MF_STRING | if current_design == 6 { MF_CHECKED } else { MF_UNCHECKED }, IDM_DESIGN_MOVE, d_move.as_ptr());

            let d_ibeam = to_wide("I Vista: Texto (I-Beam)");
            AppendMenuW(menu, MF_STRING | if current_design == 3 { MF_CHECKED } else { MF_UNCHECKED }, IDM_DESIGN_IBEAM, d_ibeam.as_ptr());

            let d_cross = to_wide("➕ Vista: Cruz (Precision)");
            AppendMenuW(menu, MF_STRING | if current_design == 9 { MF_CHECKED } else { MF_UNCHECKED }, IDM_DESIGN_CROSSHAIR, d_cross.as_ptr());

            AppendMenuW(menu, MF_SEPARATOR, 0, null_mut());
            let exit_text = to_wide("✕ Salir y Restaurar Cursor");
            AppendMenuW(menu, MF_STRING, IDM_EXIT, exit_text.as_ptr());

            let cmd = TrackPopupMenu(
                menu,
                TPM_BOTTOMALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD,
                pt.x,
                pt.y,
                0,
                hwnd,
                null_mut(),
            );

            match cmd as usize {
                IDM_PAUSE => {
                    IS_PAUSED.fetch_xor(true, Ordering::SeqCst);
                }
                IDM_CLICK_HAND => {
                    HAND_ON_CLICK.fetch_xor(true, Ordering::SeqCst);
                }
                IDM_THEME_AUTO => {
                    THEME_MODE.store(0, Ordering::SeqCst);
                }
                IDM_THEME_WHITE => {
                    THEME_MODE.store(1, Ordering::SeqCst);
                }
                IDM_THEME_BLACK => {
                    THEME_MODE.store(2, Ordering::SeqCst);
                }
                IDM_DESIGN_AUTO => {
                    DESIGN_OVERRIDE.store(0, Ordering::SeqCst);
                }
                IDM_DESIGN_ZOOM_IN => {
                    DESIGN_OVERRIDE.store(7, Ordering::SeqCst);
                }
                IDM_DESIGN_ZOOM_OUT => {
                    DESIGN_OVERRIDE.store(8, Ordering::SeqCst);
                }
                IDM_DESIGN_RESIZE_NS => {
                    DESIGN_OVERRIDE.store(4, Ordering::SeqCst);
                }
                IDM_DESIGN_RESIZE_WE => {
                    DESIGN_OVERRIDE.store(5, Ordering::SeqCst);
                }
                IDM_DESIGN_MOVE => {
                    DESIGN_OVERRIDE.store(6, Ordering::SeqCst);
                }
                IDM_DESIGN_IBEAM => {
                    DESIGN_OVERRIDE.store(3, Ordering::SeqCst);
                }
                IDM_DESIGN_CROSSHAIR => {
                    DESIGN_OVERRIDE.store(9, Ordering::SeqCst);
                }
                IDM_EXIT => {
                    SHOULD_EXIT.store(true, Ordering::SeqCst);
                    PostQuitMessage(0);
                }
                _ => {}
            }

            DestroyMenu(menu);
            return 0;
        }
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

/// System Tray controller.
pub struct TrayManager {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
    hicon: HICON,
}

impl TrayManager {
    /// Creates and registers the system tray icon in the Windows taskbar.
    pub fn new() -> Option<Self> {
        unsafe {
            let hinstance = GetModuleHandleW(null_mut());

            let mut wc: WNDCLASSEXW = std::mem::zeroed();
            wc.cb_size = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.lpfn_wnd_proc = Some(tray_wnd_proc);
            wc.h_instance = hinstance;
            wc.lpsz_class_name = TRAY_WINDOW_CLASS.as_ptr();

            RegisterClassExW(&wc);

            let hwnd = CreateWindowExW(
                0,
                TRAY_WINDOW_CLASS.as_ptr(),
                TRAY_WINDOW_CLASS.as_ptr(),
                0,
                0, 0, 0, 0,
                null_mut(),
                null_mut(),
                hinstance,
                null_mut(),
            );

            if hwnd.is_null() {
                return None;
            }

            // Create HICON from embedded icon bytes
            let hicon = CreateIconFromResourceEx(
                ICON_ICO_BYTES.as_ptr(),
                ICON_ICO_BYTES.len() as u32,
                1, // TRUE = Icon
                0x00030000,
                32, 32,
                0,
            );

            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cb_size = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.h_wnd = hwnd;
            nid.u_id = 1;
            nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            nid.u_callback_message = WM_TRAY_CALLBACK;
            nid.h_icon = hicon;

            let tip = "TinyCursor - Smooth Cursor\0".encode_utf16().collect::<Vec<u16>>();
            for (i, &ch) in tip.iter().take(nid.sz_tip.len()).enumerate() {
                nid.sz_tip[i] = ch;
            }

            Shell_NotifyIconW(NIM_ADD, &nid);

            Some(Self { hwnd, nid, hicon })
        }
    }

    /// Checks if the user requested a clean shutdown from the tray menu.
    #[inline]
    pub fn should_exit(&self) -> bool {
        SHOULD_EXIT.load(Ordering::Relaxed)
    }

    /// Checks if smooth cursor rendering is currently paused from the tray.
    #[inline]
    pub fn is_paused(&self) -> bool {
        IS_PAUSED.load(Ordering::Relaxed)
    }

    /// Checks if the Modern White Hand should be displayed during click.
    #[inline]
    pub fn is_hand_on_click_enabled(&self) -> bool {
        HAND_ON_CLICK.load(Ordering::Relaxed)
    }

    /// Queries the user's selected theme mode (Auto, AlwaysWhite, AlwaysBlack).
    #[inline]
    pub fn theme_mode(&self) -> ThemeMode {
        match THEME_MODE.load(Ordering::Relaxed) {
            1 => ThemeMode::AlwaysWhite,
            2 => ThemeMode::AlwaysBlack,
            _ => ThemeMode::Auto,
        }
    }

    /// Queries the user's selected cursor design override.
    #[inline]
    pub fn design_override(&self) -> CursorDesignOverride {
        match DESIGN_OVERRIDE.load(Ordering::Relaxed) {
            1 => CursorDesignOverride::Arrow,
            2 => CursorDesignOverride::Hand,
            3 => CursorDesignOverride::IBeam,
            4 => CursorDesignOverride::ResizeNS,
            5 => CursorDesignOverride::ResizeWE,
            6 => CursorDesignOverride::Move,
            7 => CursorDesignOverride::ZoomIn,
            8 => CursorDesignOverride::ZoomOut,
            9 => CursorDesignOverride::Crosshair,
            _ => CursorDesignOverride::Auto,
        }
    }
}

impl Drop for TrayManager {
    fn drop(&mut self) {
        unsafe {
            Shell_NotifyIconW(NIM_DELETE, &self.nid);
            if !self.hicon.is_null() {
                DestroyIcon(self.hicon);
            }
            if !self.hwnd.is_null() {
                DestroyWindow(self.hwnd);
            }
        }
    }
}
