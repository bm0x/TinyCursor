//! Core engine modules: math, physics simulation, configuration, and states.

#![allow(unused_imports)]

pub mod config;
pub mod math;
pub mod physics;
pub mod state;

pub use config::CursorConfig;
pub use math::{compute_orientation_angle, compute_squash_and_stretch, Vec2};
pub use physics::SmoothCursorPhysics;
pub use state::{CursorState, CursorVisualKind};
