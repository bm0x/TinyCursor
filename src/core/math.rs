//! Mathematical types and functions for 2D transformations,
//! vector operations, rotation calculations, and squash & stretch deformation.

#![allow(dead_code)]

use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

/// A 2D floating-point vector with utility functions for physics simulation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 1e-6 {
            self / len
        } else {
            Self::ZERO
        }
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Add for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

/// Computes the dynamic squash and stretch scale factors.
/// As speed increases, the cursor stretches along its motion axis (ScaleX)
/// and compresses along the perpendicular axis (ScaleY) to preserve perceived volume.
#[inline]
pub fn compute_squash_and_stretch(speed: f32, max_velocity: f32, stretch_factor: f32) -> (f32, f32) {
    let normalized_speed = (speed / max_velocity.max(1.0)).clamp(0.0, 1.0);
    let scale_x = 1.0 + (normalized_speed * stretch_factor);
    let scale_y = 1.0 / scale_x.sqrt();
    (scale_x, scale_y)
}

/// Computes the target orientation angle in radians given velocity components.
/// If speed is below `min_speed_threshold`, retains `previous_angle` to prevent jitter.
#[inline]
pub fn compute_orientation_angle(velocity: Vec2, previous_angle: f32, min_speed_threshold: f32) -> f32 {
    let speed = velocity.length();
    if speed > min_speed_threshold {
        velocity.y.atan2(velocity.x)
    } else {
        previous_angle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_operations() {
        let a = Vec2::new(3.0, 4.0);
        assert_eq!(a.length(), 5.0);
        let b = Vec2::new(1.0, 2.0);
        assert_eq!(a + b, Vec2::new(4.0, 6.0));
        assert_eq!(a - b, Vec2::new(2.0, 2.0));
    }

    #[test]
    fn test_squash_and_stretch() {
        let (sx, sy) = compute_squash_and_stretch(0.0, 1000.0, 0.4);
        assert!((sx - 1.0).abs() < 1e-5);
        assert!((sy - 1.0).abs() < 1e-5);

        let (sx_fast, sy_fast) = compute_squash_and_stretch(1000.0, 1000.0, 0.4);
        assert!((sx_fast - 1.4).abs() < 1e-5);
        assert!(sy_fast < 1.0);
        // Area conservation: sx * sy^2 == 1.0
        assert!((sx_fast * sy_fast * sy_fast - 1.0).abs() < 1e-4);
    }
}
