use std::fmt::Display;

use parking_lot::RwLock;
use wgpu::{Buffer, Device, Queue};

use crate::{
    graphics::LineJoinStyle,
    math::{IntoFl, Vec2, Vec4},
};

use super::{DrawCommand, DrawCommandData, GraphicsState, LineEndStyle, Texture, GRAPHICS_STATE};

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
        .care_render
        .read()
        .font_cache_texture
        .get_or_init(|| Texture::new_fill(1024, 1024, (0, 0, 0, 0)));
}

/// Set the colour used for rendering
pub fn set_colour(colour: impl Into<Vec4>) {
    GRAPHICS_STATE
        .get()
        .unwrap()
        .care_render
        .write()
        .current_colour = colour.into();
}

/// Set the colour used for rendering
pub fn set_line_style(join_style: LineJoinStyle, end_style: LineEndStyle) {
    let mut render = GRAPHICS_STATE
        .get()
        .unwrap()
        .care_render
        .write();
    render.line_join_style = join_style;
    render.line_end_style = end_style;
}

/// Render a line of text to the screen
pub fn text(text: impl Display, pos: impl Into<Vec2>) {
    let mut render = GRAPHICS_STATE
        .get()
        .expect("Graphics not initialized")
        .care_render
        .write();
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

/// Render a triangle (in a solid colour)
pub fn triangle(points: (impl Into<Vec2>, impl Into<Vec2>, impl Into<Vec2>)) {
    let mut render = GRAPHICS_STATE
        .get()
        .expect("Graphics not initialized")
        .care_render
        .write();
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
    let mut render = GRAPHICS_STATE
        .get()
        .expect("Graphics not initialized")
        .care_render
        .write();
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
    let mut render = GRAPHICS_STATE
        .get()
        .expect("Graphics not initialized")
        .care_render
        .write();
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
    let render = GRAPHICS_STATE
        .get()
        .expect("Graphics not initialized")
        .care_render
        .read();
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
    let mut render = GRAPHICS_STATE
        .get()
        .expect("Graphics not initialized")
        .care_render
        .write();
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
    let state = GRAPHICS_STATE.get().expect("Graphics not initialized");

    // Update font cache
    {
        let mut render = state.care_render.write();
        let texture = render.font_cache_texture.get().unwrap().clone();
        render
            .font_cache
            .cache_queued(|pos, data| {
                state.queue.write_texture(
                    wgpu::ImageCopyTexture {
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
                    wgpu::ImageDataLayout {
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
