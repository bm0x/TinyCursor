//! High-Performance Vector & Sprite Renderer for Modern White Cursor.
//!
//! Features:
//! - Exact pixel-perfect Modern White Arrow and Click Hand with SOLID pure white interior.
//! - Sub-pixel bilinear texture sampling with inverse rotation and squash & stretch.
//! - Soft ambient drop shadow.
//! - Taskbar & system topmost Z-order reinforcement (`HWND_TOPMOST`).

use crate::core::config::CursorConfig;
use crate::core::math::Vec2;
use std::ptr::null_mut;
use super::assets::*;
use super::sys::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject,
    SetWindowPos, UpdateLayeredWindow, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HBITMAP, HDC,
    HWND, HWND_TOPMOST, POINT, SIZE, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOOWNERZORDER, SWP_NOSIZE, SWP_SHOWWINDOW, ULW_ALPHA,
};

pub const CANVAS_SIZE: i32 = 128;
const ANCHOR: f32 = 64.0;

// Base resting angle of Modern White arrow in radians (~ -118 degrees, top-left)
const ARROW_BASE_ANGLE: f32 = -2.06;

/// Visual mode for the rendered cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderCursorKind {
    Arrow,
    Hand,
    IBeam,
    Crosshair,
    ResizeNS,
    ResizeWE,
    ResizeNWSE,
    ResizeNESW,
    Move,
    ZoomIn,
    ZoomOut,
    Wait,
    Help,
    Unavailable,
}

/// 32-bit ARGB DIBSection surface with DWM hardware-accelerated presentation.
pub struct CursorSurface {
    hdc_mem: HDC,
    hbitmap: HBITMAP,
    old_bitmap: HBITMAP,
    pixels: *mut u32,
    blend: BLENDFUNCTION,
    size: SIZE,
    src_point: POINT,
}

impl CursorSurface {
    /// Creates the offscreen 32-bit premultiplied ARGB DIBSection surface.
    pub fn new() -> Option<Self> {
        unsafe {
            let hdc_mem = CreateCompatibleDC(null_mut());
            if hdc_mem.is_null() {
                return None;
            }

            let mut bmi: BITMAPINFO = std::mem::zeroed();
            bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmi_header.bi_width = CANVAS_SIZE;
            bmi.bmi_header.bi_height = -CANVAS_SIZE; // Top-down
            bmi.bmi_header.bi_planes = 1;
            bmi.bmi_header.bi_bit_count = 32;
            bmi.bmi_header.bi_compression = BI_RGB;

            let mut bits: *mut std::ffi::c_void = null_mut();
            let hbitmap = CreateDIBSection(
                hdc_mem,
                &bmi,
                DIB_RGB_COLORS,
                &mut bits,
                null_mut(),
                0,
            );

            if hbitmap.is_null() || bits.is_null() {
                DeleteDC(hdc_mem);
                return None;
            }

            let old_bitmap = SelectObject(hdc_mem, hbitmap) as HBITMAP;

            let blend = BLENDFUNCTION {
                blend_op: AC_SRC_OVER,
                blend_flags: 0,
                source_constant_alpha: 255,
                alpha_format: AC_SRC_ALPHA,
            };

            Some(Self {
                hdc_mem,
                hbitmap,
                old_bitmap,
                pixels: bits as *mut u32,
                blend,
                size: SIZE {
                    cx: CANVAS_SIZE,
                    cy: CANVAS_SIZE,
                },
                src_point: POINT { x: 0, y: 0 },
            })
        }
    }

    /// Clears the canvas to full transparency.
    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            std::ptr::write_bytes(self.pixels, 0, (CANVAS_SIZE * CANVAS_SIZE) as usize);
        }
    }

    /// Renders the Modern cursor (Arrow or Click Hand) with solid interior,
    /// smooth continuous rotation, fluid squash & stretch, and dynamic White <-> Black theme morphing.
    pub fn render_modern_cursor(
        &mut self,
        kind: RenderCursorKind,
        angle: f32,
        scale_x: f32,
        scale_y: f32,
        is_click_pressed: bool,
        color_blend: f32, // 0.0 = Pure White, 1.0 = Pure Black
        _config: &CursorConfig,
    ) {
        self.clear();

        match kind {
            RenderCursorKind::Arrow => {
                // Align arrow tip with velocity trajectory in 360 degrees
                let delta_angle = angle + std::f32::consts::FRAC_PI_2;

                let base_scale = 0.50;
                let sx = scale_y * base_scale;
                let sy = scale_x * base_scale;

                // 1. Soft drop shadow (offset down-right)
                self.sample_and_blit(
                    &WHITE_ARROW_PIXELS,
                    &BLACK_ARROW_PIXELS,
                    color_blend,
                    ARROW_HOTSPOT_X,
                    ARROW_HOTSPOT_Y,
                    delta_angle,
                    sx,
                    sy,
                    Vec2::new(1.5, 2.5),
                    0.28,
                    true,
                );

                // 2. Main crisp Modern Arrow with dynamic theme blending
                self.sample_and_blit(
                    &WHITE_ARROW_PIXELS,
                    &BLACK_ARROW_PIXELS,
                    color_blend,
                    ARROW_HOTSPOT_X,
                    ARROW_HOTSPOT_Y,
                    delta_angle,
                    sx,
                    sy,
                    Vec2::ZERO,
                    1.0,
                    false,
                );
            }
            RenderCursorKind::Hand => {
                let click_squash = if is_click_pressed { 0.90 } else { 1.0 };
                let base_scale = 0.48 * click_squash;

                // 1. Soft drop shadow
                self.sample_and_blit(
                    &WHITE_HAND_PIXELS,
                    &BLACK_HAND_PIXELS,
                    color_blend,
                    HAND_HOTSPOT_X,
                    HAND_HOTSPOT_Y,
                    0.0,
                    base_scale,
                    base_scale,
                    Vec2::new(1.5, 2.5),
                    0.28,
                    true,
                );

                // 2. Main Modern Click Hand with dynamic theme blending
                self.sample_and_blit(
                    &WHITE_HAND_PIXELS,
                    &BLACK_HAND_PIXELS,
                    color_blend,
                    HAND_HOTSPOT_X,
                    HAND_HOTSPOT_Y,
                    0.0,
                    base_scale,
                    base_scale,
                    Vec2::ZERO,
                    1.0,
                    false,
                );
            }
            other => {
                let (w_tex, b_tex, hx, hy) = match other {
                    RenderCursorKind::IBeam => (&WHITE_IBEAM_PIXELS, &BLACK_IBEAM_PIXELS, IBEAM_HOTSPOT_X, IBEAM_HOTSPOT_Y),
                    RenderCursorKind::Crosshair => (&WHITE_CROSSHAIR_PIXELS, &BLACK_CROSSHAIR_PIXELS, CROSSHAIR_HOTSPOT_X, CROSSHAIR_HOTSPOT_Y),
                    RenderCursorKind::ResizeNS => (&WHITE_RESIZE_NS_PIXELS, &BLACK_RESIZE_NS_PIXELS, RESIZE_NS_HOTSPOT_X, RESIZE_NS_HOTSPOT_Y),
                    RenderCursorKind::ResizeWE => (&WHITE_RESIZE_WE_PIXELS, &BLACK_RESIZE_WE_PIXELS, RESIZE_WE_HOTSPOT_X, RESIZE_WE_HOTSPOT_Y),
                    RenderCursorKind::ResizeNWSE => (&WHITE_RESIZE_NWSE_PIXELS, &BLACK_RESIZE_NWSE_PIXELS, RESIZE_NWSE_HOTSPOT_X, RESIZE_NWSE_HOTSPOT_Y),
                    RenderCursorKind::ResizeNESW => (&WHITE_RESIZE_NESW_PIXELS, &BLACK_RESIZE_NESW_PIXELS, RESIZE_NESW_HOTSPOT_X, RESIZE_NESW_HOTSPOT_Y),
                    RenderCursorKind::Move => (&WHITE_MOVE_PIXELS, &BLACK_MOVE_PIXELS, MOVE_HOTSPOT_X, MOVE_HOTSPOT_Y),
                    RenderCursorKind::ZoomIn => (&WHITE_ZOOM_IN_PIXELS, &BLACK_ZOOM_IN_PIXELS, ZOOM_IN_HOTSPOT_X, ZOOM_IN_HOTSPOT_Y),
                    RenderCursorKind::ZoomOut => (&WHITE_ZOOM_OUT_PIXELS, &BLACK_ZOOM_OUT_PIXELS, ZOOM_OUT_HOTSPOT_X, ZOOM_OUT_HOTSPOT_Y),
                    RenderCursorKind::Wait => (&WHITE_WAIT_PIXELS, &BLACK_WAIT_PIXELS, WAIT_HOTSPOT_X, WAIT_HOTSPOT_Y),
                    RenderCursorKind::Help => (&WHITE_HELP_PIXELS, &BLACK_HELP_PIXELS, HELP_HOTSPOT_X, HELP_HOTSPOT_Y),
                    RenderCursorKind::Unavailable => (&WHITE_UNAVAILABLE_PIXELS, &BLACK_UNAVAILABLE_PIXELS, UNAVAILABLE_HOTSPOT_X, UNAVAILABLE_HOTSPOT_Y),
                    _ => unreachable!(),
                };
                let base_scale = 0.50;

                // 1. Soft drop shadow
                self.sample_and_blit(
                    w_tex,
                    b_tex,
                    color_blend,
                    hx,
                    hy,
                    0.0,
                    base_scale,
                    base_scale,
                    Vec2::new(1.5, 2.5),
                    0.28,
                    true,
                );

                // 2. Main body
                self.sample_and_blit(
                    w_tex,
                    b_tex,
                    color_blend,
                    hx,
                    hy,
                    0.0,
                    base_scale,
                    base_scale,
                    Vec2::ZERO,
                    1.0,
                    false,
                );
            }
        }
    }

    /// Inverse-transforms and bilinearly samples the sprite into the DIB canvas,
    /// smoothly cross-fading between White and Black textures according to `color_blend`.
    fn sample_and_blit(
        &mut self,
        white_tex: &[u32; SPRITE_DIM * SPRITE_DIM],
        black_tex: &[u32; SPRITE_DIM * SPRITE_DIM],
        color_blend: f32,
        hotspot_x: f32,
        hotspot_y: f32,
        rot_angle: f32,
        scale_x: f32,
        scale_y: f32,
        offset: Vec2,
        alpha_mult: f32,
        is_shadow: bool,
    ) {
        let blend = color_blend.clamp(0.0, 1.0);
        let cos_r = rot_angle.cos();
        let sin_r = rot_angle.sin();

        let center_x = ANCHOR + offset.x;
        let center_y = ANCHOR + offset.y;

        let bound_radius = (SPRITE_DIM as f32 * scale_x.max(scale_y) * 0.75 + 10.0) as i32;
        let min_x = (center_x as i32 - bound_radius).clamp(0, CANVAS_SIZE);
        let max_x = (center_x as i32 + bound_radius).clamp(0, CANVAS_SIZE);
        let min_y = (center_y as i32 - bound_radius).clamp(0, CANVAS_SIZE);
        let max_y = (center_y as i32 + bound_radius).clamp(0, CANVAS_SIZE);

        for y in min_y..max_y {
            let dy = y as f32 - center_y;
            for x in min_x..max_x {
                let dx = x as f32 - center_x;

                let rx = dx * cos_r + dy * sin_r;
                let ry = -dx * sin_r + dy * cos_r;

                let unscaled_x = rx / scale_x;
                let unscaled_y = ry / scale_y;

                let tx = unscaled_x + hotspot_x;
                let ty = unscaled_y + hotspot_y;

                if tx >= 0.0 && tx < (SPRITE_DIM - 1) as f32 && ty >= 0.0 && ty < (SPRITE_DIM - 1) as f32 {
                    let x0 = tx as usize;
                    let y0 = ty as usize;
                    let x1 = x0 + 1;
                    let y1 = y0 + 1;

                    let fx = tx - x0 as f32;
                    let fy = ty - y0 as f32;

                    let bilerp_chan = |c00: u32, c10: u32, c01: u32, c11: u32| -> f32 {
                        let top = c00 as f32 * (1.0 - fx) + c10 as f32 * fx;
                        let bot = c01 as f32 * (1.0 - fx) + c11 as f32 * fx;
                        top * (1.0 - fy) + bot * fy
                    };

                    let pw00 = white_tex[y0 * SPRITE_DIM + x0];
                    let pw10 = white_tex[y0 * SPRITE_DIM + x1];
                    let pw01 = white_tex[y1 * SPRITE_DIM + x0];
                    let pw11 = white_tex[y1 * SPRITE_DIM + x1];

                    if is_shadow {
                        let a = if blend <= 0.001 {
                            bilerp_chan(pw00 >> 24, pw10 >> 24, pw01 >> 24, pw11 >> 24) * alpha_mult
                        } else if blend >= 0.999 {
                            let pb00 = black_tex[y0 * SPRITE_DIM + x0];
                            let pb10 = black_tex[y0 * SPRITE_DIM + x1];
                            let pb01 = black_tex[y1 * SPRITE_DIM + x0];
                            let pb11 = black_tex[y1 * SPRITE_DIM + x1];
                            bilerp_chan(pb00 >> 24, pb10 >> 24, pb01 >> 24, pb11 >> 24) * alpha_mult
                        } else {
                            let pb00 = black_tex[y0 * SPRITE_DIM + x0];
                            let pb10 = black_tex[y0 * SPRITE_DIM + x1];
                            let pb01 = black_tex[y1 * SPRITE_DIM + x0];
                            let pb11 = black_tex[y1 * SPRITE_DIM + x1];
                            let aw = bilerp_chan(pw00 >> 24, pw10 >> 24, pw01 >> 24, pw11 >> 24);
                            let ab = bilerp_chan(pb00 >> 24, pb10 >> 24, pb01 >> 24, pb11 >> 24);
                            (aw * (1.0 - blend) + ab * blend) * alpha_mult
                        };

                        if a > 1.0 {
                            let dest_idx = (y * CANVAS_SIZE + x) as usize;
                            let shadow_a = (a * 0.40).clamp(0.0, 255.0) as u32;
                            self.blend_over(dest_idx, 0, 0, 0, shadow_a);
                        }
                    } else {
                        let (r, g, b, a) = if blend <= 0.001 {
                            let aw = bilerp_chan(pw00 >> 24, pw10 >> 24, pw01 >> 24, pw11 >> 24);
                            let rw = bilerp_chan((pw00 >> 16) & 0xFF, (pw10 >> 16) & 0xFF, (pw01 >> 16) & 0xFF, (pw11 >> 16) & 0xFF);
                            let gw = bilerp_chan((pw00 >> 8) & 0xFF, (pw10 >> 8) & 0xFF, (pw01 >> 8) & 0xFF, (pw11 >> 8) & 0xFF);
                            let bw = bilerp_chan(pw00 & 0xFF, pw10 & 0xFF, pw01 & 0xFF, pw11 & 0xFF);
                            (rw, gw, bw, aw * alpha_mult)
                        } else if blend >= 0.999 {
                            let pb00 = black_tex[y0 * SPRITE_DIM + x0];
                            let pb10 = black_tex[y0 * SPRITE_DIM + x1];
                            let pb01 = black_tex[y1 * SPRITE_DIM + x0];
                            let pb11 = black_tex[y1 * SPRITE_DIM + x1];
                            let ab = bilerp_chan(pb00 >> 24, pb10 >> 24, pb01 >> 24, pb11 >> 24);
                            let rb = bilerp_chan((pb00 >> 16) & 0xFF, (pb10 >> 16) & 0xFF, (pb01 >> 16) & 0xFF, (pb11 >> 16) & 0xFF);
                            let gb = bilerp_chan((pb00 >> 8) & 0xFF, (pb10 >> 8) & 0xFF, (pb01 >> 8) & 0xFF, (pb11 >> 8) & 0xFF);
                            let bb = bilerp_chan(pb00 & 0xFF, pb10 & 0xFF, pb01 & 0xFF, pb11 & 0xFF);
                            (rb, gb, bb, ab * alpha_mult)
                        } else {
                            let inv_t = 1.0 - blend;
                            let t = blend;

                            let aw = bilerp_chan(pw00 >> 24, pw10 >> 24, pw01 >> 24, pw11 >> 24);
                            let rw = bilerp_chan((pw00 >> 16) & 0xFF, (pw10 >> 16) & 0xFF, (pw01 >> 16) & 0xFF, (pw11 >> 16) & 0xFF);
                            let gw = bilerp_chan((pw00 >> 8) & 0xFF, (pw10 >> 8) & 0xFF, (pw01 >> 8) & 0xFF, (pw11 >> 8) & 0xFF);
                            let bw = bilerp_chan(pw00 & 0xFF, pw10 & 0xFF, pw01 & 0xFF, pw11 & 0xFF);

                            let pb00 = black_tex[y0 * SPRITE_DIM + x0];
                            let pb10 = black_tex[y0 * SPRITE_DIM + x1];
                            let pb01 = black_tex[y1 * SPRITE_DIM + x0];
                            let pb11 = black_tex[y1 * SPRITE_DIM + x1];

                            let ab = bilerp_chan(pb00 >> 24, pb10 >> 24, pb01 >> 24, pb11 >> 24);
                            let rb = bilerp_chan((pb00 >> 16) & 0xFF, (pb10 >> 16) & 0xFF, (pb01 >> 16) & 0xFF, (pb11 >> 16) & 0xFF);
                            let gb = bilerp_chan((pb00 >> 8) & 0xFF, (pb10 >> 8) & 0xFF, (pb01 >> 8) & 0xFF, (pb11 >> 8) & 0xFF);
                            let bb = bilerp_chan(pb00 & 0xFF, pb10 & 0xFF, pb01 & 0xFF, pb11 & 0xFF);

                            let a = (aw * inv_t + ab * t) * alpha_mult;
                            let r = rw * inv_t + rb * t;
                            let g = gw * inv_t + gb * t;
                            let b = bw * inv_t + bb * t;
                            (r, g, b, a)
                        };

                        if a > 1.0 {
                            let dest_idx = (y * CANVAS_SIZE + x) as usize;
                            let pr = (r * (a / 255.0)).clamp(0.0, 255.0) as u32;
                            let pg = (g * (a / 255.0)).clamp(0.0, 255.0) as u32;
                            let pb = (b * (a / 255.0)).clamp(0.0, 255.0) as u32;
                            let pa = a.clamp(0.0, 255.0) as u32;

                            self.blend_over(dest_idx, pr, pg, pb, pa);
                        }
                    }
                }
            }
        }
    }

    #[inline]
    fn blend_over(&mut self, dest_idx: usize, src_r: u32, src_g: u32, src_b: u32, src_a: u32) {
        unsafe {
            let dest = self.pixels.add(dest_idx);
            let d_val = *dest;

            if d_val == 0 {
                *dest = (src_a << 24) | (src_r << 16) | (src_g << 8) | src_b;
            } else {
                let inv_a = 255 - src_a;
                let d_a = (d_val >> 24) & 0xFF;
                let d_r = (d_val >> 16) & 0xFF;
                let d_g = (d_val >> 8) & 0xFF;
                let d_b = d_val & 0xFF;

                let out_a = src_a + ((d_a * inv_a + 127) / 255);
                let out_r = src_r + ((d_r * inv_a + 127) / 255);
                let out_g = src_g + ((d_g * inv_a + 127) / 255);
                let out_b = src_b + ((d_b * inv_a + 127) / 255);

                *dest = (out_a.min(255) << 24) | (out_r.min(255) << 16) | (out_g.min(255) << 8) | out_b.min(255);
            }
        }
    }

    /// Commits the rendered sprite to DWM and reinforces HWND_TOPMOST priority
    /// so the cursor stays above the Windows taskbar, start menu, and notifications.
    pub fn present(&self, hwnd: HWND, visual_pos: Vec2) {
        let dst_point = POINT {
            x: (visual_pos.x - ANCHOR) as i32,
            y: (visual_pos.y - ANCHOR) as i32,
        };

        unsafe {
            // Continuously reinforce topmost Z-order without fighting UpdateLayeredWindow's coordinates
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_SHOWWINDOW,
            );

            UpdateLayeredWindow(
                hwnd,
                null_mut(),
                &dst_point,
                &self.size,
                self.hdc_mem,
                &self.src_point,
                0,
                &self.blend,
                ULW_ALPHA,
            );
        }
    }
}

impl Drop for CursorSurface {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.hdc_mem, self.old_bitmap);
            DeleteObject(self.hbitmap);
            DeleteDC(self.hdc_mem);
        }
    }
}
