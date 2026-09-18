import ctypes, sys, os
from ctypes import wintypes

k32 = ctypes.windll.kernel32

BeginUpdateResourceW = k32.BeginUpdateResourceW
BeginUpdateResourceW.restype = wintypes.HANDLE
BeginUpdateResourceW.argtypes = [wintypes.LPCWSTR, wintypes.BOOL]

UpdateResourceW = k32.UpdateResourceW
UpdateResourceW.restype = wintypes.BOOL
UpdateResourceW.argtypes = [
    wintypes.HANDLE, wintypes.LPCWSTR, wintypes.LPCWSTR,
    wintypes.WORD, ctypes.c_char_p, wintypes.DWORD
]

EndUpdateResourceW = k32.EndUpdateResourceW
EndUpdateResourceW.restype = wintypes.BOOL
EndUpdateResourceW.argtypes = [wintypes.HANDLE, wintypes.BOOL]

def embed_manifest(exe_path, ui_access=False):
    ui_str = "true" if ui_access else "false"
    manifest_xml = f'''<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="0.1.0.0" processorArchitecture="*" name="TinyCursor" type="win32"/>
  <description>TinyCursor Smooth Cursor</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="{ui_str}"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}}"/>
      <supportedOS Id="{{1f676c76-80e1-4239-95bb-83d0f6d0da78}}"/>
      <supportedOS Id="{{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}}"/>
      <supportedOS Id="{{35138b9a-5d96-4fbd-8e2d-a2440225f93a}}"/>
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2, PerMonitor</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>'''.encode('utf-8')

    h = BeginUpdateResourceW(exe_path, False)
    if not h:
        print(f"Error BeginUpdateResourceW: {ctypes.GetLastError()}")
        return False

    RT_MANIFEST = ctypes.cast(24, wintypes.LPCWSTR)
    MAKEINTRESOURCE_1 = ctypes.cast(1, wintypes.LPCWSTR)

    res = UpdateResourceW(h, RT_MANIFEST, MAKEINTRESOURCE_1, 0, manifest_xml, len(manifest_xml))
    end_res = EndUpdateResourceW(h, False)
    
    if res and end_res:
        print(f"Manifest successfully embedded into {exe_path} (uiAccess={ui_str})")
        return True
    else:
        print(f"Error UpdateResourceW: {ctypes.GetLastError()}")
        return False

if __name__ == '__main__':
    exe = sys.argv[1] if len(sys.argv) > 1 else 'target/release/tiny-cursor.exe'
    embed_manifest(exe, ui_access=True)
