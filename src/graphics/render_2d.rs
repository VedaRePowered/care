use std::{fmt::Debug, sync::OnceLock};

use bytemuck::{Pod, Zeroable};
use rusttype::{gpu_cache::Cache as FontCache, PositionedGlyph};
use wgpu::VertexAttribute;
use winit::window::WindowId;

use crate::{
    math::{Fl, Mat3, Vec2, Vec4},
    prelude::Mat2,
};

use super::{Font, Texture};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How to join lines together
pub enum LineJoinStyle {
    /// None/disconnected
    None,
    /// Merge the two lines points on the left and right
    Merge,
    /// Angled so it looks like both sides of the line meet where they logically would
    /// Limited at extreme angles to avoid weird visual glitches
    Miter,
    /// Just bevel to fill in the gap with a flat line
    Bevel,
    /// Rounded with a curve
    Rounded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// How to end lines
pub enum LineEndStyle {
    /// flattened
    Flat,
    /// Angled little point
    Point,
    /// Rounded circular point
    Rounded,
}

#[derive(Debug)]
pub(crate) enum DrawCommandData {
    Rect {
        pos: Vec2,
        size: Vec2,
        rotation: Fl,
        corner_radii: [Fl; 4],
    },
    Texture {
        texture: Texture,
        pos: Vec2,
        scale: Vec2,
        source: (Vec2, Vec2),
        rotation: Fl,
        corner_radii: [Fl; 4],
    },
    TextChar {
        glyph: PositionedGlyph<'static>,
        font: u32,
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
    Line {
        points: Vec<(Vec2, Fl, LineJoinStyle)>,
        ends: (LineEndStyle, LineEndStyle),
    },
}

#[derive(Debug)]
pub(crate) struct DrawCommand {
    pub transform: Mat3,
    pub colour: Vec4,
    pub data: DrawCommandData,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub(crate) struct Vertex2d {
    position: [f32; 2],
    uv: [f32; 2],
    colour: [u8; 4],
    rounding: f32,
    tex: u32,
}

impl Vertex2d {
    pub fn descriptor() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: [VertexAttribute; 5] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Unorm8x4, 3 => Float32, 4 => Uint32];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRS,
        }
    }
}

pub(crate) struct CareRenderState {
    pub transform_stack: Vec<Mat3>,
    pub current_transform: Mat3,
    pub current_colour: Vec4,
    pub current_surface: WindowId,
    pub commands: Vec<DrawCommand>,
    pub max_textures: usize,
    pub font_cache: FontCache<'static>,
    pub font_cache_texture: OnceLock<Texture>,
    pub default_font: Font,
    pub next_font_id: u32,
    pub line_end_style: LineEndStyle,
    pub line_join_style: LineJoinStyle,
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
pub(crate) struct DrawCall<T: bytemuck::Pod + Default> {
    pub(crate) vertices: Vec<T>,
    pub(crate) indices: Vec<u32>,
    pub(crate) textures: Vec<Texture>,
}

fn helper_line_segment_normal(pos1: Vec2, pos2: Vec2, width: f32) -> Vec2 {
    (pos2 - pos1).normalize_or(Vec2::new(0.0, 0.0)).tangent() * width / 2.0
}

fn helper_add_verts_for_line_segment(
    verts: &mut Vec<Vertex2d>,
    vert_pos: &dyn Fn((Fl, Fl), Fl) -> [f32; 2],
    colour: [u8; 4],
    pos1: Vec2,
    pos2: Vec2,
    width: f32,
) {
    let norm = helper_line_segment_normal(pos1, pos2, width);
    verts.push(Vertex2d {
        position: vert_pos((pos1.x() + norm.x(), pos1.y() + norm.y()), 0.0),
        uv: [0.0, 0.0],
        colour,
        rounding: 0.0,
        tex: 0,
    });
    verts.push(Vertex2d {
        position: vert_pos((pos1.x() - norm.x(), pos1.y() - norm.y()), 0.0),
        uv: [0.0, 0.0],
        colour,
        rounding: 0.0,
        tex: 0,
    });
}

impl CareRenderState {
    pub fn reset(&mut self) {
        self.transform_stack.clear();
        self.current_transform = Mat3::ident();
        self.current_colour = Vec4::new(1, 1, 1, 1);
        self.commands.clear();
    }
    pub fn render(&mut self) -> Vec<DrawCall<Vertex2d>> {
        let screen_size = Vec2::new(800, 600);
        let mut draw_calls = Vec::new();
        let mut cdc = DrawCall::default();
        let mut use_tex = |texture: &Texture, cdc: &mut DrawCall<Vertex2d>| {
            (if let Some(idx) = cdc.textures.iter().position(|t| t == texture) {
                // offset by one because 0 represents no texture.
                idx + 1
            } else if cdc.textures.len() < self.max_textures {
                cdc.textures.push(texture.clone());
                // Using len accounts for said offset
                cdc.textures.len()
            } else {
                let mut new_draw_call = DrawCall::default();
                std::mem::swap(&mut new_draw_call, cdc);
                draw_calls.push(new_draw_call);
                cdc.textures.push(texture.clone());
                cdc.textures.len()
            }) as u32
        };
        for command in self.commands.drain(..) {
            let vert_pos = |v: (Fl, Fl), rot: Fl| {
                let v = (&command.transform) * Vec2::from(v).rotated(rot);
                [
                    (v.x() / screen_size.0.x) as f32,
                    (v.y() / screen_size.0.y) as f32,
                ]
            };
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
                    corner_radii,
                } => {
                    let n = cdc.vertices.len() as u32;
                    let (uv, _uv_per_pix) = if size.x() > size.y() {
                        (Vec2::new(1, size.y() / size.x()), 2.0 / size.x())
                    } else {
                        (Vec2::new(1, size.x() / size.y()), 2.0 / size.y())
                    };
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos((pos.x(), pos.y()), rotation),
                        uv: [0.0, 0.0],
                        colour,
                        rounding: 0.0,
                        tex: 0,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos((pos.x() + size.x(), pos.y()), rotation),
                        uv: [uv.x(), 0.0],
                        colour,
                        rounding: 0.0,
                        tex: 0,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos((pos.x(), pos.y() + size.y()), rotation),
                        uv: [0.0, uv.y()],
                        colour,
                        rounding: 0.0,
                        tex: 0,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos((pos.x() + size.x(), pos.y() + size.y()), rotation),
                        uv: [uv.x(), uv.y()],
                        colour,
                        rounding: 0.0,
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
                    let tex_size = texture.size();
                    let tex = use_tex(&texture, &mut cdc);
                    let n = cdc.vertices.len() as u32;
                    let size = tex_size * scale;
                    let uv_base = source.0 / tex_size;
                    let uv_size = source.1 / tex_size;
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos((pos.0.x, pos.0.y), rotation),
                        uv: [uv_base.x(), uv_base.y()],
                        colour,
                        rounding: 0.0,
                        tex,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos((pos.0.x + size.0.x, pos.0.y), rotation),
                        uv: [uv_base.x() + uv_size.x(), uv_base.y()],
                        colour,
                        rounding: 0.0,
                        tex,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos((pos.0.x, pos.0.y + size.0.y), rotation),
                        uv: [uv_base.x(), uv_base.y() + uv_size.y()],
                        colour,
                        rounding: 0.0,
                        tex,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos((pos.0.x + size.0.x, pos.0.y + size.0.y), rotation),
                        uv: [uv_base.x() + uv_size.x(), uv_base.y() + uv_size.y()],
                        colour,
                        rounding: 0.0,
                        tex,
                    });
                    cdc.indices
                        .extend_from_slice(&[n, n + 1, n + 2, n + 2, n + 1, n + 3])
                }
                DrawCommandData::TextChar { glyph, font } => {
                    let texture = self.font_cache_texture.get().unwrap();
                    let tex = use_tex(texture, &mut cdc);
                    let n = cdc.vertices.len() as u32;
                    if let Some(rect) = self.font_cache.rect_for(font as usize, &glyph).unwrap() {
                        let pos = Vec2::new(rect.1.min.x, rect.1.min.y);
                        let size = Vec2::new(rect.1.max.x, rect.1.max.y) - pos;
                        let uv_base = Vec2::new(rect.0.min.x, rect.0.min.y);
                        let uv_size = Vec2::new(rect.0.max.x, rect.0.max.y) - uv_base;
                        cdc.vertices.push(Vertex2d {
                            position: vert_pos((pos.0.x, pos.0.y), 0.0),
                            uv: [uv_base.x(), uv_base.y()],
                            colour,
                            rounding: 0.0,
                            tex,
                        });
                        cdc.vertices.push(Vertex2d {
                            position: vert_pos((pos.0.x + size.0.x, pos.0.y), 0.0),
                            uv: [uv_base.x() + uv_size.x(), uv_base.y()],
                            colour,
                            rounding: 0.0,
                            tex,
                        });
                        cdc.vertices.push(Vertex2d {
                            position: vert_pos((pos.0.x, pos.0.y + size.0.y), 0.0),
                            uv: [uv_base.x(), uv_base.y() + uv_size.y()],
                            colour,
                            rounding: 0.0,
                            tex,
                        });
                        cdc.vertices.push(Vertex2d {
                            position: vert_pos((pos.0.x + size.0.x, pos.0.y + size.0.y), 0.0),
                            uv: [uv_base.x() + uv_size.x(), uv_base.y() + uv_size.y()],
                            colour,
                            rounding: 0.0,
                            tex,
                        });
                        cdc.indices
                            .extend_from_slice(&[n, n + 1, n + 2, n + 2, n + 1, n + 3])
                    }
                }
                DrawCommandData::Triangle { verts, tex_uvs } => {
                    let (tex, uvs) = if let Some((tex, uvs)) = tex_uvs {
                        (use_tex(&tex, &mut cdc), uvs)
                    } else {
                        (0, [Vec2::new(0.5, 0.5); 3])
                    };
                    let n = cdc.vertices.len() as u32;
                    for (pos, uv) in verts.iter().zip(uvs.iter()) {
                        cdc.vertices.push(Vertex2d {
                            position: vert_pos((pos.x(), pos.y()), 0.0),
                            uv: [uv.x(), uv.y()],
                            colour,
                            rounding: 0.0,
                            tex,
                        });
                    }
                    cdc.indices.extend_from_slice(&[n, n + 1, n + 2])
                }
                DrawCommandData::Circle {
                    center,
                    radius,
                    elipseness,
                } => {
                    let n = cdc.vertices.len() as u32;
                    let sqrt_3 = (3.0f32).sqrt();
                    let left = Vec2::new(-sqrt_3 * radius, -radius);
                    let right = Vec2::new(sqrt_3 * radius, -radius);
                    let top = Vec2::new(0.0, 2.0 * radius);
                    let e_dir = elipseness.normalize_or(Vec2::new(1, 0));
                    let e_tan = e_dir.tangent();
                    let e_len = elipseness.length() + 1.0;
                    let e_mat =
                        Mat2::new(e_dir.x() * e_len, -e_tan.x(), e_dir.y() * e_len, -e_tan.y());
                    let left = center + &e_mat * left;
                    let right = center + &e_mat * right;
                    let top = center + &e_mat * top;
                    let left_uv = Vec2::new((1.0 - sqrt_3) / 2.0, 0.0);
                    let right_uv = Vec2::new(1.0 + (sqrt_3 - 1.0) / 2.0, 0.0);
                    let top_uv = Vec2::new(0.5, 1.5);
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos((left.x(), left.y()), 0.0),
                        uv: [left_uv.x(), left_uv.y()],
                        colour,
                        rounding: 0.5,
                        tex: 0,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos((top.x(), top.y()), 0.0),
                        uv: [top_uv.x(), top_uv.y()],
                        colour,
                        rounding: 1.0,
                        tex: 0,
                    });
                    cdc.vertices.push(Vertex2d {
                        position: vert_pos((right.x(), right.y()), 0.0),
                        uv: [right_uv.x(), right_uv.y()],
                        colour,
                        rounding: 0.5,
                        tex: 0,
                    });
                    cdc.indices.extend_from_slice(&[n, n + 1, n + 2])
                }
                DrawCommandData::Line { points, ends } => {
                    // TODO: Line Ends
                    let mut n = (cdc.vertices.len() as u32, cdc.vertices.len() as u32 + 1);
                    helper_add_verts_for_line_segment(
                        &mut cdc.vertices,
                        &vert_pos,
                        colour,
                        points[0].0,
                        points[1].0,
                        points[0].1,
                    );
                    for segs in points.windows(3) {
                        let m = cdc.vertices.len() as u32;
                        helper_add_verts_for_line_segment(
                            &mut cdc.vertices,
                            &vert_pos,
                            colour,
                            segs[1].0,
                            segs[0].0,
                            -segs[1].1,
                        );
                        cdc.indices
                            .extend_from_slice(&[n.0, n.1, m, m, n.1, m + 1]);
                        n = (cdc.vertices.len() as u32, cdc.vertices.len() as u32 + 1);
                        helper_add_verts_for_line_segment(
                            &mut cdc.vertices,
                            &vert_pos,
                            colour,
                            segs[1].0,
                            segs[2].0,
                            segs[1].1,
                        );
                    }
                    let m = cdc.vertices.len() as u32;
                    helper_add_verts_for_line_segment(
                        &mut cdc.vertices,
                        &vert_pos,
                        colour,
                        points[points.len()-1].0,
                        points[points.len()-2].0,
                        -points[points.len()-1].1,
                    );
                    cdc.indices
                        .extend_from_slice(&[n.0, n.1, m, m, n.1, m + 1]);
                }
            }
        }
        //println!("{cdc:?}");
        draw_calls.push(cdc);
        draw_calls
    }
}
