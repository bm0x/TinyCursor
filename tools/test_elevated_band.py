import ctypes
from ctypes import wintypes

user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32

CreateWindowInBand = getattr(user32, 'CreateWindowInBand', None)
if CreateWindowInBand:
    CreateWindowInBand.restype = wintypes.HWND
    CreateWindowInBand.argtypes = [
        wintypes.DWORD, wintypes.LPCWSTR, wintypes.LPCWSTR, wintypes.DWORD,
        ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_int,
        wintypes.HWND, wintypes.HMENU, wintypes.HINSTANCE, wintypes.LPVOID,
        wintypes.DWORD
    ]

WNDPROC = ctypes.WINFUNCTYPE(ctypes.c_long, wintypes.HWND, wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM)
class WNDCLASSEXW(ctypes.Structure):
    _fields_ = [
        ('cbSize', wintypes.UINT), ('style', wintypes.UINT), ('lpfnWndProc', WNDPROC),
        ('cbClsExtra', ctypes.c_int), ('cbWndExtra', ctypes.c_int), ('hInstance', wintypes.HINSTANCE),
        ('hIcon', wintypes.HICON), ('hCursor', wintypes.HCURSOR), ('hbrBackground', wintypes.HBRUSH),
        ('lpszMenuName', wintypes.LPCWSTR), ('lpszClassName', wintypes.LPCWSTR), ('hIconSm', wintypes.HICON)
    ]

def wnd_proc(hwnd, msg, wp, lp):
    return user32.DefWindowProcW(hwnd, msg, wp, lp)

wc = WNDCLASSEXW()
wc.cbSize = ctypes.sizeof(WNDCLASSEXW)
wc.lpfnWndProc = WNDPROC(wnd_proc)
wc.hInstance = kernel32.GetModuleHandleW(None)
wc.lpszClassName = 'ElevBandTestClass'
user32.RegisterClassExW(ctypes.byref(wc))

WS_POPUP = 0x80000000
WS_EX_TOPMOST = 0x00000008
WS_EX_TRANSPARENT = 0x00000020
WS_EX_LAYERED = 0x00080000
WS_EX_TOOLWINDOW = 0x00000080
WS_EX_NOACTIVATE = 0x08000000
ex_style = WS_EX_TOPMOST | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE

res_lines = []
for band in [1, 2, 6, 7, 8, 14]:
    hwnd = CreateWindowInBand(ex_style, 'ElevBandTestClass', 'ElevBandTest', WS_POPUP, 100, 100, 100, 100, None, None, wc.hInstance, None, band)
    err = ctypes.GetLastError()
    res_lines.append(f"Band {band}: hwnd={hwnd}, last_error={err}")
    if hwnd:
        user32.DestroyWindow(hwnd)

with open('band_result.txt', 'w') as f:
    f.write('\n'.join(res_lines))
print('\n'.join(res_lines))
