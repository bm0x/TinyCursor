//! Platform Abstraction Layer (PAL).
//!
//! Provides OS-agnostic interfaces for window overlays, input tracking,
//! and cursor management, with specialized implementations per operating system.

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;
