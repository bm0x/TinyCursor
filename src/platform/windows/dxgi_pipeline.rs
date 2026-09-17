//! Modern DirectX 11 / DXGI Flip Model + DirectComposition + Direct2D Hardware Pipeline.
//!
//! Features:
//! - DXGI Modern Flip Model (`DXGI_SWAP_EFFECT_FLIP_DISCARD`) with premultiplied alpha.
//! - DirectComposition visual tree integration on Windows 10/11.
//! - Hardware VBLANK synchronized `Present(1, 0)` with zero Thread::sleep or spin_loop.
//! - QueryPerformanceCounter sub-microsecond delta calculation.
//! - Direct2D hardware-accelerated 3D volumetric cursor rendering with affine transforms.
//! - Real-time contrast adaptation (Pearl White / Obsidian Black with volumetric bevel).

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
};

use crate::core::math::Vec2;
use crate::core::physics::SmoothCursorPhysics;
use super::renderer::RenderCursorKind;

pub struct DxgiPipeline {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain1,
    dcomp_device: IDCompositionDevice,
    dcomp_target: IDCompositionTarget,
    dcomp_visual: IDCompositionVisual,
    d2d_factory: ID2D1Factory1,
    d2d_context: ID2D1DeviceContext,
    d2d_target_bitmap: ID2D1Bitmap1,
    
    // Cached Path Geometries
    geo_arrow: ID2D1PathGeometry1,
    geo_hand: ID2D1PathGeometry1,
    geo_ibeam: ID2D1PathGeometry1,
    geo_resize_ns: ID2D1PathGeometry1,
    geo_resize_we: ID2D1PathGeometry1,
    geo_move: ID2D1PathGeometry1,
    geo_zoom_in: ID2D1PathGeometry1,
    geo_zoom_out: ID2D1PathGeometry1,

    perf_freq: i64,
    prev_counter: i64,
    width: u32,
    height: u32,
    offset_x: f32,
    offset_y: f32,
    animation_time: f32,
}

impl DxgiPipeline {
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
                Flags: 0,
            };

            let swap_chain = dxgi_factory.CreateSwapChainForComposition(&device, &swap_chain_desc, None)?;

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

            // Pre-compile cached 3D volumetric cursor path geometries
            let geo_arrow = create_arrow_geometry(&d2d_factory)?;
            let geo_hand = create_hand_geometry(&d2d_factory)?;
            let geo_ibeam = create_ibeam_geometry(&d2d_factory)?;
            let geo_resize_ns = create_resize_geometry(&d2d_factory, false)?;
            let geo_resize_we = create_resize_geometry(&d2d_factory, true)?;
            let geo_move = create_move_geometry(&d2d_factory)?;
            let geo_zoom_in = create_zoom_geometry(&d2d_factory, true)?;
            let geo_zoom_out = create_zoom_geometry(&d2d_factory, false)?;

            Ok(Self {
                device,
                context,
                swap_chain,
                dcomp_device,
                dcomp_target,
                dcomp_visual,
                d2d_factory,
                d2d_context,
                d2d_target_bitmap,
                geo_arrow,
                geo_hand,
                geo_ibeam,
                geo_resize_ns,
                geo_resize_we,
                geo_move,
                geo_zoom_in,
                geo_zoom_out,
                perf_freq,
                prev_counter,
                width,
                height,
                offset_x: vx as f32,
                offset_y: vy as f32,
                animation_time: 0.0,
            })
        }
    }

    /// Primary Hardware VBLANK Render Tick.
    ///
    /// Computes delta time using QPC, updates spring physics, renders the 3D volumetric cursor
    /// via Direct2D, and synchronizes to monitor VBLANK with `Present(1, 0)`.
    pub fn run_tick(
        &mut self,
        physics: &mut SmoothCursorPhysics,
        target_pos: Vec2,
        cursor_kind: RenderCursorKind,
        theme_blend: f32,
        is_clicking: bool,
    ) -> Result<()> {
        unsafe {
            // 1. Precise QPC delta calculation
            let mut current_counter = 0i64;
            QueryPerformanceCounter(&mut current_counter)?;
            let mut delta_time = (current_counter - self.prev_counter) as f32 / self.perf_freq as f32;
            self.prev_counter = current_counter;

            // Clamp delta on window move / pauses
            if delta_time <= 0.0 || delta_time > 0.05 {
                delta_time = 1.0 / 144.0;
            }
            self.animation_time += delta_time;

            // 2. Solve Spring Physics (Semi-implicit Euler + Squash & Stretch)
            physics.update(target_pos, delta_time);

            // 3. Begin Direct2D Hardware Drawing
            self.d2d_context.BeginDraw();
            self.d2d_context.Clear(Some(&D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }));

            // 4. Render Volumetric 3D Cursor with Drop Shadow and Bevel
            self.draw_volumetric_cursor(
                physics.position,
                physics.angle,
                physics.scale_x,
                physics.scale_y,
                cursor_kind,
                theme_blend,
                is_clicking,
            )?;

            self.d2d_context.EndDraw(None, None)?;

            // 5. Hardware VBLANK synchronized Present
            // SyncInterval = 1 -> Locks directly to monitor's physical VBLANK interrupt!
            // 144 Hz display -> blocks ~6.94 ms
            // 240 Hz display -> blocks ~4.16 ms
            // 60 Hz display -> blocks ~16.66 ms
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

    /// Draws the volumetric 3D cursor with smooth lighting and soft drop shadow.
    unsafe fn draw_volumetric_cursor(
        &self,
        pos: Vec2,
        angle: f32,
        scale_x: f32,
        scale_y: f32,
        kind: RenderCursorKind,
        theme_blend: f32,
        is_clicking: bool,
    ) -> Result<()> {
        let (geo, hotspot_x, hotspot_y) = match kind {
            RenderCursorKind::Arrow => (&self.geo_arrow, 3.0, 2.0),
            RenderCursorKind::Hand => (&self.geo_hand, 10.0, 2.0),
            RenderCursorKind::IBeam => (&self.geo_ibeam, 8.0, 13.0),
            RenderCursorKind::ResizeNS => (&self.geo_resize_ns, 12.0, 12.0),
            RenderCursorKind::ResizeWE => (&self.geo_resize_we, 12.0, 12.0),
            RenderCursorKind::ResizeNWSE | RenderCursorKind::ResizeNESW => (&self.geo_resize_ns, 12.0, 12.0),
            RenderCursorKind::Move => (&self.geo_move, 12.0, 12.0),
            RenderCursorKind::ZoomIn => (&self.geo_zoom_in, 10.0, 10.0),
            RenderCursorKind::ZoomOut => (&self.geo_zoom_out, 10.0, 10.0),
            RenderCursorKind::Crosshair => (&self.geo_move, 12.0, 12.0),
            RenderCursorKind::Wait | RenderCursorKind::Help | RenderCursorKind::Unavailable => (&self.geo_arrow, 3.0, 2.0),
        };

        // Theme colors (interpolating between Pearl White and Obsidian Black)
        let t = theme_blend.clamp(0.0, 1.0);
        let fill_color = D2D1_COLOR_F {
            r: lerp(0.98, 0.08, t),
            g: lerp(0.98, 0.08, t),
            b: lerp(1.00, 0.10, t),
            a: 0.98,
        };
        let bevel_color = D2D1_COLOR_F {
            r: lerp(0.85, 0.38, t),
            g: lerp(0.88, 0.38, t),
            b: lerp(0.92, 0.42, t),
            a: 0.95,
        };
        let shadow_color = D2D1_COLOR_F {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.35,
        };

        let fill_brush = self.d2d_context.CreateSolidColorBrush(&fill_color, None)?;
        let bevel_brush = self.d2d_context.CreateSolidColorBrush(&bevel_color, None)?;
        let shadow_brush = self.d2d_context.CreateSolidColorBrush(&shadow_color, None)?;

        // Additional click squash
        let click_scale = if is_clicking { 0.88 } else { 1.0 };
        let sx = scale_x * click_scale;
        let sy = scale_y * click_scale;

        let local_x = pos.x - self.offset_x;
        let local_y = pos.y - self.offset_y;

        // 1. Draw Drop Shadow (3px offset along Y, soft projection)
        let shadow_matrix = make_affine_matrix(
            local_x + 2.5,
            local_y + 3.5,
            angle,
            sx * 1.04,
            sy * 1.04,
            hotspot_x,
            hotspot_y,
        );
        self.d2d_context.SetTransform(&shadow_matrix);
        self.d2d_context.FillGeometry(geo, &shadow_brush, None);

        // 2. Draw Main Volumetric Body
        let body_matrix = make_affine_matrix(
            local_x,
            local_y,
            angle,
            sx,
            sy,
            hotspot_x,
            hotspot_y,
        );
        self.d2d_context.SetTransform(&body_matrix);
        self.d2d_context.FillGeometry(geo, &fill_brush, None);

        // 3. Draw Pronounced 3D Bevel Perimeter
        self.d2d_context.DrawGeometry(geo, &bevel_brush, 1.6, None);

        // Reset transform to identity
        self.d2d_context.SetTransform(&Matrix3x2 {
            M11: 1.0, M12: 0.0,
            M21: 0.0, M22: 1.0,
            M31: 0.0, M32: 0.0,
        });

        Ok(())
    }
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
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

// =========================================================================
// Geometry Constructors for Volumetric 3D Cursors (Magic Pointer Style)
// =========================================================================

unsafe fn create_arrow_geometry(factory: &ID2D1Factory1) -> Result<ID2D1PathGeometry1> {
    let geo = factory.CreatePathGeometry()?;
    let sink = geo.Open()?;

    // Arrow contour with rounded corners and curved wings
    sink.BeginFigure(D2D_POINT_2F { x: 3.0, y: 2.0 }, D2D1_FIGURE_BEGIN_FILLED);
    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
        point1: D2D_POINT_2F { x: 4.0, y: 10.0 },
        point2: D2D_POINT_2F { x: 3.5, y: 20.0 },
        point3: D2D_POINT_2F { x: 3.0, y: 28.0 },
    });
    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
        point1: D2D_POINT_2F { x: 4.5, y: 28.5 },
        point2: D2D_POINT_2F { x: 6.5, y: 25.0 },
        point3: D2D_POINT_2F { x: 9.0, y: 22.0 },
    });
    sink.AddLine(D2D_POINT_2F { x: 21.0, y: 22.0 });
    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
        point1: D2D_POINT_2F { x: 21.5, y: 20.5 },
        point2: D2D_POINT_2F { x: 14.0, y: 12.0 },
        point3: D2D_POINT_2F { x: 3.0, y: 2.0 },
    });
    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
    sink.Close()?;

    Ok(geo)
}

unsafe fn create_hand_geometry(factory: &ID2D1Factory1) -> Result<ID2D1PathGeometry1> {
    let geo = factory.CreatePathGeometry()?;
    let sink = geo.Open()?;

    sink.BeginFigure(D2D_POINT_2F { x: 10.0, y: 2.0 }, D2D1_FIGURE_BEGIN_FILLED);
    // Index finger left
    sink.AddLine(D2D_POINT_2F { x: 8.0, y: 14.0 });
    // Thumb
    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
        point1: D2D_POINT_2F { x: 4.0, y: 16.0 },
        point2: D2D_POINT_2F { x: 2.0, y: 19.0 },
        point3: D2D_POINT_2F { x: 4.0, y: 23.0 },
    });
    // Palm bottom
    sink.AddLine(D2D_POINT_2F { x: 7.0, y: 28.0 });
    sink.AddLine(D2D_POINT_2F { x: 18.0, y: 28.0 });
    // Knuckles (middle, ring, pinky)
    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
        point1: D2D_POINT_2F { x: 20.0, y: 23.0 },
        point2: D2D_POINT_2F { x: 19.0, y: 14.0 },
        point3: D2D_POINT_2F { x: 13.0, y: 14.0 },
    });
    // Index finger right
    sink.AddLine(D2D_POINT_2F { x: 13.0, y: 2.0 });
    // Index finger tip curve
    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
        point1: D2D_POINT_2F { x: 13.0, y: 0.5 },
        point2: D2D_POINT_2F { x: 10.0, y: 0.5 },
        point3: D2D_POINT_2F { x: 10.0, y: 2.0 },
    });
    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
    sink.Close()?;

    Ok(geo)
}

unsafe fn create_ibeam_geometry(factory: &ID2D1Factory1) -> Result<ID2D1PathGeometry1> {
    let geo = factory.CreatePathGeometry()?;
    let sink = geo.Open()?;

    sink.BeginFigure(D2D_POINT_2F { x: 3.0, y: 3.0 }, D2D1_FIGURE_BEGIN_FILLED);
    sink.AddLine(D2D_POINT_2F { x: 13.0, y: 3.0 });
    sink.AddLine(D2D_POINT_2F { x: 13.0, y: 6.0 });
    sink.AddLine(D2D_POINT_2F { x: 9.5, y: 6.0 });
    sink.AddLine(D2D_POINT_2F { x: 9.5, y: 20.0 });
    sink.AddLine(D2D_POINT_2F { x: 13.0, y: 20.0 });
    sink.AddLine(D2D_POINT_2F { x: 13.0, y: 23.0 });
    sink.AddLine(D2D_POINT_2F { x: 3.0, y: 23.0 });
    sink.AddLine(D2D_POINT_2F { x: 3.0, y: 20.0 });
    sink.AddLine(D2D_POINT_2F { x: 6.5, y: 20.0 });
    sink.AddLine(D2D_POINT_2F { x: 6.5, y: 6.0 });
    sink.AddLine(D2D_POINT_2F { x: 3.0, y: 6.0 });
    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
    sink.Close()?;

    Ok(geo)
}

unsafe fn create_resize_geometry(factory: &ID2D1Factory1, horizontal: bool) -> Result<ID2D1PathGeometry1> {
    let geo = factory.CreatePathGeometry()?;
    let sink = geo.Open()?;

    if horizontal {
        sink.BeginFigure(D2D_POINT_2F { x: 2.0, y: 12.0 }, D2D1_FIGURE_BEGIN_FILLED);
        sink.AddLine(D2D_POINT_2F { x: 7.0, y: 7.0 });
        sink.AddLine(D2D_POINT_2F { x: 7.0, y: 10.0 });
        sink.AddLine(D2D_POINT_2F { x: 17.0, y: 10.0 });
        sink.AddLine(D2D_POINT_2F { x: 17.0, y: 7.0 });
        sink.AddLine(D2D_POINT_2F { x: 22.0, y: 12.0 });
        sink.AddLine(D2D_POINT_2F { x: 17.0, y: 17.0 });
        sink.AddLine(D2D_POINT_2F { x: 17.0, y: 14.0 });
        sink.AddLine(D2D_POINT_2F { x: 7.0, y: 14.0 });
        sink.AddLine(D2D_POINT_2F { x: 7.0, y: 17.0 });
    } else {
        sink.BeginFigure(D2D_POINT_2F { x: 12.0, y: 2.0 }, D2D1_FIGURE_BEGIN_FILLED);
        sink.AddLine(D2D_POINT_2F { x: 7.0, y: 7.0 });
        sink.AddLine(D2D_POINT_2F { x: 10.0, y: 7.0 });
        sink.AddLine(D2D_POINT_2F { x: 10.0, y: 17.0 });
        sink.AddLine(D2D_POINT_2F { x: 7.0, y: 17.0 });
        sink.AddLine(D2D_POINT_2F { x: 12.0, y: 22.0 });
        sink.AddLine(D2D_POINT_2F { x: 17.0, y: 17.0 });
        sink.AddLine(D2D_POINT_2F { x: 14.0, y: 17.0 });
        sink.AddLine(D2D_POINT_2F { x: 14.0, y: 7.0 });
        sink.AddLine(D2D_POINT_2F { x: 17.0, y: 7.0 });
    }
    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
    sink.Close()?;

    Ok(geo)
}

unsafe fn create_move_geometry(factory: &ID2D1Factory1) -> Result<ID2D1PathGeometry1> {
    let geo = factory.CreatePathGeometry()?;
    let sink = geo.Open()?;

    sink.BeginFigure(D2D_POINT_2F { x: 12.0, y: 2.0 }, D2D1_FIGURE_BEGIN_FILLED);
    sink.AddLine(D2D_POINT_2F { x: 9.0, y: 6.0 });
    sink.AddLine(D2D_POINT_2F { x: 11.0, y: 6.0 });
    sink.AddLine(D2D_POINT_2F { x: 11.0, y: 9.0 });
    sink.AddLine(D2D_POINT_2F { x: 6.0, y: 9.0 });
    sink.AddLine(D2D_POINT_2F { x: 6.0, y: 7.0 });
    sink.AddLine(D2D_POINT_2F { x: 2.0, y: 12.0 });
    sink.AddLine(D2D_POINT_2F { x: 6.0, y: 17.0 });
    sink.AddLine(D2D_POINT_2F { x: 6.0, y: 15.0 });
    sink.AddLine(D2D_POINT_2F { x: 11.0, y: 15.0 });
    sink.AddLine(D2D_POINT_2F { x: 11.0, y: 18.0 });
    sink.AddLine(D2D_POINT_2F { x: 9.0, y: 18.0 });
    sink.AddLine(D2D_POINT_2F { x: 12.0, y: 22.0 });
    sink.AddLine(D2D_POINT_2F { x: 15.0, y: 18.0 });
    sink.AddLine(D2D_POINT_2F { x: 13.0, y: 18.0 });
    sink.AddLine(D2D_POINT_2F { x: 13.0, y: 15.0 });
    sink.AddLine(D2D_POINT_2F { x: 18.0, y: 15.0 });
    sink.AddLine(D2D_POINT_2F { x: 18.0, y: 17.0 });
    sink.AddLine(D2D_POINT_2F { x: 22.0, y: 12.0 });
    sink.AddLine(D2D_POINT_2F { x: 18.0, y: 7.0 });
    sink.AddLine(D2D_POINT_2F { x: 18.0, y: 9.0 });
    sink.AddLine(D2D_POINT_2F { x: 13.0, y: 9.0 });
    sink.AddLine(D2D_POINT_2F { x: 13.0, y: 6.0 });
    sink.AddLine(D2D_POINT_2F { x: 15.0, y: 6.0 });
    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
    sink.Close()?;

    Ok(geo)
}

unsafe fn create_zoom_geometry(factory: &ID2D1Factory1, _is_in: bool) -> Result<ID2D1PathGeometry1> {
    let geo = factory.CreatePathGeometry()?;
    let sink = geo.Open()?;

    // Circular lens rim
    sink.BeginFigure(D2D_POINT_2F { x: 10.0, y: 2.0 }, D2D1_FIGURE_BEGIN_FILLED);
    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
        point1: D2D_POINT_2F { x: 15.5, y: 2.0 },
        point2: D2D_POINT_2F { x: 18.0, y: 4.5 },
        point3: D2D_POINT_2F { x: 18.0, y: 10.0 },
    });
    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
        point1: D2D_POINT_2F { x: 18.0, y: 15.5 },
        point2: D2D_POINT_2F { x: 15.5, y: 18.0 },
        point3: D2D_POINT_2F { x: 10.0, y: 18.0 },
    });
    // Handle
    sink.AddLine(D2D_POINT_2F { x: 16.0, y: 24.0 });
    sink.AddLine(D2D_POINT_2F { x: 19.0, y: 21.0 });
    sink.AddLine(D2D_POINT_2F { x: 14.0, y: 16.0 });
    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
        point1: D2D_POINT_2F { x: 4.5, y: 18.0 },
        point2: D2D_POINT_2F { x: 2.0, y: 15.5 },
        point3: D2D_POINT_2F { x: 2.0, y: 10.0 },
    });
    sink.AddBezier(&D2D1_BEZIER_SEGMENT {
        point1: D2D_POINT_2F { x: 2.0, y: 4.5 },
        point2: D2D_POINT_2F { x: 4.5, y: 2.0 },
        point3: D2D_POINT_2F { x: 10.0, y: 2.0 },
    });
    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
    sink.Close()?;

    Ok(geo)
}
