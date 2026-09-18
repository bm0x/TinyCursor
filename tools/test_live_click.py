import subprocess
import time
import ctypes
from ctypes import wintypes

user32 = ctypes.windll.user32

class POINT(ctypes.Structure):
    _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]

class MOUSEINPUT(ctypes.Structure):
    _fields_ = [
        ("dx", wintypes.LONG),
        ("dy", wintypes.LONG),
        ("mouseData", wintypes.DWORD),
        ("dwFlags", wintypes.DWORD),
        ("time", wintypes.DWORD),
        ("dwExtraInfo", ctypes.POINTER(wintypes.ULONG)),
    ]

class INPUT(ctypes.Structure):
    class _INPUT_UNION(ctypes.Union):
        _fields_ = [("mi", MOUSEINPUT)]
    _anonymous_ = ("u",)
    _fields_ = [
        ("type", wintypes.DWORD),
        ("u", _INPUT_UNION),
    ]

INPUT_MOUSE = 0
MOUSEEVENTF_LEFTDOWN = 0x0002
MOUSEEVENTF_LEFTUP = 0x0004
MOUSEEVENTF_MOVE = 0x0001
MOUSEEVENTF_ABSOLUTE = 0x8000

def run_test():
    print("1. Launching target\\release\\tiny-cursor.exe...")
    proc = subprocess.Popen([r"target\release\tiny-cursor.exe"])
    time.sleep(1.5)

    try:
        overlay_hwnd = user32.FindWindowW("TinyCursorOverlay", None)
        print(f"Overlay HWND found: {hex(overlay_hwnd) if overlay_hwnd else 'None'}")

        if not overlay_hwnd:
            print("ERROR: Overlay window not found!")
            return False

        # Check WindowFromPoint at multiple screen positions
        points_to_check = [(100, 100), (500, 500), (960, 540)]
        for x, y in points_to_check:
            pt = POINT(x, y)
            hit_hwnd = user32.WindowFromPoint(pt)
            print(f"WindowFromPoint({x}, {y}) returned HWND: {hex(hit_hwnd)}")
            if hit_hwnd == overlay_hwnd:
                print(f"FAIL: WindowFromPoint intercepted by overlay HWND {hex(overlay_hwnd)}!")
                return False
            else:
                print(f"PASS: Point ({x}, {y}) passed through overlay to {hex(hit_hwnd)}")

        print("\nAll WindowFromPoint checks passed! Overlay is 100% click-through to Windows OS!")
        return True
    finally:
        print("\nTerminating tiny-cursor.exe...")
        proc.terminate()
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.kill()
        # Restore cursors
        SPI_SETCURSORS = 0x0057
        user32.SystemParametersInfoW(SPI_SETCURSORS, 0, None, 0)
        print("Restored system cursors.")

if __name__ == "__main__":
    success = run_test()
    if not success:
        exit(1)
