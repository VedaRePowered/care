use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, LazyLock, OnceLock},
};

use parking_lot::RwLock;
use pollster::FutureExt;
use rusttype::gpu_cache::Cache as FontCache;
use wgpu::{Adapter, Buffer, Device, Instance, Queue, RenderPipeline, Surface};
use winit::window::WindowId;

use crate::math::{Mat3, Vec4};

use super::{CareRenderState, Font, LineEndStyle, LineJoinStyle, Texture, Vertex2d};

pub type WindowSurface = RwLock<(Surface<'static>, (u32, u32))>;

#[derive(Debug)]
pub(crate) struct GraphicsState {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub window_surfaces: HashMap<WindowId, WindowSurface>,
    pub render_pipeline_2d: RenderPipeline,
    pub vertex_buffer_2d: RwLock<Buffer>,
    pub index_buffer_2d: RwLock<Buffer>,
    pub bind_group_layout_2d: wgpu::BindGroupLayout,
    pub placeholder_texture: OnceLock<Texture>,
    pub care_render: RwLock<CareRenderState>,
    #[cfg(feature = "gui")]
    pub egui: crate::gui::EguiGraphics,
}

impl GraphicsState {
    pub(crate) fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        #[cfg(feature = "window")]
        let window_surfaces: HashMap<_, _> = crate::window::WINDOWS
            .read()
            .iter()
            .map(|win| {
                let win = win.clone();
                let size = (win.inner_size().width, win.inner_size().height);
                (
                    win.id(),
                    RwLock::new((
                        instance
                            .create_surface(win)
                            .expect("Failed to create surface for window."),
                        size,
                    )),
                )
            })
            .collect();
        #[cfg(not(feature = "window"))]
        let window_surfaces = HashMap::new();

        let adapter = {
            let surface = window_surfaces.values().next().map(|surf| surf.read());
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    force_fallback_adapter: false,
                    compatible_surface: surface.as_ref().map(|s| &s.0),
                })
                .block_on()
                .expect("No graphics adapter found")
        };
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Care render device"),
                    required_features: wgpu::Features::default()
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .block_on()
            .expect("No graphics device found in adapter");

        let mut surface_formats = HashMap::new();
        for (key, surf) in &window_surfaces {
            let surf = surf.read();
            let surface_caps = surf.0.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]);
            surface_formats.insert(key, surface_format);
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: surf.1 .0,
                height: surf.1 .1,
                present_mode: surface_caps.present_modes[0],
                desired_maximum_frame_latency: 10,
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
            };
            surf.0.configure(&device, &config);
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
            default_font: Font::new_from_bytes_and_id(
                include_bytes!("../assets/Urbanist-Regular.ttf"),
                1,
            ),
            next_font_id: 2,
            line_join_style: LineJoinStyle::Rounded,
            line_end_style: LineEndStyle::Rounded,
        };

        let (render_pipeline_2d, vertex_buffer_2d, index_buffer_2d, bind_group_layouts_2d, surface_format) = {
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
            // TODO: uhhh this is sometimes BGRA on some computers I have... I probably
            // should find a function that gives me the colour space of the surface
            let surface_format = surface_formats[&render.current_surface];
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("2D Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Vertex2d::descriptor()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
                cache: None,
            });
            (
                pipeline,
                RwLock::new(vertex_buffer),
                RwLock::new(index_buffer),
                textures_bind_group_layout,
                surface_format,
            )
        };

        #[cfg(feature = "gui")]
        let egui = crate::gui::EguiGraphics {
            egui_renderer: parking_lot::Mutex::new(egui_wgpu::Renderer::new(&device, surface_format, None, 1, false)),
            egui_ctx: egui::Context::default(),
            start_time: std::time::Instant::now(),
        };

        Self {
            instance,
            adapter,
            device: Arc::new(device),
            queue: Arc::new(queue),
            window_surfaces,
            render_pipeline_2d,
            vertex_buffer_2d,
            index_buffer_2d,
            bind_group_layout_2d: bind_group_layouts_2d,
            placeholder_texture: OnceLock::new(),
            care_render: RwLock::new(render),

            #[cfg(feature = "gui")]
            egui,
        }
    }
}

pub(crate) static GRAPHICS_STATE: LazyLock<GraphicsState> = LazyLock::new(GraphicsState::new);
