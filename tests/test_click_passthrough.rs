#[cfg(windows)]
#[test]
fn test_overlay_click_passthrough() {
    use windows::Win32::Foundation::*;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::core::PCWSTR;
    use std::ptr::null_mut;

    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_NCHITTEST => LRESULT(HTTRANSPARENT as _),
            WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as _),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    unsafe {
        let class_name: Vec<u16> = "TestOverlayClass\0".encode_utf16().collect();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: WNDCLASS_STYLES(0),
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: HINSTANCE(null_mut()),
            hIcon: HICON(null_mut()),
            hCursor: HCURSOR(null_mut()),
            hbrBackground: HBRUSH(null_mut()),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hIconSm: HICON(null_mut()),
        };
        let _ = RegisterClassExW(&wc);

        let ex_style = WS_EX_TOPMOST
            | WS_EX_TRANSPARENT
            | WS_EX_LAYERED
            | WS_EX_NOREDIRECTIONBITMAP
            | WS_EX_TOOLWINDOW
            | WS_EX_NOACTIVATE;

        let hwnd_res = CreateWindowExW(
            ex_style,
            PCWSTR(class_name.as_ptr()),
            PCWSTR(class_name.as_ptr()),
            WS_POPUP,
            0,
            0,
            1920,
            1080,
            HWND(null_mut()),
            HMENU(null_mut()),
            HINSTANCE(null_mut()),
            None,
        );

        let hwnd = hwnd_res.expect("CreateWindowExW failed");
        let res = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);
        assert!(res.is_ok(), "SetLayeredWindowAttributes failed");

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            1920,
            1080,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );

        let pt = POINT { x: 500, y: 500 };
        let hit_hwnd = WindowFromPoint(pt);
        println!("With WS_EX_LAYERED -> Hit HWND: {:?}, Overlay HWND: {:?}", hit_hwnd, hwnd);
        assert_ne!(hit_hwnd, hwnd, "Overlay intercepted hit-testing! Click-through failed!");

        let _ = DestroyWindow(hwnd);
    }
}
