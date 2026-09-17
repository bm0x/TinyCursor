//! Input querying and hardware mouse tracking for Windows.

use std::ptr::null_mut;
use crate::core::math::Vec2;
use super::renderer::RenderCursorKind;
use super::sys::{
    GetAsyncKeyState, GetClassNameW, GetCursorPos, GetDC, GetDeviceCaps, GetPixel,
    GetWindow, GetWindowLongW, GetWindowRect, ReleaseDC, WindowFromPoint, CLR_INVALID,
    GWL_STYLE, GW_HWNDNEXT, HWND, POINT, RECT, VK_CONTROL, VK_ESCAPE, VK_LBUTTON,
    VK_MENU, VK_SHIFT, VREFRESH, WS_THICKFRAME,
};

/// Queries the real hardware mouse cursor position in virtual screen coordinates.
#[inline]
pub fn get_hardware_cursor_pos() -> Option<Vec2> {
    let mut pt = POINT { x: 0, y: 0 };
    let success = unsafe { GetCursorPos(&mut pt) };
    if success != 0 {
        Some(Vec2::new(pt.x as f32, pt.y as f32))
    } else {
        None
    }
}

/// Checks if the left mouse button is currently held down.
#[inline]
pub fn is_left_button_down() -> bool {
    let state = unsafe { GetAsyncKeyState(VK_LBUTTON) };
    (state as u16 & 0x8000) != 0
}

/// Checks if the emergency fail-safe hotkey (Ctrl + Alt + Shift + Esc) is held down.
/// Requires Alt to never collide with standard Windows Task Manager shortcut (Ctrl + Shift + Esc).
#[inline]
pub fn is_emergency_escape_pressed() -> bool {
    let ctrl = unsafe { GetAsyncKeyState(VK_CONTROL) } < 0;
    let alt = unsafe { GetAsyncKeyState(VK_MENU) } < 0;
    let shift = unsafe { GetAsyncKeyState(VK_SHIFT) } < 0;
    let esc = unsafe { GetAsyncKeyState(VK_ESCAPE) } < 0;
    ctrl && alt && shift && esc
}

/// Samples the screen luminance around the given hardware cursor position.
/// Uses 4 perimeter probe points situated strictly outside the cursor sprite bounds (radius ~32px)
/// to prevent any self-sampling feedback loop with the rendered cursor overlay.
/// Returns perceptual luminance in 0.0 .. 255.0 (0.0 = pure black, 255.0 = pure white).
pub fn sample_screen_luminance(pos: Vec2) -> f32 {
    let x = pos.x as i32;
    let y = pos.y as i32;

    unsafe {
        let hdc = GetDC(null_mut());
        if hdc.is_null() {
            return 128.0;
        }

        // Diamond pattern probe points at 32px radius
        const PROBES: [(i32, i32); 4] = [
            (0, -32),
            (32, 0),
            (0, 32),
            (-32, 0),
        ];

        let mut total_lum = 0.0f32;
        let mut count = 0;

        for &(dx, dy) in &PROBES {
            let px = x + dx;
            let py = y + dy;
            let color = GetPixel(hdc, px, py);
            if color != CLR_INVALID {
                let r = (color & 0xFF) as f32;
                let g = ((color >> 8) & 0xFF) as f32;
                let b = ((color >> 16) & 0xFF) as f32;
                // Standard ITU-R BT.601 perceptual luminance formula
                let lum = 0.299 * r + 0.587 * g + 0.114 * b;
                total_lum += lum;
                count += 1;
            }
        }

        ReleaseDC(null_mut(), hdc);

        if count > 0 {
            total_lum / (count as f32)
        } else {
            128.0
        }
    }
}

/// Queries the refresh rate of the primary display (e.g. 144 Hz, 240 Hz, 360 Hz).
pub fn get_max_display_frequency() -> u32 {
    unsafe {
        let hdc = GetDC(null_mut());
        if hdc.is_null() {
            return 144;
        }
        let hz = GetDeviceCaps(hdc, VREFRESH);
        ReleaseDC(null_mut(), hdc);
        if hz > 0 {
            hz as u32
        } else {
            144
        }
    }
}

/// Detects the active system cursor design using zero-message geometry and class detection.
/// Never sends cross-process window messages (preventing WinUI / Microsoft Store memory recursion & lag).
pub fn detect_system_cursor(
    pos: Vec2,
    is_clicking: bool,
    is_hand_on_click: bool,
    overlay_hwnd: HWND,
) -> RenderCursorKind {
    let pt = POINT {
        x: pos.x as i32,
        y: pos.y as i32,
    };

    unsafe {
        let mut hwnd = WindowFromPoint(pt);
        // If WindowFromPoint hits our own overlay window, skip down the Z-order
        // to discover the real underlying application window (Notepad, Chrome, Store, etc.)
        while !hwnd.is_null() && hwnd == overlay_hwnd {
            hwnd = GetWindow(hwnd, GW_HWNDNEXT);
        }

        if !hwnd.is_null() {
            let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            // Only resizable windows have resize borders
            if (style & WS_THICKFRAME) != 0 {
                let mut rect = RECT::default();
                if GetWindowRect(hwnd, &mut rect) != 0 {
                    const BORDER: i32 = 8;
                    let on_left = (pt.x - rect.left).abs() <= BORDER;
                    let on_right = (rect.right - pt.x).abs() <= BORDER;
                    let on_top = (pt.y - rect.top).abs() <= BORDER;
                    let on_bottom = (rect.bottom - pt.y).abs() <= BORDER;

                    if on_top && on_left { return RenderCursorKind::ResizeNWSE; }
                    if on_top && on_right { return RenderCursorKind::ResizeNESW; }
                    if on_bottom && on_left { return RenderCursorKind::ResizeNESW; }
                    if on_bottom && on_right { return RenderCursorKind::ResizeNWSE; }
                    if on_left || on_right { return RenderCursorKind::ResizeWE; }
                    if on_top || on_bottom { return RenderCursorKind::ResizeNS; }
                }
            }

            let mut class_buf = [0u16; 32];
            let len = GetClassNameW(hwnd, class_buf.as_mut_ptr(), class_buf.len() as i32);
            if len > 0 {
                let class_str = String::from_utf16_lossy(&class_buf[..len as usize]);
                let lower = class_str.to_ascii_lowercase();
                if lower.contains("edit") || lower.contains("textbox") || lower.contains("scintilla") {
                    return RenderCursorKind::IBeam;
                }
            }
        }
    }

    if is_clicking && is_hand_on_click {
        RenderCursorKind::Hand
    } else {
        RenderCursorKind::Arrow
    }
}


