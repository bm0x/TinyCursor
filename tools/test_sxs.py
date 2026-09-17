import ctypes
from ctypes import wintypes

class ACTCTXW(ctypes.Structure):
    _fields_ = [
        ("cbSize", wintypes.ULONG),
        ("dwFlags", wintypes.DWORD),
        ("lpSource", wintypes.LPCWSTR),
        ("wProcessorArchitecture", wintypes.USHORT),
        ("wLangId", wintypes.WORD),
        ("lpAssemblyDirectory", wintypes.LPCWSTR),
        ("lpResourceName", wintypes.LPCWSTR),
        ("lpApplicationName", wintypes.LPCWSTR),
        ("hModule", wintypes.HMODULE),
    ]

kernel32 = ctypes.WinDLL('kernel32', use_last_error=True)
CreateActCtxW = kernel32.CreateActCtxW
CreateActCtxW.argtypes = [ctypes.POINTER(ACTCTXW)]
CreateActCtxW.restype = wintypes.HANDLE

ReleaseActCtx = kernel32.ReleaseActCtx
ReleaseActCtx.argtypes = [wintypes.HANDLE]
ReleaseActCtx.restype = None

def test_file(path, res_id=1):
    ctx = ACTCTXW()
    ctx.cbSize = ctypes.sizeof(ACTCTXW)
    ctx.dwFlags = 0x008  # ACTCTX_FLAG_RESOURCE_NAME_VALID
    ctx.lpSource = path
    ctx.lpResourceName = ctypes.cast(res_id, wintypes.LPCWSTR)
    
    handle = CreateActCtxW(ctypes.byref(ctx))
    if handle == wintypes.HANDLE(-1).value or handle == -1 or handle == 0:
        err = ctypes.get_last_error()
        print(f"[-] {path}: FAILED with Win32 Error {err}")
    else:
        ReleaseActCtx(handle)
        print(f"[+] {path}: SUCCESS! Activation Context (SxS) parsed perfectly!")

test_file(r"C:\Users\rafy2\Documents\antigravity\TinyCursor\target\release\tiny-cursor.exe")
test_file(r"C:\Program Files\TinyCursor\tiny-cursor.exe")
