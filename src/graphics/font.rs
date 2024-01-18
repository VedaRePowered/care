use std::{fmt::Debug, fs, path::Path, sync::Arc};

use super::GRAPHICS_STATE;

#[derive(Debug, Clone)]
/// A font that can be used to display text
pub struct Font(pub(crate) Arc<(rusttype::Font<'static>, u32)>);

fn next_font_id() -> u32 {
    let mut render = GRAPHICS_STATE.get().unwrap().care_render.write();
    let id = render.next_font_id;
    render.next_font_id += 1;
    id
}

impl Font {
    /// Create a new font from a font file
    pub fn new(file: impl AsRef<Path>) -> Self {
        Font::new_from_vec(fs::read(file).unwrap())
    }
    pub fn new_from_vec(bytes: Vec<u8>) -> Self {
        Font(Arc::new((
            rusttype::Font::try_from_vec(bytes).unwrap(),
            next_font_id(),
        )))
    }
    pub fn new_from_bytes(bytes: &'static [u8]) -> Self {
        Self::new_from_bytes_and_id(bytes, next_font_id())
    }
    pub(crate) fn new_from_bytes_and_id(bytes: &'static [u8], id: u32) -> Self {
        Font(Arc::new((
            rusttype::Font::try_from_bytes(bytes).unwrap(),
            id,
        )))
    }
}
