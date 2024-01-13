//! Graphics functions, all of which will panic if called from a thread that is not the main
//! thread, or if any function is called before calling [init()] from the main thread.

use std::{fmt::Display, sync::OnceLock};

use wgpu::Instance;

use crate::math::Vec2;

struct GraphicsState {
    instance: Instance,
}

impl GraphicsState {
    fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        Self { instance }
    }
}

static GRAPHICS_STATE: OnceLock<GraphicsState> = OnceLock::new();

/// Initialize the graphics library, must be called on the main thread!
pub fn init() {
    GRAPHICS_STATE.get_or_init(|| GraphicsState::new());
}

pub fn text(text: impl Display, pos: impl Into<Vec2>) {}

pub fn swap() {}
