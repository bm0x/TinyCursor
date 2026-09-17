# TinyCursor 🚀
> High-performance native desktop smooth cursor engine with critically damped spring dynamics, real-time refresh rate synchronization, and 14 adaptive dual-theme designs. Built in 100% pure native Rust.

[![CI](https://github.com/rafy2/TinyCursor/actions/workflows/ci.yml/badge.svg)](https://github.com/rafy2/TinyCursor/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011%20(64--bit)-lightgrey.svg)]()
[![Rust](https://img.shields.io/badge/Rust-2021%20Edition-orange.svg)]()

Website: **[https://tinycursor.vercel.app](https://tinycursor.vercel.app)**

---

## ✨ Features

* **⚡ 288Hz+ Ultra-High Polling Rate:**
  * Dynamically detects your display's hardware refresh rate (60Hz, 144Hz, 240Hz, 360Hz) and paces physics updates at $2\times$ screen Hz.
  * Unlocks 1ms high-precision Windows multimedia timer (`timeBeginPeriod(1)`) for sub-millisecond input latency.
* **🧲 Critically Damped Spring Dynamics:**
  * Euler semi-implicit integration with dynamic sub-stepping at 500 Hz ($\zeta \approx 0.986$, $k = 750$). Eliminates jitter and unnatural bouncing while maintaining organic physical trailing.
* **🎨 14 Adaptive Dual-Theme Designs:**
  * Full suite of handcrafted pointers in Arctic White and Obsidian Black:
    * Standard Arrow, Link Hand Pointer, Help, Wait, Crosshair, Text I-Beam, Pencil.
    * 4-Axis Window Resizing (`resize_ns`, `resize_we`, `resize_nwse`, `resize_nesw`).
    * Move & Pan, Zoom In (+), Zoom Out (-), Unavailable.
  * Real-time background luminance sampling: seamlessly morphs color to maintain 100% readability across light and dark surfaces.
* **🪟 Universal Overlay (UIAccess & Band Integration):**
  * Floats smoothly over Windows desktop, taskbar, Start Menu, Notification Center, and fullscreen windows.
* **🦀 100% Pure Native Rust:**
  * Zero heavy runtime frameworks (no Electron, no Chromium).
  * Consumes under 15 MB of RAM and $<0.2\%$ CPU in active movement (0.0% in idle mode).
* **🛡️ Privacy-First & 100% Open Source:**
  * Fully transparent under the permissive MIT license.
  * Zero telemetry, zero cloud tracking, zero network activity.

---

## 📦 Installation

### Option 1: Automated Windows Installer (.exe)
Download the latest `TinyCursor-Setup-x64.exe` from [GitHub Releases](https://github.com/rafy2/TinyCursor/releases/latest).
The Inno Setup wizard installs TinyCursor in `C:\Program Files\TinyCursor` with optional auto-start on Windows boot.

### Option 2: Portable (.zip)
Download `TinyCursor-Portable-x64.zip`, extract anywhere, and run `tiny-cursor.exe`.

### Option 3: Build from Source
```bash
# Clone the repository
git clone https://github.com/rafy2/TinyCursor.git
cd TinyCursor

# Run unit tests
cargo test

# Build optimized release binary
cargo build --release
```

---

## 🛠️ Architecture

```
TinyCursor/
├── .github/
│   └── workflows/
│       ├── ci.yml                             # Automated testing on Windows
│       └── release.yml                        # Automated Inno Setup + SignPath release pipeline
├── Cargo.toml                                 # Zero external dependencies
├── LICENSE                                    # OSI-approved MIT License
├── installer.iss                              # Inno Setup 6 packaging script
├── vercel.json                                # Vercel landing page deployment config
├── web/                                       # Project landing page & interactive preview
├── assets/                                    # 14 Dual-theme cursor sprites
├── tools/                                     # Diagnostic, resource, and test utilities
└── src/
    ├── main.rs                                # Entrypoint, refresh rate pacing, event loop
    ├── core/                                  # 100% Platform-agnostic mathematical core
    │   ├── math.rs                            # 2D Vectors, angles, squash & stretch
    │   ├── physics.rs                         # Euler semi-implicit spring dynamics
    │   └── config.rs                          # Calibrated stiffness and damping
    └── platform/windows/                      # Windows native backend
        ├── assets.rs                          # Pre-compiled ARGB bitmap memory arrays
        ├── overlay.rs                         # Transparent layered window
        ├── tray.rs                            # Taskbar system tray menu
        ├── input.rs                           # Cursor context detection & click states
        └── renderer.rs                        # Bilinear sampling & DWM composition
```

---

## 🔐 Privacy Policy

TinyCursor does not collect, record, or transmit any user data, screen content, keystrokes, or personal information. All luminance sampling for contrast adaptation occurs strictly in local RAM around the cursor position and is discarded every frame. The application works completely offline.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
Code signing provided by the [SignPath Foundation](https://signpath.org) for open-source projects.
