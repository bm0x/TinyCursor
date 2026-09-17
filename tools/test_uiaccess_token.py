import ctypes
from ctypes import wintypes

advapi32 = ctypes.windll.advapi32
kernel32 = ctypes.windll.kernel32
user32 = ctypes.windll.user32

TOKEN_DUPLICATE = 0x0002
TOKEN_QUERY = 0x0008
TOKEN_ADJUST_DEFAULT = 0x0080
TOKEN_ASSIGN_PRIMARY = 0x0001
TokenUIAccess = 26

OpenProcessToken = advapi32.OpenProcessToken
DuplicateTokenEx = advapi32.DuplicateTokenEx
GetCurrentProcess = kernel32.GetCurrentProcess
hToken = wintypes.HANDLE()
OpenProcessToken.argtypes = [wintypes.HANDLE, wintypes.DWORD, ctypes.POINTER(wintypes.HANDLE)]
OpenProcessToken.restype = wintypes.BOOL
DuplicateTokenEx.argtypes = [wintypes.HANDLE, wintypes.DWORD, ctypes.c_void_p, ctypes.c_int, ctypes.c_int, ctypes.POINTER(wintypes.HANDLE)]
DuplicateTokenEx.restype = wintypes.BOOL

res = OpenProcessToken(GetCurrentProcess(), 0xF00FF, ctypes.byref(hToken))
print(f"OpenProcessToken: {res}, hToken: {hToken.value}")

if res:
    hNewToken = wintypes.HANDLE()
    DuplicateTokenEx = advapi32.DuplicateTokenEx
    res_dup = DuplicateTokenEx(hToken, 0x02000000 | 0x000F0000 | 0x00100000, None, 2, 1, ctypes.byref(hNewToken))
    print(f"DuplicateTokenEx: {res_dup}, hNewToken: {hNewToken.value}")
    
    if res_dup:
        bUIAccess = wintypes.DWORD(1)
        SetTokenInformation = advapi32.SetTokenInformation
        res_set = SetTokenInformation(hNewToken, TokenUIAccess, ctypes.byref(bUIAccess), ctypes.sizeof(bUIAccess))
        err = ctypes.GetLastError()
        print(f"SetTokenInformation(TokenUIAccess): {res_set}, LastError: {err}")
