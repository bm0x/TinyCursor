//! Pure Win32 FFI declarations, types, and constants.
//! Links directly to user32, gdi32, kernel32, shell32, and winmm without any external crates.

#![allow(dead_code, non_snake_case, non_camel_case_types)]

use std::ffi::c_void;

pub type HWND = *mut c_void;
pub type HDC = *mut c_void;
pub type HBITMAP = *mut c_void;
pub type HGDIOBJ = *mut c_void;
pub type HCURSOR = *mut c_void;
pub type HICON = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HBRUSH = *mut c_void;
pub type HMENU = *mut c_void;
pub type BOOL = i32;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct SIZE {
    pub cx: i32,
    pub cy: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BLENDFUNCTION {
    pub blend_op: u8,
    pub blend_flags: u8,
    pub source_constant_alpha: u8,
    pub alpha_format: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BITMAPINFOHEADER {
    pub bi_size: u32,
    pub bi_width: i32,
    pub bi_height: i32,
    pub bi_planes: u16,
    pub bi_bit_count: u16,
    pub bi_compression: u32,
    pub bi_size_image: u32,
    pub bi_x_pels_per_meter: i32,
    pub bi_y_pels_per_meter: i32,
    pub bi_clr_used: u32,
    pub bi_clr_important: u32,
}

#[repr(C)]
pub struct BITMAPINFO {
    pub bmi_header: BITMAPINFOHEADER,
    pub bmi_colors: [u32; 1],
}

pub type WNDPROC = Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>;

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cb_size: u32,
    pub style: u32,
    pub lpfn_wnd_proc: WNDPROC,
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: HINSTANCE,
    pub h_icon: HICON,
    pub h_cursor: HCURSOR,
    pub hbr_background: HBRUSH,
    pub lpsz_menu_name: *const u16,
    pub lpsz_class_name: *const u16,
    pub h_icon_sm: HICON,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub w_param: WPARAM,
    pub l_param: LPARAM,
    pub time: u32,
    pub pt: POINT,
}

#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cb_size: u32,
    pub h_wnd: HWND,
    pub u_id: u32,
    pub u_flags: u32,
    pub u_callback_message: u32,
    pub h_icon: HICON,
    pub sz_tip: [u16; 128],
    pub dw_state: u32,
    pub dw_state_mask: u32,
    pub sz_info: [u16; 256],
    pub u_timeout_or_version: u32,
    pub sz_info_title: [u16; 64],
    pub dw_info_flags: u32,
    pub guid_item: [u8; 16],
    pub h_balloon_icon: HICON,
}

// Window Styles & Extended Styles
pub const WS_POPUP: u32 = 0x80000000;
pub const WS_EX_TOPMOST: u32 = 0x00000008;
pub const WS_EX_TRANSPARENT: u32 = 0x00000020;
pub const WS_EX_LAYERED: u32 = 0x00080000;
pub const WS_EX_TOOLWINDOW: u32 = 0x00000080;
pub const WS_EX_NOACTIVATE: u32 = 0x08000000;
pub const GWL_STYLE: i32 = -16;
pub const WS_THICKFRAME: u32 = 0x00040000;

// SetWindowPos Constants
pub const HWND_TOPMOST: HWND = -1isize as *mut c_void;
pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SWP_SHOWWINDOW: u32 = 0x0040;
pub const SWP_NOOWNERZORDER: u32 = 0x0200;

// ShowWindow & Message Loop
pub const SW_HIDE: i32 = 0;
pub const SW_SHOW: i32 = 5;
pub const PM_REMOVE: u32 = 0x0001;
pub const WM_QUIT: u32 = 0x0012;
pub const WM_APP: u32 = 0x8000;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_RBUTTONUP: u32 = 0x0205;
pub const WM_LBUTTONUP: u32 = 0x0202;

// Layered Window & GDI Constants
pub const ULW_ALPHA: u32 = 0x00000002;
pub const AC_SRC_OVER: u8 = 0x00;
pub const AC_SRC_ALPHA: u8 = 0x01;
pub const BI_RGB: u32 = 0;
pub const DIB_RGB_COLORS: u32 = 0;

// Virtual Keys
pub const VK_LBUTTON: i32 = 0x01;
pub const VK_SHIFT: i32 = 0x10;
pub const VK_CONTROL: i32 = 0x11;
pub const VK_MENU: i32 = 0x12;
pub const VK_ESCAPE: i32 = 0x1B;

// System Cursor Parameters
pub const SPI_SETCURSORS: u32 = 0x0057;
pub const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: isize = -4;

// Shell Tray Constants
pub const NIM_ADD: u32 = 0x00000000;
pub const NIM_MODIFY: u32 = 0x00000001;
pub const NIM_DELETE: u32 = 0x00000002;
pub const NIF_MESSAGE: u32 = 0x00000001;
pub const NIF_ICON: u32 = 0x00000002;
pub const NIF_TIP: u32 = 0x00000004;

// Menu Constants
pub const MF_STRING: u32 = 0x00000000;
pub const MF_SEPARATOR: u32 = 0x00000800;
pub const MF_DISABLED: u32 = 0x00000002;
pub const MF_CHECKED: u32 = 0x00000008;
pub const MF_UNCHECKED: u32 = 0x00000000;
pub const TPM_BOTTOMALIGN: u32 = 0x0020;
pub const TPM_RIGHTBUTTON: u32 = 0x0002;
pub const TPM_RETURNCMD: u32 = 0x0100;

// Windows DWM Z-Order Band Identifiers
pub const ZBID_DEFAULT: u32 = 0;
pub const ZBID_DESKTOP: u32 = 1;
pub const ZBID_UIACCESS: u32 = 2;
pub const ZBID_IMMERSIVE_IHM: u32 = 3;
pub const ZBID_IMMERSIVE_NOTIFICATION: u32 = 4;
pub const ZBID_IMMERSIVE_APP: u32 = 8;
pub const ZBID_IMMERSIVE_MOGO: u32 = 14;
pub const ZBID_IMMERSIVE_EDGY: u32 = 15;
pub const ZBID_SYSTEM_TOOLS: u32 = 16;

#[link(name = "user32")]
extern "system" {
    pub fn GetDC(h_wnd: HWND) -> HDC;
    pub fn ReleaseDC(h_wnd: HWND, h_dc: HDC) -> i32;
    pub fn GetCursorPos(lp_point: *mut POINT) -> BOOL;
    pub fn WindowFromPoint(point: POINT) -> HWND;
    pub fn SendMessageTimeoutW(
        h_wnd: HWND,
        msg: u32,
        w_param: WPARAM,
        l_param: LPARAM,
        fu_flags: u32,
        u_timeout: u32,
        lpdw_result: *mut usize,
    ) -> LRESULT;
    pub fn GetClassNameW(h_wnd: HWND, lp_class_name: *mut u16, n_max_count: i32) -> i32;
    pub fn GetWindowRect(h_wnd: HWND, lp_rect: *mut RECT) -> BOOL;
    pub fn GetWindowLongW(h_wnd: HWND, n_index: i32) -> i32;
    pub fn GetCursorInfo(pci: *mut CURSORINFO) -> BOOL;
    pub fn GetAsyncKeyState(v_key: i32) -> i16;
    pub fn CreateWindowExW(
        dw_ex_style: u32,
        lp_class_name: *const u16,
        lp_window_name: *const u16,
        dw_style: u32,
        x: i32,
        y: i32,
        n_width: i32,
        n_height: i32,
        h_wnd_parent: HWND,
        h_menu: *mut c_void,
        h_instance: HINSTANCE,
        lp_param: *mut c_void,
    ) -> HWND;
    pub fn DestroyWindow(h_wnd: HWND) -> BOOL;
    pub fn ShowWindow(h_wnd: HWND, n_cmd_show: i32) -> BOOL;
    pub fn DefWindowProcW(h_wnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
    pub fn RegisterClassExW(unnamed1: *const WNDCLASSEXW) -> u16;
    pub fn PeekMessageW(
        lp_msg: *mut MSG,
        h_wnd: HWND,
        w_msg_filter_min: u32,
        w_msg_filter_max: u32,
        w_remove_msg: u32,
    ) -> BOOL;
    pub fn TranslateMessage(lp_msg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lp_msg: *const MSG) -> LRESULT;
    pub fn UpdateLayeredWindow(
        h_wnd: HWND,
        hdc_dst: HDC,
        ppt_dst: *const POINT,
        psize: *const SIZE,
        hdc_src: HDC,
        ppt_src: *const POINT,
        cr_key: u32,
        pblend: *const BLENDFUNCTION,
        dw_flags: u32,
    ) -> BOOL;
    pub fn SetWindowPos(
        h_wnd: HWND,
        h_wnd_insert_after: HWND,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        u_flags: u32,
    ) -> BOOL;
    pub fn CreateCursor(
        h_inst: HINSTANCE,
        x_hot_spot: i32,
        y_hot_spot: i32,
        n_width: i32,
        n_height: i32,
        pv_and_plane: *const u8,
        pv_xor_plane: *const u8,
    ) -> HCURSOR;
    pub fn DestroyCursor(h_cursor: HCURSOR) -> BOOL;
    pub fn CopyIcon(h_icon: HICON) -> HICON;
    pub fn SetSystemCursor(hcur: HCURSOR, id: u32) -> BOOL;
    pub fn SystemParametersInfoW(
        ui_action: u32,
        ui_param: u32,
        pv_param: *mut c_void,
        f_win_ini: u32,
    ) -> BOOL;
    pub fn SetProcessDpiAwarenessContext(value: isize) -> BOOL;
    pub fn CreatePopupMenu() -> HMENU;
    pub fn DestroyMenu(h_menu: HMENU) -> BOOL;
    pub fn AppendMenuW(h_menu: HMENU, u_flags: u32, u_id_new_item: usize, lp_new_item: *const u16) -> BOOL;
    pub fn TrackPopupMenu(
        h_menu: HMENU,
        u_flags: u32,
        x: i32,
        y: i32,
        n_reserved: i32,
        h_wnd: HWND,
        prc_rect: *const RECT,
    ) -> BOOL;
    pub fn SetForegroundWindow(h_wnd: HWND) -> BOOL;
    pub fn CreateIconFromResourceEx(
        pb_icon_bits: *const u8,
        cb_icon_bits: u32,
        f_icon: BOOL,
        dw_version: u32,
        cx_desired: i32,
        cy_desired: i32,
        u_flags: u32,
    ) -> HICON;
    pub fn DestroyIcon(h_icon: HICON) -> BOOL;
    pub fn PostQuitMessage(n_exit_code: i32);
}

#[link(name = "shell32")]
extern "system" {
    pub fn Shell_NotifyIconW(dw_message: u32, lp_data: *const NOTIFYICONDATAW) -> BOOL;
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GetModuleHandleW(lp_module_name: *const u16) -> HINSTANCE;
    pub fn GetProcAddress(h_module: HINSTANCE, lp_proc_name: *const u8) -> *mut c_void;
    pub fn LoadLibraryW(lp_lib_file_name: *const u16) -> HINSTANCE;
    pub fn FreeLibrary(h_lib_module: HINSTANCE) -> BOOL;
}

#[link(name = "gdi32")]
extern "system" {
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn DeleteDC(hdc: HDC) -> BOOL;
    pub fn CreateDIBSection(
        hdc: HDC,
        pbmi: *const BITMAPINFO,
        usage: u32,
        ppv_bits: *mut *mut c_void,
        h_section: *mut c_void,
        offset: u32,
    ) -> HBITMAP;
    pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    pub fn GetPixel(hdc: HDC, x: i32, y: i32) -> u32;
    pub fn GetDeviceCaps(hdc: HDC, index: i32) -> i32;
}

pub const CLR_INVALID: u32 = 0xFFFFFFFF;
pub const VREFRESH: i32 = 116;
pub const WM_NCHITTEST: u32 = 0x0084;
pub const SMTO_ABORTIFHUNG: u32 = 0x0002;
pub const CURSOR_SHOWING: u32 = 0x00000001;
pub const CURSOR_SUPPRESSED: u32 = 0x00000002;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CURSORINFO {
    pub cb_size: u32,
    pub flags: u32,
    pub h_cursor: HCURSOR,
    pub pt_screen_pos: POINT,
}

#[link(name = "winmm")]
extern "system" {
    pub fn timeBeginPeriod(u_period: u32) -> u32;
    pub fn timeEndPeriod(u_period: u32) -> u32;
}
