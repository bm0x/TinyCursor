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

#[cfg(windows)]
#[test]
fn test_band_16_system_tools_click_passthrough() {
    use windows::Win32::Foundation::*;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::System::LibraryLoader::GetProcAddress;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::core::PCWSTR;
    use std::ptr::null_mut;

    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_NCHITTEST => LRESULT(HTTRANSPARENT as _),
            WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as _),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

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
        *mut std::ffi::c_void,
        u32,
    ) -> HWND;

    unsafe {
        let class_name: Vec<u16> = "TestBand16Class\0".encode_utf16().collect();
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

        let user32_name: Vec<u16> = "user32.dll\0".encode_utf16().collect();
        let user32_mod = GetModuleHandleW(PCWSTR(user32_name.as_ptr())).unwrap_or_default();
        if user32_mod.0.is_null() {
            return;
        }

        let p_create = GetProcAddress(user32_mod, windows::core::s!("CreateWindowInBand"));
        if let Some(create_in_band_raw) = p_create {
            let create_in_band: PfnCreateWindowInBand = std::mem::transmute(create_in_band_raw);

            let ex_style = WS_EX_TOPMOST.0
                | WS_EX_TRANSPARENT.0
                | WS_EX_LAYERED.0
                | WS_EX_NOREDIRECTIONBITMAP.0
                | WS_EX_TOOLWINDOW.0
                | WS_EX_NOACTIVATE.0;

            let hwnd = create_in_band(
                ex_style,
                class_name.as_ptr(),
                class_name.as_ptr(),
                WS_POPUP.0,
                0,
                0,
                1920,
                1080,
                HWND(null_mut()),
                HMENU(null_mut()),
                HINSTANCE(null_mut()),
                null_mut(),
                16, // ZBID_SYSTEM_TOOLS
            );

            if !hwnd.0.is_null() {
                let res = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);
                assert!(res.is_ok(), "SetLayeredWindowAttributes failed on Band 16");

                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetWindowPos(
                    hwnd,
                    HWND(null_mut()),
                    0,
                    0,
                    1920,
                    1080,
                    SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );

                let pt = POINT { x: 500, y: 500 };
                let hit_hwnd = WindowFromPoint(pt);
                println!("Band 16 Window -> Hit HWND: {:?}, Overlay HWND: {:?}", hit_hwnd, hwnd);
                assert_ne!(hit_hwnd, hwnd, "Band 16 Overlay intercepted hit-testing! Click-through failed!");

                let _ = DestroyWindow(hwnd);
            }
        }
    }
}

