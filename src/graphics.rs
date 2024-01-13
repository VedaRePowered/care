//! Graphics functions, all of which will panic if called from a thread that is not the main
//! thread, or if any function is called before calling [init()] from the main thread.

use std::{collections::HashMap, fmt::Display, sync::OnceLock};

use bytemuck::{Pod, Zeroable};
use parking_lot::RwLock;
use pollster::FutureExt;
use wgpu::{Adapter, Buffer, Device, Instance, Queue, RenderPipeline, Surface, VertexAttribute};
use winit::window::WindowId;

use crate::math::{Fl, IntoFl, Mat4, Vec2, Vec4};

#[derive(Debug)]
struct GraphicsState {
    _instance: Instance,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    window_surfaces: HashMap<WindowId, (Surface, (u32, u32))>,
    render_pipeline_2d: RenderPipeline,
    vertex_buffer_2d: RwLock<Buffer>,
    index_buffer_2d: RwLock<Buffer>,
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

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Vertex2d {
    position: [f32; 2],
    uv: [f32; 2],
    colour: [u8; 4],
    rounding: f32,
}

impl Vertex2d {
    fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: [VertexAttribute; 4] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Uint8x4, 3 => Float32];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRS,
        }
    }
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
        self.transform_stack.clear();
        self.current_transform = Mat4::ident();
        self.current_colour = Vec4::new(1, 1, 1, 1);
        self.commands.clear();
    }
    fn render(&self) -> (Vec<Vertex2d>, Vec<u32>) {
        let screen_size = Vec2::new(800, 600);
        let vert_pos = |x: Fl, y: Fl| [(x / screen_size.0.x) as f32, (y / screen_size.0.y) as f32];
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for command in &self.commands {
            let colour = [
                (command.colour.0.x * 255.99) as u8,
                (command.colour.0.y * 255.99) as u8,
                (command.colour.0.z * 255.99) as u8,
                (command.colour.0.w * 255.99) as u8,
            ];
            match command.data {
                DrawCommandData::Rect {
                    pos,
                    size,
                    rotation,
                    round_corners,
                } => {
                    let n = vertices.len() as u32;
                    vertices.push(Vertex2d {
                        position: vert_pos(pos.0.x, pos.0.y),
                        uv: [0.0, 0.0],
                        colour,
                        rounding: 0.0,
                    });
                    vertices.push(Vertex2d {
                        position: vert_pos(pos.0.x + size.0.x, pos.0.y),
                        uv: [1.0, 0.0],
                        colour,
                        rounding: 0.0,
                    });
                    vertices.push(Vertex2d {
                        position: vert_pos(pos.0.x, pos.0.y + size.0.y),
                        uv: [0.0, 1.0],
                        colour,
                        rounding: 0.0,
                    });
                    vertices.push(Vertex2d {
                        position: vert_pos(pos.0.x + size.0.x, pos.0.y + size.0.y),
                        uv: [1.0, 1.0],
                        colour,
                        rounding: 0.0,
                    });
                    indices.extend_from_slice(&[n, n + 1, n + 2, n + 2, n + 1, n + 3])
                }
                DrawCommandData::Triangle { verts } => todo!(),
                DrawCommandData::Circle {
                    center,
                    radius,
                    elipseness,
                } => todo!(),
            }
        }
        (vertices, indices)
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
                    (
                        unsafe { instance.create_surface(win) }
                            .expect("Failed to create surface for window."),
                        (win.inner_size().width, win.inner_size().height),
                    ),
                )
            })
            .collect();
        #[cfg(not(feature = "window"))]
        let window_surfaces = HashMap::new();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: window_surfaces.values().next().map(|(surf, _dims)| surf),
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

        for (_, (surface, dims)) in &window_surfaces {
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
                width: dims.0,
                height: dims.1,
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

        let (render_pipeline_2d, vertex_buffer_2d, index_buffer_2d) = {
            let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("2D Vertex Buffer"),
                size: 1024,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("2D Index Buffer"),
                size: 1024,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let shader = device.create_shader_module(wgpu::include_wgsl!("shader_2d.wgsl"));
            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("2D Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("2D Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex2d::descriptor()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        // TODO: uhhh
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });
            (
                pipeline,
                RwLock::new(vertex_buffer),
                RwLock::new(index_buffer),
            )
        };

        Self {
            _instance: instance,
            _adapter: adapter,
            device,
            queue,
            window_surfaces,
            render_pipeline_2d,
            vertex_buffer_2d,
            index_buffer_2d,
            care_render: RwLock::new(render),
        }
    }
}

static GRAPHICS_STATE: OnceLock<GraphicsState> = OnceLock::new();

/// Initialize the graphics library, must be called on the main thread!
pub fn init() {
    GRAPHICS_STATE.get_or_init(|| GraphicsState::new());
}

pub fn set_colour(colour: impl Into<Vec4>) {
    GRAPHICS_STATE.get().unwrap().care_render.write().current_colour = colour.into();
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

fn upload_buffer(device: &Device, queue: &Queue, buffer_lock: &RwLock<Buffer>, data: &[u8]) {
    let mut buffer = buffer_lock.write();
    // TODO: Use map_async if possible
    if data.len() > buffer.size() as usize {
        let usage = buffer.usage();
        buffer.destroy();
        *buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Vertex Buffer"),
            size: data.len().next_power_of_two().max(1024) as u64,
            usage,
            mapped_at_creation: false,
        });
    }
    queue.write_buffer(&buffer, 0, data)
}

pub fn present() {
    // Lets try render some stuff oh boy!
    let state = GRAPHICS_STATE.get().expect("Graphics not initialized");
    let output = state
        .window_surfaces
        .values()
        .next()
        .unwrap()
        .0
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
    let (vertices, indices) = state.care_render.read().render();
    upload_buffer(
        &state.device,
        &state.queue,
        &state.vertex_buffer_2d,
        bytemuck::cast_slice(&vertices),
    );
    upload_buffer(
        &state.device,
        &state.queue,
        &state.index_buffer_2d,
        bytemuck::cast_slice(&indices),
    );
    let vert = state.vertex_buffer_2d.read();
    let idx = state.index_buffer_2d.read();
    {
        // Render pass time
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        render_pass.set_pipeline(&state.render_pipeline_2d);
        render_pass.set_vertex_buffer(0, vert.slice(..));
        render_pass.set_index_buffer(idx.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }
    state.queue.submit([encoder.finish()]);
    output.present();

    state.care_render.write().reset();
}
