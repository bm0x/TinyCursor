//! Cursor visual states and context types.

#![allow(dead_code)]

/// The semantic visual type of cursor shape currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorVisualKind {
    #[default]
    Arrow,
    Text,
    Pointer,
    Move,
    Wait,
    Hidden,
}

/// Dynamic contextual state of the cursor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorState {
    pub kind: CursorVisualKind,
    pub is_pressed: bool,
    pub is_visible: bool,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            kind: CursorVisualKind::Arrow,
            is_pressed: false,
            is_visible: true,
        }
    }
}
