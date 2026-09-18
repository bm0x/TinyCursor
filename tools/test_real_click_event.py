import subprocess
import time
import threading
import ctypes
from ctypes import wintypes
import tkinter as tk

user32 = ctypes.windll.user32

class POINT(ctypes.Structure):
    _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]

# Mouse input flags
MOUSEEVENTF_LEFTDOWN = 0x0002
MOUSEEVENTF_LEFTUP = 0x0004

def test_real_click():
    print("1. Spawning tiny-cursor.exe...")
    proc = subprocess.Popen([r"target\release\tiny-cursor.exe"])
    time.sleep(1.2)

    click_registered = False

    def on_button_click():
        nonlocal click_registered
        click_registered = True
        print(">>> SUCCESS: Real Button clicked! Click event received by underlying UI app!")
        root.destroy()

    root = tk.Tk()
    root.title("Click Passthrough Verification")
    root.geometry("300x200+400+300")
    root.attributes("-topmost", True)

    btn = tk.Button(root, text="Click Me Test", font=("Segoe UI", 14), command=on_button_click)
    btn.pack(expand=True, fill="both", padx=20, pady=20)

    def simulate_click():
        time.sleep(0.8)
        # Button center in screen coordinates
        # Window is at 400, 300 with size 300x200 -> center is around 550, 400
        cx = 550
        cy = 400
        user32.SetCursorPos(cx, cy)
        time.sleep(0.1)
        user32.mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0)
        time.sleep(0.05)
        user32.mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0)
        time.sleep(0.5)
        if not click_registered:
            print("FAILED: Button was NOT clicked!")
            root.destroy()

    threading.Thread(target=simulate_click, daemon=True).start()

    try:
        root.mainloop()
    finally:
        print("Terminating tiny-cursor.exe...")
        proc.terminate()
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.kill()
        # Restore system cursors
        user32.SystemParametersInfoW(0x0057, 0, None, 0)

    if click_registered:
        print("\n*** VERIFIED 100%: MOUSE CLICKS PASS THROUGH OVERLAY AND INTERACT WITH WINDOWS APPS! ***\n")
        return True
    else:
        return False

if __name__ == "__main__":
    ok = test_real_click()
    if not ok:
        exit(1)
