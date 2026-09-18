//! Spring physics engine with Semi-implicit Euler integration,
//! continuous shortest-arc angular interpolation, and organic squash & stretch.

use super::config::CursorConfig;
use super::math::{compute_orientation_angle, compute_squash_and_stretch, Vec2};

// Canonical resting angle (~ -135 degrees, pointing up-left like standard Windows cursor)
pub const RESTING_ANGLE: f32 = -std::f32::consts::FRAC_PI_2 - std::f32::consts::FRAC_PI_4;

/// Manages the second-order spring dynamics and smooth rotational trajectory.
#[derive(Debug, Clone)]
pub struct SmoothCursorPhysics {
    /// Current interpolated visual position (where the cursor is drawn).
    pub position: Vec2,

    /// Instantaneous velocity vector of the visual cursor.
    pub velocity: Vec2,

    /// Current visual orientation angle (in radians) with shortest-arc smoothing.
    pub angle: f32,

    /// Horizontal scale along motion axis (stretch).
    pub scale_x: f32,

    /// Vertical scale perpendicular to motion axis (squash).
    pub scale_y: f32,

    /// Physics simulation parameters.
    pub config: CursorConfig,

    /// Whether the simulation has reached equilibrium (visual == target, velocity == 0).
    pub is_settled: bool,
}

impl SmoothCursorPhysics {
    /// Creates a new physics simulation initialized at `initial_pos`.
    pub fn new(initial_pos: Vec2, config: CursorConfig) -> Self {
        Self {
            position: initial_pos,
            velocity: Vec2::ZERO,
            angle: RESTING_ANGLE,
            scale_x: 1.0,
            scale_y: 1.0,
            config,
            is_settled: true,
        }
    }

    /// Snaps the visual position to `pos` and resets velocity.
    #[allow(dead_code)]
    pub fn teleport(&mut self, pos: Vec2) {
        self.position = pos;
        self.velocity = Vec2::ZERO;
        self.scale_x = 1.0;
        self.scale_y = 1.0;
        self.is_settled = true;
    }

    /// Advances the physics simulation towards `target_pos` across elapsed time `delta_time`.
    ///
    /// Sub-steps physics at 250 Hz and smoothly interpolates orientation angle and deformation,
    /// ensuring that abrupt reversals in direction produce a graceful, fluid sweeping arc.
    pub fn update(&mut self, target_pos: Vec2, delta_time: f32) {
        // Clamp total delta to avoid divergence on lag spikes
        let dt = delta_time.clamp(0.0005, 0.05);

        // Fixed sub-stepping: maximum 2ms per sub-step (500 Hz precision)
        const MAX_SUB_STEP: f32 = 0.002;
        let sub_steps = (dt / MAX_SUB_STEP).ceil() as usize;
        let sub_step_dt = dt / (sub_steps as f32);

        let k = self.config.stiffness;
        let c = self.config.damping;

        for _ in 0..sub_steps {
            // Spring acceleration: F = k * (target - current) - c * v
            let displacement = target_pos - self.position;
            let force = displacement * k - self.velocity * c;

            // Semi-implicit Euler integration
            self.velocity += force * sub_step_dt;
            self.position += self.velocity * sub_step_dt;
        }

        let speed = self.velocity.length();
        let distance_to_target = self.position.distance(target_pos);

        // Check if settled to allow 0% CPU sleeping
        if speed < self.config.idle_speed_threshold
            && distance_to_target < self.config.idle_distance_threshold
        {
            self.position = target_pos;
            self.velocity = Vec2::ZERO;

            // Smoothly return scale to 1.0
            self.scale_x += (1.0 - self.scale_x) * (1.0 - (-12.0 * dt).exp());
            self.scale_y = 1.0 / self.scale_x.sqrt();

            // Smoothly relax orientation to canonical resting pose (pointing up-left)
            let angle_diff = (RESTING_ANGLE - self.angle + std::f32::consts::PI)
                .rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;
            if angle_diff.abs() > 0.005 {
                self.angle += angle_diff * (1.0 - (-14.0 * dt).exp());
                self.is_settled = false;
            } else {
                self.angle = RESTING_ANGLE;
                self.is_settled = true;
            }
            return;
        }

        self.is_settled = false;

        // 1. Target orientation angle based on velocity
        // When moving, the arrow points along velocity vector.
        // When slowing down below threshold, it fluidly recovers towards canonical resting angle.
        let (target_angle, angular_speed) = if speed > self.config.min_rotation_speed {
            (self.velocity.y.atan2(self.velocity.x), 24.0 + (speed / 300.0).min(16.0))
        } else {
            (RESTING_ANGLE, 12.0)
        };

        // 2. Shortest-arc angular interpolation (Slerp)
        // Prevents abrupt 180-degree flips when reversing mouse direction.
        let angle_diff = (target_angle - self.angle + std::f32::consts::PI)
            .rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;

        let angular_blend = (1.0 - (-angular_speed * dt).exp()).clamp(0.0, 1.0);
        self.angle += angle_diff * angular_blend;

        // 3. Smooth Squash & Stretch deformation
        let (target_sx, _) = compute_squash_and_stretch(
            speed,
            self.config.max_velocity,
            self.config.stretch_factor,
        );

        let scale_blend = (1.0 - (-20.0 * dt).exp()).clamp(0.0, 1.0);
        self.scale_x += (target_sx - self.scale_x) * scale_blend;
        self.scale_y = 1.0 / self.scale_x.sqrt();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_convergence() {
        let config = CursorConfig::default();
        let mut physics = SmoothCursorPhysics::new(Vec2::ZERO, config);

        let target = Vec2::new(500.0, 300.0);

        for _ in 0..120 {
            physics.update(target, 1.0 / 60.0);
        }

        assert!(physics.position.distance(target) < 1.0);
        assert!(physics.is_settled);
    }
}
