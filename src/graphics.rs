//! Graphics functions, all of which will panic if called from a thread that is not the main
//! thread, or if any function is called before calling [init()] from the main thread.

use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{Debug, Display},
    fs,
    path::Path,
    sync::{Arc, OnceLock},
    time::Instant,
};

use bytemuck::{Pod, Zeroable};
use image::{DynamicImage, EncodableLayout};
use parking_lot::RwLock;
use pollster::FutureExt;
use rusttype::gpu_cache::Cache as FontCache;
use wgpu::{
    Adapter, Buffer, Device, Instance, Limits, Queue, RenderPipeline, Surface, VertexAttribute,
};
use winit::window::WindowId;

use crate::math::{Fl, IntoFl, Mat3, Vec2, Vec4};

#[derive(Debug)]
struct GraphicsState {
    _instance: Instance,
    _adapter: Adapter,
    device: Device,
    queue: Queue,
    limits: Limits,
    window_surfaces: HashMap<WindowId, (Surface, (u32, u32))>,
    render_pipeline_2d: RenderPipeline,
    vertex_buffer_2d: RwLock<Buffer>,
    index_buffer_2d: RwLock<Buffer>,
    bind_group_layout_2d: wgpu::BindGroupLayout,
    placeholder_texture: OnceLock<Texture>,
    care_render: RwLock<CareRenderState>,
}

#[derive(Debug, Clone)]
pub struct Texture(Arc<TextureHandle>);

impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Texture {
    pub fn new(filename: impl AsRef<Path>) -> Self {
        Self::new_from_image(image::io::Reader::open(filename).unwrap().decode().unwrap())
    }
    pub fn new_from_file_format(file_data: &[u8]) {
        todo!()
    }
    pub fn new_fill(width: u32, height: u32, colour: impl Into<Vec4>) -> Self {
        let c = colour.into() * 255.9;
        Self::new_from_data(
            width,
            height,
            (0..width * height)
                .flat_map(|_| [c.x() as u8, c.y() as u8, c.z() as u8, c.w() as u8])
                .collect::<Vec<_>>()
                .as_slice(),
        )
    }
    pub fn new_from_image(img: DynamicImage) -> Self {
        Self::new_from_data(img.width(), img.height(), img.to_rgba8().as_bytes())
    }
    pub fn new_from_data(width: u32, height: u32, data: &[u8]) -> Self {
        let state = GRAPHICS_STATE.get().expect("Graphics not initialized");
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = state.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        state.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Texture(Arc::new(TextureHandle {
            size: Vec2::new(width, height),
            texture,
            view,
            sampler,
        }))
    }
    pub fn upload_region(&self, data: &[u8], x: u32, y: u32, width: u32, height: u32) {
        let state = GRAPHICS_STATE.get().expect("Graphics not initialized");
        state.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.0.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
    pub fn size(&self) -> Vec2 {
        self.0.size
    }
}

#[derive(Debug)]
struct TextureHandle {
    size: Vec2,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl TextureHandle {
    fn bind_group_entries(&self, i: u32) -> [wgpu::BindGroupEntry<'_>; 2] {
        [
            wgpu::BindGroupEntry {
                binding: i * 2,
                resource: wgpu::BindingResource::TextureView(&self.view),
            },
            wgpu::BindGroupEntry {
                binding: i * 2 + 1,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
        ]
    }
}

#[derive(Debug)]
struct Font(Arc<rusttype::Font<'static>>);

impl Font {
    pub fn new(file: impl AsRef<Path>) -> Self {
        Font::new_from_vec(fs::read(file).unwrap())
    }
    pub fn new_from_vec(bytes: Vec<u8>) -> Self {
        Font(Arc::new(rusttype::Font::try_from_vec(bytes).unwrap()))
    }
    pub fn new_from_bytes(bytes: &'static [u8]) -> Self {
        Font(Arc::new(rusttype::Font::try_from_bytes(bytes).unwrap()))
    }
}

#[derive(Debug)]
enum LineJoinStyle {
    Miter,
    LimitedMiter,
    Bevel,
    Rounded,
    None,
}

#[derive(Debug)]
enum LineEndStyle {
    Flat,
    Point,
    Rounded,
}

#[derive(Debug)]
enum DrawCommandData {
    Rect {
        pos: Vec2,
        size: Vec2,
        rotation: Fl,
        corner_radii: [Fl; 4],
    },
    Triangle {
        verts: [Vec2; 3],
        tex_uvs: Option<(Texture, [Vec2; 3])>,
    },
    Circle {
        center: Vec2,
        radius: Fl,
        elipseness: Vec2,
    },
    Text {
        pos: Vec2,
        rotation: Fl,
        text: Cow<'static, str>,
    },
    Texture {
        texture: Texture,
        pos: Vec2,
        scale: Vec2,
        source: (Vec2, Vec2),
        rotation: Fl,
        corner_radii: [Fl; 4],
    },
    Line {
        points: Vec<(Vec2, Fl, LineJoinStyle)>,
        ends: (LineEndStyle, LineEndStyle),
    },
}

#[derive(Debug)]
struct DrawCommand {
    transform: Mat3,
    colour: Vec4,
    data: DrawCommandData,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
struct Vertex2d {
    position: [f32; 2],
    uv: [f32; 2],
    colour: [u8; 4],
    rounding: [f32; 2],
    tex: u32,
}

impl Vertex2d {
    fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: [VertexAttribute; 5] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Unorm8x4, 3 => Float32x2, 4 => Uint32];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRS,
        }
    }
}

struct CareRenderState {
    transform_stack: Vec<Mat3>,
    current_transform: Mat3,
    current_colour: Vec4,
    current_surface: WindowId,
    commands: Vec<DrawCommand>,
    max_textures: usize,
    font_cache: FontCache<'static>,
    font_cache_texture: OnceLock<Texture>,
    default_font: Font,
}

impl Debug for CareRenderState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CareRenderState")
            .field("transform_stack", &self.transform_stack)
            .field("current_transform", &self.current_transform)
            .field("current_colour", &self.current_colour)
            .field("current_surface", &self.current_surface)
            .field("commands", &self.commands)
            .field("max_textures", &self.max_textures)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default)]
struct DrawCall<T: bytemuck::Pod + Default> {
    vertices: Vec<T>,
    indices: Vec<u32>,
    textures: Vec<Texture>,
}

impl CareRenderState {
    fn reset(&mut self) {
        self.transform_stack.clear();
        self.current_transform = Mat3::ident();
        self.current_colour = Vec4::new(1, 1, 1, 1);
        self.commands.clear();
    }
    fn render(&mut self) -> Vec<DrawCall<Vertex2d>> {
        let screen_size = Vec2::new(800, 600);
        let vert_pos = |x: Fl, y: Fl| [(x / screen_size.0.x) as f32, (y / screen_size.0.y) as f32];
        let mut draw_calls = Vec::new();
        let mut cdc = DrawCall::default();
        for command in self.commands.drain(..) {
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
                    corner_radii: round_corners,
                } => {
                    let n = cdc.vertices.len() as u32;
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos(pos.0.x, pos.0.y),
                        uv: [0.0, 0.0],
                        colour,
                        rounding: [0.0, 0.0],
                        tex: 0,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos(pos.0.x + size.0.x, pos.0.y),
                        uv: [1.0, 0.0],
                        colour,
                        rounding: [0.0, 0.0],
                        tex: 0,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos(pos.0.x, pos.0.y + size.0.y),
                        uv: [0.0, 1.0],
                        colour,
                        rounding: [0.0, 0.0],
                        tex: 0,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos(pos.0.x + size.0.x, pos.0.y + size.0.y),
                        uv: [1.0, 1.0],
                        colour,
                        rounding: [0.0, 0.0],
                        tex: 0,
                    });
                    cdc.indices
                        .extend_from_slice(&[n, n + 1, n + 2, n + 2, n + 1, n + 3])
                }
                DrawCommandData::Texture {
                    texture,
                    pos,
                    scale,
                    source,
                    rotation,
                    corner_radii,
                } => {
                    let tex_size = texture.0.size;
                    let tex = if let Some(idx) = cdc.textures.iter().position(|t| t == &texture) {
                        idx
                    } else if cdc.textures.len() < self.max_textures {
                        let new_idx = cdc.textures.len();
                        cdc.textures.push(texture);
                        // offset by one because 0 represents no texture.
                        cdc.textures.len()
                    } else {
                        draw_calls.push(cdc);
                        cdc = DrawCall::default();
                        cdc.textures.push(texture);
                        cdc.textures.len()
                    } as u32;
                    let n = cdc.vertices.len() as u32;
                    let size = tex_size * scale;
                    let uv_base = source.0 / tex_size;
                    let uv_size = source.1 / tex_size;
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos(pos.0.x, pos.0.y),
                        uv: [uv_base.x(), uv_base.y()],
                        colour,
                        rounding: [0.0, 0.0],
                        tex,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos(pos.0.x + size.0.x, pos.0.y),
                        uv: [uv_base.x() + uv_size.x(), uv_base.y()],
                        colour,
                        rounding: [0.0, 0.0],
                        tex,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos(pos.0.x, pos.0.y + size.0.y),
                        uv: [uv_base.x(), uv_base.y() + uv_size.y()],
                        colour,
                        rounding: [0.0, 0.0],
                        tex,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos(pos.0.x + size.0.x, pos.0.y + size.0.y),
                        uv: [uv_base.x() + uv_size.x(), uv_base.y() + uv_size.y()],
                        colour,
                        rounding: [0.0, 0.0],
                        tex,
                    });
                    cdc.indices
                        .extend_from_slice(&[n, n + 1, n + 2, n + 2, n + 1, n + 3])
                }
                DrawCommandData::Text {
                    text,
                    pos,
                    rotation,
                } => {
                    todo!()
                }
                DrawCommandData::Triangle {
                    verts: _,
                    tex_uvs: _,
                } => todo!(),
                DrawCommandData::Circle {
                    center: _,
                    radius: _,
                    elipseness: _,
                } => todo!(),
                DrawCommandData::Line { points: _, ends: _ } => todo!(),
            }
        }
        draw_calls.push(cdc);
        draw_calls
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

        let limits = device.limits();

        let render = CareRenderState {
            transform_stack: Vec::new(),
            current_transform: Mat3::ident(),
            current_colour: Vec4::new(1, 1, 1, 1),
            // TODO: How do render textures / canvases relate to surfaces?
            current_surface: *window_surfaces.keys().next().unwrap(),
            commands: Vec::new(),
            max_textures: (limits.max_bindings_per_bind_group / 2)
                .min(limits.max_sampled_textures_per_shader_stage)
                .min(limits.max_samplers_per_shader_stage) as usize,
            font_cache: FontCache::builder().dimensions(1024, 1024).build(),
            font_cache_texture: OnceLock::new(),
            default_font: Font::new_from_bytes(include_bytes!("Urbanist-Regular.ttf")),
        };

        let (render_pipeline_2d, vertex_buffer_2d, index_buffer_2d, bind_group_layouts_2d) = {
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

            let textures_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("2D Textures Bind Group Layout"),
                    entries: (0..render.max_textures as u32)
                        .flat_map(|i| {
                            [
                                wgpu::BindGroupLayoutEntry {
                                    binding: i * 2,
                                    visibility: wgpu::ShaderStages::FRAGMENT,
                                    ty: wgpu::BindingType::Texture {
                                        multisampled: false,
                                        view_dimension: wgpu::TextureViewDimension::D2,
                                        sample_type: wgpu::TextureSampleType::Float {
                                            filterable: true,
                                        },
                                    },
                                    count: None,
                                },
                                wgpu::BindGroupLayoutEntry {
                                    binding: i * 2 + 1,
                                    visibility: wgpu::ShaderStages::FRAGMENT,
                                    ty: wgpu::BindingType::Sampler(
                                        wgpu::SamplerBindingType::Filtering,
                                    ),
                                    count: None,
                                },
                            ]
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                });

            let shader = device.create_shader_module(wgpu::include_wgsl!("shader_2d.wgsl"));
            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("2D Render Pipeline Layout"),
                    bind_group_layouts: &[&textures_bind_group_layout],
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
                textures_bind_group_layout,
            )
        };

        Self {
            _instance: instance,
            _adapter: adapter,
            limits,
            device,
            queue,
            window_surfaces,
            render_pipeline_2d,
            vertex_buffer_2d,
            index_buffer_2d,
            bind_group_layout_2d: bind_group_layouts_2d,
            placeholder_texture: OnceLock::new(),
            care_render: RwLock::new(render),
        }
    }
}

static GRAPHICS_STATE: OnceLock<GraphicsState> = OnceLock::new();

/// Initialize the graphics library, must be called on the main thread!
pub fn init() {
    let state = GRAPHICS_STATE.get_or_init(|| GraphicsState::new());
    state.placeholder_texture.get_or_init(|| {
        Texture::new_from_data(
            2,
            2,
            &[
                255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255,
            ],
        )
    });
    state
        .placeholder_texture
        .get_or_init(|| Texture::new_fill(1024, 1024, (0, 0, 0, 0)));
}

pub fn set_colour(colour: impl Into<Vec4>) {
    GRAPHICS_STATE
        .get()
        .unwrap()
        .care_render
        .write()
        .current_colour = colour.into();
}

pub fn text(text: impl Display, pos: impl Into<Vec2>) {
    let mut render = GRAPHICS_STATE
        .get()
        .expect("Graphics not initialized")
        .care_render
        .write();
    let pos = pos.into();
    let text = text.to_string();
    let glyphs: Vec<_> = render.default_font.0.layout(
        &text,
        rusttype::Scale { x: 18.0, y: 18.0 },
        rusttype::Point {
            x: pos.x(),
            y: pos.y(),
        },
    ).collect();
    for glyph in glyphs {
        render.font_cache.queue_glyph(0, glyph.clone());
        let rect = render.font_cache.rect_for(0, &glyph).unwrap().unwrap();
        let command = DrawCommand {
            transform: render.current_transform.clone(),
            colour: render.current_colour,
            data: DrawCommandData::TextRect {
                texture: render.font_cache_texture.get().unwrap().clone(),
                pos: Vec2::new(glyph.position().x, glyph.position().y),
                scale: Vec2::new(1, 1),
                source: (Vec2::new(rect.0.min.x, rect.0.min.y), Vec2::new(rect.0.max.x-rect.0.min.x, rect.0.max.y-rect.0.min.y)),
                rotation: 0.0,
                corner_radii: [0.0; 4],
            },
        };
        render.commands.push(command);
    }
}

#[inline(always)]
pub fn texture(tex: &Texture, pos: impl Into<Vec2>) {
    texture_scale(tex, pos, (1, 1))
}

#[inline(always)]
pub fn texture_scale(tex: &Texture, pos: impl Into<Vec2>, scale: impl Into<Vec2>) {
    texture_source(tex, pos, scale, (0, 0), tex.size())
}

#[inline(always)]
pub fn texture_source(
    tex: &Texture,
    pos: impl Into<Vec2>,
    scale: impl Into<Vec2>,
    source_pos: impl Into<Vec2>,
    source_size: impl Into<Vec2>,
) {
    texture_rot(tex, pos, scale, source_pos, source_size, 0)
}

#[inline(always)]
pub fn texture_rot(
    tex: &Texture,
    pos: impl Into<Vec2>,
    scale: impl Into<Vec2>,
    source_pos: impl Into<Vec2>,
    source_size: impl Into<Vec2>,
    rotation: impl IntoFl,
) {
    texture_rounded(
        tex,
        pos,
        scale,
        source_pos,
        source_size,
        rotation,
        [0, 0, 0, 0],
    )
}

pub fn texture_rounded(
    tex: &Texture,
    pos: impl Into<Vec2>,
    scale: impl Into<Vec2>,
    source_pos: impl Into<Vec2>,
    source_size: impl Into<Vec2>,
    rotation: impl IntoFl,
    corner_radii: [impl IntoFl; 4],
) {
    // TODO: Function to get this:
    let mut render = GRAPHICS_STATE
        .get()
        .expect("Graphics not initialized")
        .care_render
        .write();
    // TODO: Function to do this:
    let command = DrawCommand {
        transform: render.current_transform.clone(),
        colour: render.current_colour,
        data: DrawCommandData::Texture {
            texture: tex.clone(),
            pos: pos.into(),
            scale: scale.into(),
            source: (source_pos.into(), source_size.into()),
            rotation: rotation.into_fl(),
            corner_radii: corner_radii.map(|n| n.into_fl()),
        },
    };
    render.commands.push(command);
}

#[inline(always)]
pub fn rectangle(pos: impl Into<Vec2>, size: impl Into<Vec2>) {
    rectangle_rot(pos, size, 0.0)
}

#[inline(always)]
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
            corner_radii: corner_radii.map(|n| n.into_fl()),
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
    let max_textures = state.care_render.read().max_textures;
    let draw_calls = state.care_render.write().render();
    let placeholder_tex = state.placeholder_texture.get().unwrap();
    for draw_call in draw_calls {
        let inst = Instant::now();
        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Temp Bind Group"),
            layout: &state.bind_group_layout_2d,
            entries: (0..max_textures)
                .flat_map(|i| {
                    (if let Some(tex) = draw_call.textures.get(i) {
                        tex
                    } else {
                        placeholder_tex
                    })
                    .0
                    .bind_group_entries(i as u32)
                })
                .collect::<Vec<_>>()
                .as_slice(),
        });

        upload_buffer(
            &state.device,
            &state.queue,
            &state.vertex_buffer_2d,
            bytemuck::cast_slice(&draw_call.vertices),
        );
        upload_buffer(
            &state.device,
            &state.queue,
            &state.index_buffer_2d,
            bytemuck::cast_slice(&draw_call.indices),
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
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_vertex_buffer(0, vert.slice(..));
            render_pass.set_index_buffer(idx.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..draw_call.indices.len() as u32, 0, 0..1);
        }
    }
    state.queue.submit([encoder.finish()]);
    output.present();

    state.care_render.write().reset();
}
