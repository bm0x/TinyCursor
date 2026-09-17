import ctypes
user32 = ctypes.windll.user32

def enum_windows_cb(hwnd, lparam):
    if user32.IsWindowVisible(hwnd):
        title = ctypes.create_unicode_buffer(256)
        cls = ctypes.create_unicode_buffer(256)
        user32.GetWindowTextW(hwnd, title, 256)
        user32.GetClassNameW(hwnd, cls, 256)
        name = title.value.lower() + " " + cls.value.lower()
        if any(x in name for x in ['start', 'tray', 'shell', 'experience', 'notification', 'taskbar']):
            ex_style = user32.GetWindowLongW(hwnd, -20)
            band = 0
            if hasattr(user32, 'GetWindowBand'):
                b = ctypes.c_ulong(0)
                user32.GetWindowBand(hwnd, ctypes.byref(b))
                band = b.value
            print(f'HWND: {hwnd}, Title: "{title.value}", Class: "{cls.value}", Band: {band}, ExStyle: 0x{ex_style:08X}')
    return True

WNDENUMPROC = ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)
user32.EnumWindows(WNDENUMPROC(enum_windows_cb), 0)
