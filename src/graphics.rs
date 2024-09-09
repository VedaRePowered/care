//! Graphics functions, all of which will panic if called from a thread that is not the main
//! thread, or if any function is called before calling [init] from the main thread.

mod api;
mod font;
mod graphics_state;
mod render_2d;
mod texture;

#[doc(inline)]
pub use api::*;
#[doc(inline)]
pub use font::Font;
#[doc(inline)]
pub use render_2d::{LineEndStyle, LineJoinStyle};
#[doc(inline)]
pub use texture::Texture;

pub(crate) use graphics_state::{GraphicsState, GRAPHICS_STATE};
pub(crate) use render_2d::*;

/// Useful default struct imports
pub mod prelude {
    pub use super::Font;
    pub use super::Texture;
}
