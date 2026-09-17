//! TinyCursor: High-performance Smooth Cursor for Windows Desktop.
//!
//! Features:
//! - Background System Tray application (no terminal window).
//! - Modern White Arrow & Hand Click Pointer.
//! - Taskbar-topmost layer reinforcement (never occluded by taskbar or icons).
//! - Spring physics (Euler semi-implicit + dynamic sub-stepping).
//! - Click-through overlay with 0% CPU in idle mode.
//! - Fail-safe cursor recovery (hotkey + panic hook + tray exit).

#![windows_subsystem = "windows"]

mod core;
mod platform;

use std::time::{Duration, Instant};
use crate::core::config::CursorConfig;
use crate::core::physics::SmoothCursorPhysics;

#[cfg(target_os = "windows")]
use crate::platform::windows::{
    detect_system_cursor, get_hardware_cursor_pos, get_max_display_frequency,
    is_emergency_escape_pressed, is_left_button_down, restore_system_cursors,
    sample_screen_luminance, CursorDesignOverride, CursorSurface, DxgiPipeline,
    OverlayWindow, RenderCursorKind, SystemCursorGuard, ThemeMode, TrayManager,
};
#[cfg(target_os = "windows")]
use crate::platform::windows::sys::{timeBeginPeriod, timeEndPeriod};

fn main() {
    #[cfg(target_os = "windows")]
    run_windows();

    #[cfg(not(target_os = "windows"))]
    {
        println!("TinyCursor is currently implemented for Windows. macOS and Linux support is under development.");
    }
}

#[cfg(target_os = "windows")]
fn run_windows() {
    // 0. Enable Windows 1ms high-precision timer for ultra-smooth tracking
    unsafe {
        timeBeginPeriod(1);
    }

    let config = CursorConfig::default();

    // 1. Initialize system tray icon in Windows taskbar
    let tray = match TrayManager::new() {
        Some(t) => t,
        None => {
            unsafe { timeEndPeriod(1); }
            return;
        }
    };

    // 2. Query initial hardware cursor position
    let initial_pos = get_hardware_cursor_pos().unwrap_or(crate::core::math::Vec2::new(100.0, 100.0));
    let physics = SmoothCursorPhysics::new(initial_pos, config);

    // 3. Detect initial screen luminance and initialize theme blend state
    let initial_lum = sample_screen_luminance(initial_pos);
    let target_color_blend = if initial_lum > 130.0 { 1.0f32 } else { 0.0f32 };
    let color_theme_blend = target_color_blend;

    // 4. Hide system cursors with RAII fail-safe protection
    let cursor_guard = SystemCursorGuard::hide();

    // 5. Attempt Modern DXGI Flip Model + DirectComposition Hardware Pipeline
    // Hardware VBLANK synchronized Present(1, 0) with ZERO Thread::sleep in render loop
    if let Some((overlay, vx, vy, vw, vh)) = OverlayWindow::new_direct_composition() {
        if let Ok(pipeline) = DxgiPipeline::new(overlay.hwnd(), vx, vy, vw, vh) {
            run_dxgi_loop(
                overlay,
                pipeline,
                physics,
                tray,
                cursor_guard,
                color_theme_blend,
                target_color_blend,
            );
            unsafe { timeEndPeriod(1); }
            restore_system_cursors();
            return;
        }
    }

    // 6. GDI Layered Window Fallback (for older systems or basic display drivers)
    let overlay = match OverlayWindow::new() {
        Some(w) => w,
        None => {
            unsafe { timeEndPeriod(1); }
            restore_system_cursors();
            return;
        }
    };

    let surface = match CursorSurface::new() {
        Some(s) => s,
        None => {
            unsafe { timeEndPeriod(1); }
            restore_system_cursors();
            return;
        }
    };

    run_gdi_loop(
        overlay,
        surface,
        physics,
        tray,
        cursor_guard,
        color_theme_blend,
        target_color_blend,
        initial_pos,
    );

    unsafe {
        timeEndPeriod(1);
    }
    restore_system_cursors();
}

#[cfg(target_os = "windows")]
fn run_dxgi_loop(
    overlay: OverlayWindow,
    mut pipeline: DxgiPipeline,
    mut physics: SmoothCursorPhysics,
    tray: TrayManager,
    mut cursor_guard: Option<SystemCursorGuard>,
    mut color_theme_blend: f32,
    mut target_color_blend: f32,
) {
    let mut was_paused = false;
    let mut last_lum_sample_time = Instant::now();
    let mut last_sample_pos = physics.position;
    let mut last_frame_time = Instant::now();

    loop {
        // A. Check exit conditions (Tray "Salir" or emergency hotkey Ctrl + Alt + Shift + Esc)
        if tray.should_exit() || is_emergency_escape_pressed() {
            break;
        }

        // B. Handle Pause/Resume from Taskbar Tray
        let is_paused = tray.is_paused();
        if is_paused != was_paused {
            was_paused = is_paused;
            if is_paused {
                cursor_guard = None;
                restore_system_cursors();
                let _ = pipeline.clear();
            } else {
                cursor_guard = SystemCursorGuard::hide();
            }
        }

        if is_paused {
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }

        // C. Poll OS window messages
        if !overlay.poll_events() {
            break;
        }

        // D. Query real hardware mouse position
        let current_target = match get_hardware_cursor_pos() {
            Some(pos) => pos,
            None => physics.position,
        };

        // E. Detect click state (left mouse button held down)
        let is_clicking = is_left_button_down();

        // F. Update target color blend based on background luminance or tray preference
        let now = Instant::now();
        match tray.theme_mode() {
            ThemeMode::AlwaysWhite => {
                target_color_blend = 0.0;
            }
            ThemeMode::AlwaysBlack => {
                target_color_blend = 1.0;
            }
            ThemeMode::Auto => {
                let dist_sq = (current_target.x - last_sample_pos.x).powi(2)
                    + (current_target.y - last_sample_pos.y).powi(2);
                if dist_sq > 64.0 && now.duration_since(last_lum_sample_time) >= Duration::from_millis(60) {
                    last_lum_sample_time = now;
                    last_sample_pos = current_target;
                    let lum = sample_screen_luminance(current_target);
                    if lum > 140.0 {
                        target_color_blend = 1.0;
                    } else if lum < 115.0 {
                        target_color_blend = 0.0;
                    }
                }
            }
        }

        // Smooth continuous color morphing
        let blend_diff = target_color_blend - color_theme_blend;
        if blend_diff.abs() > 0.001 {
            let blend_rate = 8.5;
            let dt = (now - last_frame_time).as_secs_f32().clamp(0.001, 0.05);
            color_theme_blend += blend_diff * (1.0 - (-blend_rate * dt).exp());
        } else {
            color_theme_blend = target_color_blend;
        }
        last_frame_time = now;

        // G. Select cursor kind: Dynamic OS detection or Tray override
        let cursor_kind = match tray.design_override() {
            CursorDesignOverride::Auto => {
                detect_system_cursor(current_target, is_clicking, tray.is_hand_on_click_enabled(), overlay.hwnd())
            }
            CursorDesignOverride::Arrow => RenderCursorKind::Arrow,
            CursorDesignOverride::Hand => RenderCursorKind::Hand,
            CursorDesignOverride::IBeam => RenderCursorKind::IBeam,
            CursorDesignOverride::ResizeNS => RenderCursorKind::ResizeNS,
            CursorDesignOverride::ResizeWE => RenderCursorKind::ResizeWE,
            CursorDesignOverride::Move => RenderCursorKind::Move,
            CursorDesignOverride::ZoomIn => RenderCursorKind::ZoomIn,
            CursorDesignOverride::ZoomOut => RenderCursorKind::ZoomOut,
            CursorDesignOverride::Crosshair => RenderCursorKind::Crosshair,
        };

        // H. Hardware VBLANK locked tick: Present(1, 0)
        // STRICT: Zero Thread::sleep, zero Task::delay, zero SetTimer in this loop.
        // Synchronizes directly with the monitor's VBLANK interrupt (144 Hz = ~6.94ms, 240 Hz = ~4.16ms).
        if let Err(_) = pipeline.run_tick(
            &mut physics,
            current_target,
            cursor_kind,
            color_theme_blend,
            is_clicking,
        ) {
            break;
        }
    }

    let _ = pipeline.clear();
    drop(cursor_guard);
    restore_system_cursors();
}

#[cfg(target_os = "windows")]
fn run_gdi_loop(
    overlay: OverlayWindow,
    mut surface: CursorSurface,
    mut physics: SmoothCursorPhysics,
    tray: TrayManager,
    mut cursor_guard: Option<SystemCursorGuard>,
    mut color_theme_blend: f32,
    mut target_color_blend: f32,
    initial_pos: crate::core::math::Vec2,
) {
    surface.render_modern_cursor(
        RenderCursorKind::Arrow,
        physics.angle,
        physics.scale_x,
        physics.scale_y,
        false,
        color_theme_blend,
        &physics.config,
    );
    surface.present(overlay.hwnd(), physics.position);

    let mut last_frame_time = Instant::now();
    let mut last_lum_sample_time = Instant::now();
    let mut last_sample_pos = initial_pos;

    let display_hz = get_max_display_frequency().max(60);
    let target_fps = (display_hz * 2).clamp(120, 500);
    let frame_budget = Duration::from_micros((1_000_000 / target_fps) as u64);
    let mut was_paused = false;

    loop {
        // A. Check exit conditions (Tray "Salir" or emergency hotkey Ctrl + Alt + Shift + Esc)
        if tray.should_exit() || is_emergency_escape_pressed() {
            break;
        }

        // B. Handle Pause/Resume from Taskbar Tray
        let is_paused = tray.is_paused();
        if is_paused != was_paused {
            was_paused = is_paused;
            if is_paused {
                // Temporarily drop guard to restore OS default cursor
                cursor_guard = None;
                restore_system_cursors();
                surface.clear();
                surface.present(overlay.hwnd(), physics.position);
            } else {
                // Re-enable smooth cursor
                cursor_guard = SystemCursorGuard::hide();
            }
        }

        if is_paused {
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }

        // C. Poll OS window messages
        if !overlay.poll_events() {
            break;
        }

        // D. Query real hardware mouse position
        let current_target = match get_hardware_cursor_pos() {
            Some(pos) => pos,
            None => initial_pos,
        };

        // E. Detect click state (left mouse button held down)
        let is_clicking = is_left_button_down();

        // F. Calculate delta time with wake-up smoothing
        let now = Instant::now();
        let raw_dt = (now - last_frame_time).as_secs_f32();
        last_frame_time = now;
        // Avoid physics explosion or abrupt jumps when resuming from idle sleep
        let dt = if raw_dt > 0.033 { 0.003 } else { raw_dt };

        // G. Update target color blend based on user preference or background luminance
        match tray.theme_mode() {
            ThemeMode::AlwaysWhite => {
                target_color_blend = 0.0;
            }
            ThemeMode::AlwaysBlack => {
                target_color_blend = 1.0;
            }
            ThemeMode::Auto => {
                // Only sample background luminance if cursor has physically moved (> 8px)
                // and at least 60ms have elapsed. Eliminates GPU readback stall and PCI-e bus contention.
                let dist_sq = (current_target.x - last_sample_pos.x).powi(2) + (current_target.y - last_sample_pos.y).powi(2);
                if dist_sq > 64.0 && now.duration_since(last_lum_sample_time) >= Duration::from_millis(60) {
                    last_lum_sample_time = now;
                    last_sample_pos = current_target;
                    let lum = sample_screen_luminance(current_target);
                    // Hysteresis deadband:
                    // > 140.0: light background -> target Black cursor (1.0)
                    // < 115.0: dark background -> target White cursor (0.0)
                    // 115.0..=140.0: retain current target to avoid edge jitter
                    if lum > 140.0 {
                        target_color_blend = 1.0;
                    } else if lum < 115.0 {
                        target_color_blend = 0.0;
                    }
                }
            }
        }

        // H. Smooth continuous color morphing (exponential decay ~200ms transition)
        let blend_diff = target_color_blend - color_theme_blend;
        if blend_diff.abs() > 0.001 {
            let blend_rate = 8.5;
            color_theme_blend += blend_diff * (1.0 - (-blend_rate * dt).exp());
        } else {
            color_theme_blend = target_color_blend;
        }

        // I. Advance spring physics with shortest-arc slerp
        physics.update(current_target, dt);

        // J. Select cursor kind: Dynamic OS detection or Tray override
        let cursor_kind = match tray.design_override() {
            CursorDesignOverride::Auto => detect_system_cursor(current_target, is_clicking, tray.is_hand_on_click_enabled(), overlay.hwnd()),
            CursorDesignOverride::Arrow => RenderCursorKind::Arrow,
            CursorDesignOverride::Hand => RenderCursorKind::Hand,
            CursorDesignOverride::IBeam => RenderCursorKind::IBeam,
            CursorDesignOverride::ResizeNS => RenderCursorKind::ResizeNS,
            CursorDesignOverride::ResizeWE => RenderCursorKind::ResizeWE,
            CursorDesignOverride::Move => RenderCursorKind::Move,
            CursorDesignOverride::ZoomIn => RenderCursorKind::ZoomIn,
            CursorDesignOverride::ZoomOut => RenderCursorKind::ZoomOut,
            CursorDesignOverride::Crosshair => RenderCursorKind::Crosshair,
        };

        // K. Render and present to DWM (with HWND_TOPMOST reinforcement over taskbar)
        surface.render_modern_cursor(
            cursor_kind,
            physics.angle,
            physics.scale_x,
            physics.scale_y,
            is_clicking,
            color_theme_blend,
            &physics.config,
        );
        surface.present(overlay.hwnd(), physics.position);

        // L. High-precision low-latency frame pacing for true real-time responsiveness
        let is_color_settled = (color_theme_blend - target_color_blend).abs() < 0.002;
        if physics.is_settled && !is_clicking && is_color_settled {
            std::thread::sleep(Duration::from_millis(2));
        } else {
            let elapsed = now.elapsed();
            if elapsed < frame_budget {
                let remaining = frame_budget - elapsed;
                if remaining > Duration::from_millis(2) {
                    std::thread::sleep(Duration::from_millis(1));
                }
                while now.elapsed() < frame_budget {
                    std::hint::spin_loop();
                }
            }
        }
    }

    // Cleanup: restore system cursors, timer resolution, and tray
    unsafe {
        timeEndPeriod(1);
    }
    drop(cursor_guard);
    restore_system_cursors();
}
