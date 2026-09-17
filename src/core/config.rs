//! Configuration parameters for Smooth Cursor spring dynamics,
//! visual appearance, and performance thresholds.

/// Configuration parameters for the smooth cursor simulation and rendering.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CursorConfig {
    /// Angular frequency squared ($\omega_n^2$). Higher values make the cursor
    /// snap more aggressively to the real hardware position.
    /// Recommended range: 250.0 - 550.0. Default: 380.0.
    pub stiffness: f32,

    /// Damping coefficient ($2\zeta\omega_n$). Controls the friction/damping.
    /// Values around $\zeta \approx 0.7$ give a snappy spring with minimal overshoot.
    /// Recommended range: 24.0 - 38.0. Default: 30.0.
    pub damping: f32,

    /// Maximum velocity reference for the squash and stretch calculation (pixels/second).
    /// Default: 3500.0.
    pub max_velocity: f32,

    /// Elasticity coefficient ($\lambda_{\text{stretch}}$) for squash & stretch deformation.
    /// Recommended range: 0.2 - 0.5. Default: 0.38.
    pub stretch_factor: f32,

    /// Minimum speed threshold (pixels/second) before the cursor rotates.
    /// Below this speed, the orientation angle is locked to prevent jitter.
    /// Default: 20.0.
    pub min_rotation_speed: f32,

    /// Base width/size of the rendered cursor in logical pixels.
    /// Default: 24.0.
    pub cursor_size: f32,

    /// Primary fill color in [R, G, B, A] format (normalized 0.0 - 1.0).
    /// Default: Deep sleek charcoal/black with subtle glow.
    pub color_fill: [f32; 4],

    /// Border stroke color in [R, G, B, A] format.
    /// Default: Crisp white border.
    pub color_stroke: [f32; 4],

    /// Stroke thickness for cursor border.
    pub stroke_width: f32,

    /// Speed below which the physics simulation enters the idle/sleep state
    /// to reduce CPU and GPU usage to 0%.
    pub idle_speed_threshold: f32,

    /// Distance between visual and real cursor below which the physics settles.
    pub idle_distance_threshold: f32,
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self {
            stiffness: 1024.0,
            damping: 46.08,
            max_velocity: 3200.0,
            stretch_factor: 0.35,
            min_rotation_speed: 20.0,
            cursor_size: 24.0,
            color_fill: [0.08, 0.08, 0.10, 0.95],
            color_stroke: [1.0, 1.0, 1.0, 0.98],
            stroke_width: 1.5,
            idle_speed_threshold: 0.2,
            idle_distance_threshold: 0.3,
        }
    }
}
