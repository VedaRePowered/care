use std::{fmt::Display, time::Duration};

use parking_lot::RwLock;
use wgpu::{Buffer, Device, Queue};

use crate::{
    graphics::LineJoinStyle, math::{IntoFl, Vec2, Vec4}
};

use super::{DrawCommand, DrawCommandData, LineEndStyle, Texture, Vertex2d, GRAPHICS_STATE};

/// Initialize the graphics library, must be called on the main thread!
pub fn init() {
    GRAPHICS_STATE.placeholder_texture.get_or_init(|| {
        Texture::new_from_data(
            2,
            2,
            &[
                255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255,
            ],
        )
    });
    GRAPHICS_STATE
        .care_render
        .read()
        .font_cache_texture
        .get_or_init(|| Texture::new_fill(1024, 1024, (0, 0, 0, 0)));
}

/// Set the colour used for rendering
pub fn set_colour(colour: impl Into<Vec4>) {
    GRAPHICS_STATE.care_render.write().current_colour = colour.into();
}

/// Set the colour used for rendering
pub fn set_line_style(join_style: LineJoinStyle, end_style: LineEndStyle) {
    let mut render = GRAPHICS_STATE.care_render.write();
    render.line_join_style = join_style;
    render.line_end_style = end_style;
}

/// Render a line of text to the screen
pub fn text(text: impl Display, pos: impl Into<Vec2>) {
    let mut render = GRAPHICS_STATE.care_render.write();
    let pos = pos.into()
        + Vec2::new(
            0.0,
            render
                .default_font
                .0
                 .0
                .v_metrics(rusttype::Scale { x: 18.0, y: 18.0 })
                .ascent,
        );
    let text = text.to_string();
    let glyphs: Vec<_> = render
        .default_font
        .0
         .0
        .layout(
            &text,
            rusttype::Scale { x: 18.0, y: 18.0 },
            rusttype::Point {
                x: pos.x(),
                y: pos.y(),
            },
        )
        .collect();
    for glyph in glyphs {
        let font_id = render.default_font.0 .1;
        render
            .font_cache
            .queue_glyph(font_id as usize, glyph.clone());
        let command = DrawCommand {
            transform: render.current_transform.clone(),
            colour: render.current_colour,
            data: DrawCommandData::TextChar {
                glyph,
                font: render.default_font.0 .1,
            },
        };
        render.commands.push(command);
    }
}

#[inline(always)]
/// Render a texture
pub fn texture(tex: &Texture, pos: impl Into<Vec2>) {
    texture_scale(tex, pos, (1, 1))
}

#[inline(always)]
/// Render a texture, with custom scale
pub fn texture_scale(tex: &Texture, pos: impl Into<Vec2>, scale: impl Into<Vec2>) {
    texture_source(tex, pos, scale, (0, 0), tex.size())
}

#[inline(always)]
/// Render a texture, with custom scale, and source region
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
/// Render a texture, with custom scale, source region, and rotation
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

/// Render a texture with all settings
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
    let mut render = GRAPHICS_STATE.care_render.write();
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
/// Render a rectangle
pub fn rectangle(pos: impl Into<Vec2>, size: impl Into<Vec2>) {
    rectangle_rot(pos, size, 0.0)
}

#[inline(always)]
/// Render a rectangle, with a rotation
pub fn rectangle_rot(pos: impl Into<Vec2>, size: impl Into<Vec2>, rotation: impl IntoFl) {
    rectangle_rounded(pos, size, rotation, [0.0; 4])
}

/// Render a rectangle, with a rotation, and rounding corners
pub fn rectangle_rounded(
    pos: impl Into<Vec2>,
    size: impl Into<Vec2>,
    rotation: impl IntoFl,
    corner_radii: [impl IntoFl; 4],
) {
    let mut render = GRAPHICS_STATE.care_render.write();
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

#[inline(always)]
/// Render an outline of a rectangle
pub fn rectangle_line(pos: impl Into<Vec2>, size: impl Into<Vec2>, width: impl IntoFl) {
    rectangle_line_rot(pos.into(), size.into(), width.into_fl(), 0.0)
}

#[inline(always)]
/// Render an outline of a rectangle, with a rotation
pub fn rectangle_line_rot(
    pos: impl Into<Vec2>,
    size: impl Into<Vec2>,
    width: impl IntoFl,
    rotation: impl IntoFl,
) {
    let pos = pos.into();
    let size = size.into();
    let rot = rotation.into_fl();
    polyline(
        [
            pos,
            pos + Vec2::new(size.x, 0).rotated(rot),
            pos + size.rotated(rot),
            pos + Vec2::new(0, size.y).rotated(rot),
        ],
        width.into_fl(),
    )
}

/// Render a triangle (in a solid colour)
pub fn triangle(points: (impl Into<Vec2>, impl Into<Vec2>, impl Into<Vec2>)) {
    let mut render = GRAPHICS_STATE.care_render.write();
    let command = DrawCommand {
        transform: render.current_transform.clone(),
        colour: render.current_colour,
        data: DrawCommandData::Triangle {
            verts: [points.0.into(), points.1.into(), points.2.into()],
            tex_uvs: None,
        },
    };
    render.commands.push(command);
}

/// Render a triangle with a texture
pub fn triangle_textured(
    points: (impl Into<Vec2>, impl Into<Vec2>, impl Into<Vec2>),
    tex: &Texture,
    uvs: (impl Into<Vec2>, impl Into<Vec2>, impl Into<Vec2>),
) {
    let mut render = GRAPHICS_STATE.care_render.write();
    let command = DrawCommand {
        transform: render.current_transform.clone(),
        colour: render.current_colour,
        data: DrawCommandData::Triangle {
            verts: [points.0.into(), points.1.into(), points.2.into()],
            tex_uvs: Some((tex.clone(), [uvs.0.into(), uvs.1.into(), uvs.2.into()])),
        },
    };
    render.commands.push(command);
}

/// Render a circle
pub fn circle(center: impl Into<Vec2>, radius: impl IntoFl) {
    ellipse(center, radius, (0, 0))
}

/// Render a circle
pub fn ellipse(center: impl Into<Vec2>, radius: impl IntoFl, elipseness: impl Into<Vec2>) {
    let mut render = GRAPHICS_STATE.care_render.write();
    let command = DrawCommand {
        transform: render.current_transform.clone(),
        colour: render.current_colour,
        data: DrawCommandData::Circle {
            center: center.into(),
            radius: radius.into_fl(),
            elipseness: elipseness.into(),
        },
    };
    render.commands.push(command);
}

/// Draw a single line segment
pub fn line_segment(point1: impl Into<Vec2>, point2: impl Into<Vec2>, width: impl IntoFl) {
    line([point1.into(), point2.into()], width)
}

/// Draw a line with consistant width and line join style
pub fn line(points: impl IntoIterator<Item = impl Into<Vec2>>, width: impl IntoFl) {
    let (line_join_style, line_end_style) = {
        let render = GRAPHICS_STATE.care_render.read();
        (render.line_join_style, render.line_end_style)
    };
    let width = width.into_fl();
    line_varying_styles(
        points.into_iter().map(|pt| (pt, width, line_join_style)),
        (line_end_style, line_end_style),
    )
}

/// Draw a line with varying width or join style
pub fn line_varying_styles(
    points: impl IntoIterator<Item = (impl Into<Vec2>, impl IntoFl, LineJoinStyle)>,
    ends: (LineEndStyle, LineEndStyle),
) {
    let mut render = GRAPHICS_STATE.care_render.write();
    // Clippy detects this as an issue because when Fl = f32, the explicit conversions are not
    // needed, but when Fl = f64, they are neccesary.
    #[allow(clippy::unnecessary_cast, clippy::useless_conversion)]
    let command = DrawCommand {
        transform: render.current_transform.clone(),
        colour: render.current_colour,
        data: DrawCommandData::Line {
            points: points
                .into_iter()
                .map(|(p, w, j)| (p.into(), w.into_fl() as f32, j.into()))
                .collect(),
            ends,
        },
    };
    render.commands.push(command);
}

/// Draw a line with consistant width and line join style
pub fn polyline(points: impl IntoIterator<Item = impl Into<Vec2>>, width: impl IntoFl) {
    let mut render = GRAPHICS_STATE.care_render.write();
    // Clippy detects this as an issue because when Fl = f32, the explicit conversions are not
    // needed, but when Fl = f64, they are neccesary.
    #[allow(clippy::unnecessary_cast, clippy::useless_conversion)]
    let width = width.into_fl() as f32;
    let mut points = points.into_iter().map(|v| v.into());
    let start_points = [
        points.next().unwrap_or(Vec2::new(0, 0)),
        points.next().unwrap_or(Vec2::new(0, 0)),
    ];
    let extra_points = [
        start_points[0],
        start_points[0] + (start_points[1] - start_points[0]) / 256.0,
    ];
    let command = DrawCommand {
        transform: render.current_transform.clone(),
        colour: render.current_colour,
        data: DrawCommandData::Line {
            points: start_points
                .into_iter()
                .chain(points)
                .chain(extra_points)
                .map(|p| (p, width, render.line_join_style))
                .collect(),
            ends: (LineEndStyle::Flat, LineEndStyle::Flat),
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

/// Present the current frame
pub fn present() {
    // Lets try render some stuff oh boy!
    // Update font cache
    {
        let mut render = GRAPHICS_STATE.care_render.write();
        let texture = render.font_cache_texture.get().unwrap().clone();
        render
            .font_cache
            .cache_queued(|pos, data| {
                GRAPHICS_STATE.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture.0.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: pos.min.x,
                            y: pos.min.y,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    data.iter()
                        .flat_map(|&n| [255, 255, 255, n])
                        .collect::<Vec<_>>()
                        .as_slice(),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some((pos.max.x - pos.min.x) * 4),
                        rows_per_image: Some(pos.max.y - pos.min.y),
                    },
                    wgpu::Extent3d {
                        width: pos.max.x - pos.min.x,
                        height: pos.max.y - pos.min.y,
                        depth_or_array_layers: 1,
                    },
                )
            })
            .unwrap();
    }

    let output_key = GRAPHICS_STATE.window_surfaces.keys().next().unwrap();
    let output = GRAPHICS_STATE.window_surfaces[output_key]
        .read()
        .0
        .get_current_texture();
    let output = if let Ok(output) = output {
        output
    } else {
        // Output is outdated, request a new surface...
        let windows = crate::window::WINDOWS.read();
        let win = windows
            .iter()
            .find(|w| w.id() == *output_key)
            .cloned()
            .unwrap();
        let size = (win.inner_size().width, win.inner_size().height);
        let mut output = GRAPHICS_STATE.window_surfaces[output_key].write();
        *output = (
            GRAPHICS_STATE
                .instance
                .create_surface(win)
                .expect("Failed to create surface for window."),
            size,
        );

        // Configure the new surface
        let surface_caps = output.0.get_capabilities(&GRAPHICS_STATE.adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: output.1 .0,
            height: output.1 .1,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 10,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        output.0.configure(&GRAPHICS_STATE.device, &config);

        output.0.get_current_texture().unwrap()
    };

    let screen_size = output.texture.size();
    let screen_size = Vec2::new(screen_size.width, screen_size.height);

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder =
        GRAPHICS_STATE
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Present command encoder"),
            });

    let mut command_buffers = Vec::new();
    // Render egui
    #[cfg(feature = "gui")]
    let egui_data = {
        let mut egui_rend = GRAPHICS_STATE.egui.egui_renderer.lock();
        if let Some(full_output) = crate::gui::get_full_output() {
        let clipped_primitives = GRAPHICS_STATE
            .egui
            .egui_ctx
            .tessellate(full_output.shapes, 1.0);
        let egui_screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [output.texture.size().width, output.texture.size().height],
            pixels_per_point: 1.0,
        };
        let mut egui_command_buffers = egui_rend.update_buffers(
            &GRAPHICS_STATE.device,
            &GRAPHICS_STATE.queue,
            &mut encoder,
            &clipped_primitives,
            &egui_screen_descriptor,
        );
        command_buffers.append(&mut egui_command_buffers);
        for (tex, delta) in &full_output.textures_delta.set {
            egui_rend.update_texture(&GRAPHICS_STATE.device, &GRAPHICS_STATE.queue, *tex, delta);
        }
            Some((full_output.textures_delta, clipped_primitives, egui_screen_descriptor, egui_rend))
        } else {
            None
        }
    };

    // Render our stuff
    let max_textures = GRAPHICS_STATE.care_render.read().max_textures;
    let draw_calls = GRAPHICS_STATE.care_render.write().render(screen_size);
    let placeholder_tex = GRAPHICS_STATE.placeholder_texture.get().unwrap();
    let vertices: ForceAlign<Vec<Vertex2d>> = ForceAlign(
        draw_calls
            .iter()
            .flat_map(|v| &v.vertices)
            .cloned()
            .collect(),
    );
    let indices: ForceAlign<Vec<u32>> = ForceAlign(
        draw_calls
            .iter()
            .flat_map(|v| &v.indices)
            .cloned()
            .collect(),
    );
    upload_buffer(
        &GRAPHICS_STATE.device,
        &GRAPHICS_STATE.queue,
        &GRAPHICS_STATE.vertex_buffer_2d,
        bytemuck::cast_slice(&vertices.0),
    );
    upload_buffer(
        &GRAPHICS_STATE.device,
        &GRAPHICS_STATE.queue,
        &GRAPHICS_STATE.index_buffer_2d,
        bytemuck::cast_slice(&indices.0),
    );
    if vertices.0.is_empty() || indices.0.is_empty() {
        GRAPHICS_STATE.care_render.write().reset();
        return;
    }
    let mut vstart: wgpu::BufferAddress = 0;
    let mut istart: wgpu::BufferAddress = 0;
    let draw_call_info: Vec<_> = draw_calls
        .into_iter()
        .filter_map(|draw_call| {
            let vend = vstart
                + (draw_call.vertices.len() * std::mem::size_of::<Vertex2d>())
                    as wgpu::BufferAddress;
            let iend = istart
                + (draw_call.indices.len() * std::mem::size_of::<u32>()) as wgpu::BufferAddress;
            if vend == vstart || iend == istart {
                return None;
            }
            let bind_group = GRAPHICS_STATE
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Temp Bind Group"),
                    layout: &GRAPHICS_STATE.bind_group_layout_2d,
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
            let uwu = (
                vstart..vend,
                istart..iend,
                bind_group,
                draw_call.indices.len(),
            );
            vstart = vend;
            istart = iend;
            Some(uwu)
        })
        .collect();
    let vert = GRAPHICS_STATE.vertex_buffer_2d.read();
    let idx = GRAPHICS_STATE.index_buffer_2d.read();
    // Render pass time
    {
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
        for (vrange, irange, bind_group, indices_count) in draw_call_info {
            render_pass.set_pipeline(&GRAPHICS_STATE.render_pipeline_2d);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_vertex_buffer(0, vert.slice(vrange));
            render_pass.set_index_buffer(idx.slice(irange), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..indices_count as u32, 0, 0..1);
        }
    }
    // Egui render pass
    #[cfg(feature = "gui")]
    if let Some((textures_delta, clipped_primitives, egui_screen_descriptor, mut egui_rend)) = egui_data {
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("EGUI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            // This is fine maybe? idk it's needed for egui
            let mut render_pass = render_pass.forget_lifetime();
            egui_rend.render(
                &mut render_pass,
                &clipped_primitives,
                &egui_screen_descriptor,
            );
        }
        for id in &textures_delta.free {
            egui_rend.free_texture(id);
        }
    }

    command_buffers.push(encoder.finish());
    GRAPHICS_STATE.queue.submit(command_buffers);
    std::thread::sleep(Duration::from_millis(2));
    output.present();

    GRAPHICS_STATE.care_render.write().reset();
}

#[repr(C, align(256))]
struct ForceAlign<T>(T);
