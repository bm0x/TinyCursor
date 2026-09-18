import struct

manifest_xml = b'''<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="0.1.0.0" processorArchitecture="*" name="TinyCursor" type="win32"/>
  <description>TinyCursor Smooth Cursor</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>
      <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}"/>
      <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}"/>
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2, PerMonitor</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>'''

# Make .res file:
# 1. Null header (32 bytes)
# DataSize = 0, HeaderSize = 32, Type = 0xFFFF 0x0000, Name = 0xFFFF 0x0000, DataVersion = 0, MemoryFlags = 0, LanguageId = 0, Version = 0, Characteristics = 0
null_header = struct.pack('<IIHHHHIHHII', 0, 32, 0xFFFF, 0, 0xFFFF, 0, 0, 0, 0, 0, 0)

# 2. RT_MANIFEST entry
data_size = len(manifest_xml)
# Type: 0xFFFF, 24 (RT_MANIFEST)
# Name: 0xFFFF, 1 (CREATEPROCESS_MANIFEST_RESOURCE_ID)
# HeaderSize: 32 bytes
res_header = struct.pack('<IIHHHHIHHII', data_size, 32, 0xFFFF, 24, 0xFFFF, 1, 0, 0x0030, 0, 0, 0)

# Pad data to 4-byte boundary
pad_len = (4 - (data_size % 4)) % 4
padded_data = manifest_xml + (b'\x00' * pad_len)

import os

script_dir = os.path.dirname(os.path.abspath(__file__))
project_dir = os.path.abspath(os.path.join(script_dir, '..'))
res_path = os.path.join(project_dir, 'manifest.res')

with open(res_path, 'wb') as f:
    f.write(null_header + res_header + padded_data)

print(f"Created manifest.res at {res_path} (size: {len(null_header + res_header + padded_data)} bytes)")
