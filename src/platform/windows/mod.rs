//! Windows platform-specific implementations.

#![allow(unused_imports, dead_code)]

pub mod assets;
pub mod cursor_guard;
pub mod input;
pub mod overlay;
pub mod renderer;
pub mod sys;
pub mod tray;

pub use cursor_guard::{is_cursor_hidden, restore_system_cursors, SystemCursorGuard};
pub use input::{
    detect_system_cursor, get_hardware_cursor_pos, get_max_display_frequency,
    is_emergency_escape_pressed, is_left_button_down, sample_screen_luminance,
};
pub use overlay::OverlayWindow;
pub use renderer::{CursorSurface, RenderCursorKind};
pub use tray::{CursorDesignOverride, ThemeMode, TrayManager};

