//! Modern DirectX 11 / DXGI Modern Flip Model + DirectComposition + Direct2D Hardware Pipeline.
//!
//! Features:
//! - DXGI Modern Flip Model (`DXGI_SWAP_EFFECT_FLIP_DISCARD`) with premultiplied alpha.
//! - DirectComposition visual tree integration on Windows 10/11 with zero redirection buffer.
//! - Hardware VBLANK synchronized `Present(1, 0)` with ZERO Thread::sleep or spin-wait.
//! - Hardware GPU rasterization using high-resolution textures (`ID2D1Bitmap`) with bilinear filtering.
//! - Calibrated orientation: smooth organic delta tilt for Arrow, fixed upright orientation for Hand/I-Beam.
//! - Real-time contrast adaptation (dynamic Pearl White <-> Obsidian Black cross-fade).
//! - Continuous topmost Z-order reinforcement (`HWND_TOPMOST`) over shell and taskbar.

use windows::{
    core::*,
    Foundation::Numerics::Matrix3x2,
    Win32::Foundation::*,
    Win32::Graphics::Direct2D::*,
    Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D11::*,
    Win32::Graphics::DirectComposition::*,
    Win32::Graphics::Dxgi::*,
    Win32::Graphics::Dxgi::Common::*,
    Win32::System::Performance::*,
    Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};

use crate::core::math::Vec2;
use crate::core::physics::SmoothCursorPhysics;
use super::assets::*;
use super::renderer::RenderCursorKind;

/// Pre-loaded pair of high-resolution White and Black GPU textures for a cursor state.
struct GpuCursorPair {
    white: ID2D1Bitmap1,
    black: ID2D1Bitmap1,
    hotspot_x: f32,
    hotspot_y: f32,
}

pub struct DxgiPipeline {
    hwnd: HWND,
    _device: ID3D11Device,
    _context: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain1,
    _dcomp_device: IDCompositionDevice,
    _dcomp_target: IDCompositionTarget,
    _dcomp_visual: IDCompositionVisual,
    _d2d_factory: ID2D1Factory1,
    d2d_context: ID2D1DeviceContext,
    _d2d_target_bitmap: ID2D1Bitmap1,

    // High-fidelity GPU textures for all system cursor states
    tex_arrow: GpuCursorPair,
    tex_hand: GpuCursorPair,
    tex_ibeam: GpuCursorPair,
    tex_crosshair: GpuCursorPair,
    tex_resize_ns: GpuCursorPair,
    tex_resize_we: GpuCursorPair,
    tex_resize_nwse: GpuCursorPair,
    tex_resize_nesw: GpuCursorPair,
    tex_move: GpuCursorPair,
    tex_zoom_in: GpuCursorPair,
    tex_zoom_out: GpuCursorPair,
    tex_wait: GpuCursorPair,
    tex_help: GpuCursorPair,
    tex_unavailable: GpuCursorPair,

    perf_freq: i64,
    prev_counter: i64,
    _width: u32,
    _height: u32,
    offset_x: f32,
    offset_y: f32,
    animation_time: f32,
    frame_counter: u64,
    waitable_object: HANDLE,
}

impl DxgiPipeline {
    /// Creates and initializes the complete DirectX 11 / DirectComposition / Direct2D pipeline.
    pub fn new(hwnd_raw: *mut std::ffi::c_void, vx: i32, vy: i32, width: u32, height: u32) -> Result<Self> {
        let hwnd = HWND(hwnd_raw as _);
        unsafe {
            let mut perf_freq = 0i64;
            QueryPerformanceFrequency(&mut perf_freq)?;
            let mut prev_counter = 0i64;
            QueryPerformanceCounter(&mut prev_counter)?;

            let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_SINGLETHREADED;
            let mut device: Option<ID3D11Device> = None;
            let mut context: Option<ID3D11DeviceContext> = None;
            let mut feature_level = D3D_FEATURE_LEVEL_11_0;

            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                flags,
                Some(&[
                    D3D_FEATURE_LEVEL_11_1,
                    D3D_FEATURE_LEVEL_11_0,
                    D3D_FEATURE_LEVEL_10_1,
                ]),
                D3D11_SDK_VERSION,
                Some(&mut device),
                Some(&mut feature_level),
                Some(&mut context),
            )?;
            let device = device.ok_or_else(|| Error::from(E_FAIL))?;
            let context = context.ok_or_else(|| Error::from(E_FAIL))?;

            let dxgi_device: IDXGIDevice = device.cast()?;
            let dxgi_adapter: IDXGIAdapter = dxgi_device.GetAdapter()?;
            let dxgi_factory: IDXGIFactory2 = dxgi_adapter.GetParent()?;

            let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: width,
                Height: height,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                Stereo: BOOL(0),
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                Scaling: DXGI_SCALING_STRETCH,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
                Flags: DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32,
            };

            let swap_chain = dxgi_factory.CreateSwapChainForComposition(&device, &swap_chain_desc, None)?;
            let waitable_object = if let Ok(swap_chain2) = swap_chain.cast::<IDXGISwapChain2>() {
                let _ = swap_chain2.SetMaximumFrameLatency(1);
                swap_chain2.GetFrameLatencyWaitableObject()
            } else {
                HANDLE(std::ptr::null_mut())
            };

            let dcomp_device: IDCompositionDevice = DCompositionCreateDevice(Some(&dxgi_device))?;
            let dcomp_target = dcomp_device.CreateTargetForHwnd(hwnd, true)?;
            let dcomp_visual = dcomp_device.CreateVisual()?;
            dcomp_visual.SetContent(&swap_chain)?;
            dcomp_target.SetRoot(&dcomp_visual)?;
            dcomp_device.Commit()?;

            let d2d_factory: ID2D1Factory1 = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let d2d_device = d2d_factory.CreateDevice(&dxgi_device)?;
            let d2d_context = d2d_device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;

            let back_buffer: IDXGISurface = swap_chain.GetBuffer(0)?;

            let bitmap_props = D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                colorContext: std::mem::ManuallyDrop::new(None),
            };

            let d2d_target_bitmap = d2d_context.CreateBitmapFromDxgiSurface(&back_buffer, Some(&bitmap_props))?;
            d2d_context.SetTarget(&d2d_target_bitmap);
            d2d_context.SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);

            // Upload all pixel-perfect textures to GPU memory
            let tex_arrow = create_gpu_pair(&d2d_context, &WHITE_ARROW_PIXELS, &BLACK_ARROW_PIXELS, ARROW_HOTSPOT_X, ARROW_HOTSPOT_Y)?;
            let tex_hand = create_gpu_pair(&d2d_context, &WHITE_HAND_PIXELS, &BLACK_HAND_PIXELS, HAND_HOTSPOT_X, HAND_HOTSPOT_Y)?;
            let tex_ibeam = create_gpu_pair(&d2d_context, &WHITE_IBEAM_PIXELS, &BLACK_IBEAM_PIXELS, IBEAM_HOTSPOT_X, IBEAM_HOTSPOT_Y)?;
            let tex_crosshair = create_gpu_pair(&d2d_context, &WHITE_CROSSHAIR_PIXELS, &BLACK_CROSSHAIR_PIXELS, CROSSHAIR_HOTSPOT_X, CROSSHAIR_HOTSPOT_Y)?;
            let tex_resize_ns = create_gpu_pair(&d2d_context, &WHITE_RESIZE_NS_PIXELS, &BLACK_RESIZE_NS_PIXELS, RESIZE_NS_HOTSPOT_X, RESIZE_NS_HOTSPOT_Y)?;
            let tex_resize_we = create_gpu_pair(&d2d_context, &WHITE_RESIZE_WE_PIXELS, &BLACK_RESIZE_WE_PIXELS, RESIZE_WE_HOTSPOT_X, RESIZE_WE_HOTSPOT_Y)?;
            let tex_resize_nwse = create_gpu_pair(&d2d_context, &WHITE_RESIZE_NWSE_PIXELS, &BLACK_RESIZE_NWSE_PIXELS, RESIZE_NWSE_HOTSPOT_X, RESIZE_NWSE_HOTSPOT_Y)?;
            let tex_resize_nesw = create_gpu_pair(&d2d_context, &WHITE_RESIZE_NESW_PIXELS, &BLACK_RESIZE_NESW_PIXELS, RESIZE_NESW_HOTSPOT_X, RESIZE_NESW_HOTSPOT_Y)?;
            let tex_move = create_gpu_pair(&d2d_context, &WHITE_MOVE_PIXELS, &BLACK_MOVE_PIXELS, MOVE_HOTSPOT_X, MOVE_HOTSPOT_Y)?;
            let tex_zoom_in = create_gpu_pair(&d2d_context, &WHITE_ZOOM_IN_PIXELS, &BLACK_ZOOM_IN_PIXELS, ZOOM_IN_HOTSPOT_X, ZOOM_IN_HOTSPOT_Y)?;
            let tex_zoom_out = create_gpu_pair(&d2d_context, &WHITE_ZOOM_OUT_PIXELS, &BLACK_ZOOM_OUT_PIXELS, ZOOM_OUT_HOTSPOT_X, ZOOM_OUT_HOTSPOT_Y)?;
            let tex_wait = create_gpu_pair(&d2d_context, &WHITE_WAIT_PIXELS, &BLACK_WAIT_PIXELS, WAIT_HOTSPOT_X, WAIT_HOTSPOT_Y)?;
            let tex_help = create_gpu_pair(&d2d_context, &WHITE_HELP_PIXELS, &BLACK_HELP_PIXELS, HELP_HOTSPOT_X, HELP_HOTSPOT_Y)?;
            let tex_unavailable = create_gpu_pair(&d2d_context, &WHITE_UNAVAILABLE_PIXELS, &BLACK_UNAVAILABLE_PIXELS, UNAVAILABLE_HOTSPOT_X, UNAVAILABLE_HOTSPOT_Y)?;

            Ok(Self {
                hwnd,
                _device: device,
                _context: context,
                swap_chain,
                _dcomp_device: dcomp_device,
                _dcomp_target: dcomp_target,
                _dcomp_visual: dcomp_visual,
                _d2d_factory: d2d_factory,
                d2d_context,
                _d2d_target_bitmap: d2d_target_bitmap,
                tex_arrow,
                tex_hand,
                tex_ibeam,
                tex_crosshair,
                tex_resize_ns,
                tex_resize_we,
                tex_resize_nwse,
                tex_resize_nesw,
                tex_move,
                tex_zoom_in,
                tex_zoom_out,
                tex_wait,
                tex_help,
                tex_unavailable,
                perf_freq,
                prev_counter,
                _width: width,
                _height: height,
                offset_x: vx as f32,
                offset_y: vy as f32,
                animation_time: 0.0,
                frame_counter: 0,
                waitable_object,
            })
        }
    }

    /// Primary Hardware VBLANK Render Tick.
    ///
    /// Computes delta time using QPC, updates spring physics, renders the high-fidelity cursor
    /// via Direct2D on GPU, and synchronizes strictly to monitor VBLANK with `Present(1, 0)`.
    pub fn run_tick(
        &mut self,
        physics: &mut SmoothCursorPhysics,
        target_pos: Vec2,
        cursor_kind: RenderCursorKind,
        theme_blend: f32,
        is_clicking: bool,
    ) -> Result<()> {
        unsafe {
            // 0. Ultra-low latency synchronization via DXGI waitable object (1-frame queue)
            // Synchronizes directly with physical monitor VBLANK (144 Hz = 6.94ms, 240 Hz = 4.16ms, 360 Hz = 2.77ms)
            if !self.waitable_object.is_invalid() {
                let _ = WaitForSingleObject(self.waitable_object, 1000);
            }

            // 1. Precise QPC delta calculation
            let mut current_counter = 0i64;
            QueryPerformanceCounter(&mut current_counter)?;
            let mut delta_time = (current_counter - self.prev_counter) as f32 / self.perf_freq as f32;
            self.prev_counter = current_counter;

            // Clamp delta on abnormal pauses or lag spikes (default to 144Hz fallback dt)
            if delta_time <= 0.0 || delta_time > 0.05 {
                delta_time = 1.0 / 144.0;
            }
            self.animation_time += delta_time;

            // 2. Solve Spring Physics (Semi-implicit Euler + Squash & Stretch)
            physics.update(target_pos, delta_time);

            // 3. Reinforce HWND_TOPMOST priority periodically over Taskbar and Shell
            if self.frame_counter % 120 == 0 {
                let _ = SetWindowPos(
                    self.hwnd,
                    HWND_TOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_SHOWWINDOW,
                );
            }
            self.frame_counter = self.frame_counter.wrapping_add(1);

            // 4. Begin Direct2D Hardware Drawing
            self.d2d_context.BeginDraw();
            self.d2d_context.Clear(Some(&D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }));

            // 5. Draw High-Fidelity GPU Textured Cursor
            self.draw_hardware_cursor(
                physics.position,
                physics.angle,
                physics.scale_x,
                physics.scale_y,
                cursor_kind,
                theme_blend,
                is_clicking,
            )?;

            self.d2d_context.EndDraw(None, None)?;

            // 6. Hardware VBLANK synchronized Present
            // SyncInterval = 1 -> Locks directly to monitor's physical VBLANK interrupt!
            // 144 Hz display -> blocks ~6.94 ms
            // 240 Hz display -> blocks ~4.16 ms
            // 60 Hz display  -> blocks ~16.66 ms
            self.swap_chain.Present(1, DXGI_PRESENT(0)).ok()?;

            Ok(())
        }
    }

    /// Clears the DirectComposition swapchain to completely transparent black.
    pub fn clear(&mut self) -> Result<()> {
        unsafe {
            self.d2d_context.BeginDraw();
            self.d2d_context.Clear(Some(&D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }));
            self.d2d_context.EndDraw(None, None)?;
            self.swap_chain.Present(1, DXGI_PRESENT(0)).ok()?;
            Ok(())
        }
    }

    /// Draws the high-fidelity cursor texture on the GPU with bilinear interpolation,
    /// soft ambient projection shadow, and dynamic White <-> Black theme morphing.
    unsafe fn draw_hardware_cursor(
        &self,
        pos: Vec2,
        angle: f32,
        scale_x: f32,
        scale_y: f32,
        kind: RenderCursorKind,
        theme_blend: f32,
        is_clicking: bool,
    ) -> Result<()> {
        let (pair, is_arrow, is_hand) = match kind {
            RenderCursorKind::Arrow => (&self.tex_arrow, true, false),
            RenderCursorKind::Hand => (&self.tex_hand, false, true),
            RenderCursorKind::IBeam => (&self.tex_ibeam, false, false),
            RenderCursorKind::Crosshair => (&self.tex_crosshair, false, false),
            RenderCursorKind::ResizeNS => (&self.tex_resize_ns, false, false),
            RenderCursorKind::ResizeWE => (&self.tex_resize_we, false, false),
            RenderCursorKind::ResizeNWSE => (&self.tex_resize_nwse, false, false),
            RenderCursorKind::ResizeNESW => (&self.tex_resize_nesw, false, false),
            RenderCursorKind::Move => (&self.tex_move, false, false),
            RenderCursorKind::ZoomIn => (&self.tex_zoom_in, false, false),
            RenderCursorKind::ZoomOut => (&self.tex_zoom_out, false, false),
            RenderCursorKind::Wait => (&self.tex_wait, false, false),
            RenderCursorKind::Help => (&self.tex_help, false, false),
            RenderCursorKind::Unavailable => (&self.tex_unavailable, false, false),
        };

        let local_x = pos.x - self.offset_x;
        let local_y = pos.y - self.offset_y;

        // Base scale: 0.50 maps the 64x64 source texture to a sharp ~32px cursor
        let (draw_angle, sx, sy) = if is_arrow {
            // In the source sprite texture, the arrow points UP (-PI/2).
            // Adding PI/2 aligns the arrow tip directly with the velocity trajectory (angle) in full 360 degrees.
            // At canonical resting pose (-3*PI/4), draw_angle is -PI/4 (-45 deg, standard top-left).
            let draw_angle = angle + std::f32::consts::FRAC_PI_2;
            let base_scale = 0.50;
            // Stretch along the arrow's length (Y in sprite space) and squash along width (X in sprite space)
            (draw_angle, scale_y * base_scale, scale_x * base_scale)
        } else if is_hand {
            // Hand pointer always stays upright! Subtle click squash on button down
            let click_squash = if is_clicking { 0.90 } else { 1.0 };
            let base_scale = 0.48 * click_squash;
            (0.0f32, base_scale, base_scale)
        } else {
            // Text, resize, and utility cursors maintain fixed orientation
            let base_scale = 0.50;
            (0.0f32, base_scale, base_scale)
        };

        // 1. Draw Soft Drop Projection Shadow (offset +1.5, +2.5) with opacity 0.28
        let shadow_matrix = make_affine_matrix(
            local_x + 1.5,
            local_y + 2.5,
            draw_angle,
            sx,
            sy,
            pair.hotspot_x,
            pair.hotspot_y,
        );
        self.d2d_context.SetTransform(&shadow_matrix);
        self.d2d_context.DrawBitmap(
            &pair.black,
            None,
            0.28,
            D2D1_INTERPOLATION_MODE_LINEAR,
            None,
            None,
        );

        // 2. Draw Main Crisp Cursor Body with Dynamic Theme Blending
        let body_matrix = make_affine_matrix(
            local_x,
            local_y,
            draw_angle,
            sx,
            sy,
            pair.hotspot_x,
            pair.hotspot_y,
        );
        self.d2d_context.SetTransform(&body_matrix);

        let t = theme_blend.clamp(0.0, 1.0);
        let inv_t = 1.0 - t;

        if inv_t > 0.01 {
            self.d2d_context.DrawBitmap(
                &pair.white,
                None,
                inv_t,
                D2D1_INTERPOLATION_MODE_LINEAR,
                None,
                None,
            );
        }

        if t > 0.01 {
            self.d2d_context.DrawBitmap(
                &pair.black,
                None,
                t,
                D2D1_INTERPOLATION_MODE_LINEAR,
                None,
                None,
            );
        }

        // Reset transform to identity
        self.d2d_context.SetTransform(&Matrix3x2 {
            M11: 1.0, M12: 0.0,
            M21: 0.0, M22: 1.0,
            M31: 0.0, M32: 0.0,
        });

        Ok(())
    }
}

unsafe fn create_gpu_pair(
    context: &ID2D1DeviceContext,
    white_pixels: &[u32; 64 * 64],
    black_pixels: &[u32; 64 * 64],
    hotspot_x: f32,
    hotspot_y: f32,
) -> Result<GpuCursorPair> {
    let props = D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
        },
        dpiX: 96.0,
        dpiY: 96.0,
        bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
        colorContext: std::mem::ManuallyDrop::new(None),
    };
    let size = D2D_SIZE_U {
        width: 64,
        height: 64,
    };
    let white = context.CreateBitmap(
        size,
        Some(white_pixels.as_ptr() as *const _),
        64 * 4,
        &props,
    )?;
    let black = context.CreateBitmap(
        size,
        Some(black_pixels.as_ptr() as *const _),
        64 * 4,
        &props,
    )?;
    Ok(GpuCursorPair {
        white,
        black,
        hotspot_x,
        hotspot_y,
    })
}

pub fn make_affine_matrix(
    pos_x: f32,
    pos_y: f32,
    angle_rad: f32,
    scale_x: f32,
    scale_y: f32,
    origin_x: f32,
    origin_y: f32,
) -> Matrix3x2 {
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    let m11 = scale_x * cos_a;
    let m12 = scale_x * sin_a;
    let m21 = -scale_y * sin_a;
    let m22 = scale_y * cos_a;

    let dx = pos_x - (m11 * origin_x + m21 * origin_y);
    let dy = pos_y - (m12 * origin_x + m22 * origin_y);

    Matrix3x2 {
        M11: m11,
        M12: m12,
        M21: m21,
        M22: m22,
        M31: dx,
        M32: dy,
    }
}
