//! Graphics functions, all of which will panic if called from a thread that is not the main
//! thread, or if any function is called before calling [init()] from the main thread.

use std::{collections::HashMap, fmt::Display, sync::OnceLock};

use parking_lot::RwLock;
use pollster::FutureExt;
use wgpu::{Adapter, Device, Instance, Queue, Surface};
use winit::window::WindowId;

use crate::math::{Fl, IntoFl, Mat4, Vec2, Vec4};

#[derive(Debug)]
struct GraphicsState {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    window_surfaces: HashMap<WindowId, Surface>,
    care_render: RwLock<CareRenderState>,
}

#[derive(Debug)]
enum DrawCommandData {
    Rect {
        pos: Vec2,
        size: Vec2,
        rotation: Fl,
        round_corners: [Fl; 4],
    },
    Triangle {
        verts: [Vec2; 3],
    },
    Circle {
        center: Vec2,
        radius: Fl,
        elipseness: Vec2,
    },
}

#[derive(Debug)]
struct DrawCommand {
    transform: Mat4,
    colour: Vec4,
    data: DrawCommandData,
}

#[derive(Debug)]
struct CareRenderState {
    transform_stack: Vec<Mat4>,
    current_transform: Mat4,
    current_colour: Vec4,
    current_surface: WindowId,
    commands: Vec<DrawCommand>,
}

impl CareRenderState {
    fn reset(&mut self) {
        self.transform_stack = vec![];
        self.current_transform = Mat4::ident();
        self.current_colour = Vec4::new(1, 1, 1, 1);
    }
}

impl GraphicsState {
    fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        #[cfg(feature = "window")]
        let window_surfaces: HashMap<_, _> = crate::window::WINDOWS
            .read()
            .iter()
            .map(|win| {
                (
                    win.id(),
                    unsafe { instance.create_surface(win) }
                        .expect("Failed to create surface for window."),
                )
            })
            .collect();
        #[cfg(not(feature = "window"))]
        let window_surfaces = HashMap::new();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: window_surfaces.values().next(),
            })
            .block_on()
            .expect("No graphics adapter found");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Care render device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .block_on()
            .expect("No graphics device found in adapter");

        for (_, surface) in &window_surfaces {
            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .copied()
                .filter(|f| f.is_srgb())
                .next()
                .unwrap_or(surface_caps.formats[0]);
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: 800, // TODO: don't be lazy veda
                height: 600,
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
            };
            surface.configure(&device, &config);
        }

        let render = CareRenderState {
            transform_stack: Vec::new(),
            current_transform: Mat4::ident(),
            current_colour: Vec4::new(1, 1, 1, 1),
            // TODO: How do render textures / canvases relate to surfaces?
            current_surface: *window_surfaces.keys().next().unwrap(),
            commands: Vec::new(),
        };

        Self {
            instance,
            adapter,
            device,
            queue,
            window_surfaces,
            care_render: RwLock::new(render),
        }
    }
}

static GRAPHICS_STATE: OnceLock<GraphicsState> = OnceLock::new();

/// Initialize the graphics library, must be called on the main thread!
pub fn init() {
    GRAPHICS_STATE.get_or_init(|| GraphicsState::new());
}

pub fn text(text: impl Display, pos: impl Into<Vec2>) {}

pub fn rectangle(pos: impl Into<Vec2>, size: impl Into<Vec2>) {
    rectangle_rot(pos, size, 0.0)
}

pub fn rectangle_rot(pos: impl Into<Vec2>, size: impl Into<Vec2>, rotation: impl IntoFl) {
    rectangle_rounded(pos, size, rotation, [0.0; 4])
}

pub fn rectangle_rounded(
    pos: impl Into<Vec2>,
    size: impl Into<Vec2>,
    rotation: impl IntoFl,
    corner_radii: [impl IntoFl; 4],
) {
    let mut render = GRAPHICS_STATE
        .get()
        .expect("Graphics not initialized")
        .care_render
        .write();
    let command = DrawCommand {
        transform: render.current_transform.clone(),
        colour: render.current_colour,
        data: DrawCommandData::Rect {
            pos: pos.into(),
            size: size.into(),
            rotation: rotation.into_fl(),
            round_corners: corner_radii.map(|n| n.into_fl()),
        },
    };
    render.commands.push(command);
}

pub fn present() {
    // Lets try render some stuff oh boy!
    let state = GRAPHICS_STATE.get().expect("Graphics not initialized");
    let output = state
        .window_surfaces
        .values()
        .next()
        .unwrap()
        .get_current_texture()
        .unwrap();
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Present command encoder"),
        });
    {
        // Render pass time
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("2D Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
    }
    state.queue.submit([encoder.finish()]);
    output.present();
}
