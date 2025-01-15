use std::sync::OnceLock;

pub use wrgpgpu::*;

/// Additional available bindings for the compute shader
pub mod bind {
    pub use wrgpgpu::bindings::buffer::*;
    pub use wrgpgpu::bindings::texture::*;
    pub use wrgpgpu::bindings::{Bind, BindGroup, BindGroupData, BindGroups};
}

pub use bind::{BindableImage, PlainTextureBind, RgbaStorageTextureBind, StorageTextureBind};
pub use bind::{StorageBufferBind, StorageReadBufferBind, UniformBufferBind};

static COMPUTE_DEVICE: OnceLock<wrgpgpu::Device> = OnceLock::new();

fn new_compute_device() -> wrgpgpu::Device {
    #[cfg(feature = "graphics")]
    {
        let state =
            crate::graphics::GRAPHICS_STATE.get_or_init(crate::graphics::GraphicsState::new);
        wrgpgpu::Device::from_wgpu(state.device.clone(), state.queue.clone())
    }
    #[cfg(not(feature = "graphics"))]
    wrgpgpu::Device::auto_high_performance()
}

/// Create a compute shader
pub fn create_shader<B: wrgpgpu::bindings::BindGroups>(args: ShaderArgs<'_>) -> ComputeShader<B> {
    let device = COMPUTE_DEVICE.get_or_init(new_compute_device);

    device.create_shader(args)
}

/// Create an empty bind (e.g. texture or buffer) to use with a compute shader.
pub fn empty_bind<B: wrgpgpu::bindings::Bind>(create_info: B::CreateInfo) -> B {
    let device = COMPUTE_DEVICE.get_or_init(new_compute_device);

    B::new_empty(device, create_info)
}

/// Create an bind (e.g. texture or buffer) filled with initial data to use with a compute shader.
pub fn init_bind<B: wrgpgpu::bindings::Bind>(data: B::Data) -> B {
    let device = COMPUTE_DEVICE.get_or_init(new_compute_device);

    B::new_init(device, data)
}

/// Create a bind group for use in a shader
pub fn bind<B: wrgpgpu::bindings::BindGroupData>(data: &B) -> wrgpgpu::BindGroup<B> {
    let device = COMPUTE_DEVICE.get_or_init(new_compute_device);

    device.bind(data)
}

/// Dispatch a compute shader pass
pub fn dispatch<B: wrgpgpu::bindings::BindGroups>(
    shader: &ComputeShader<B>,
    bindings: &B,
    workgroups: (u32, u32, u32),
) {
    let device = COMPUTE_DEVICE.get_or_init(new_compute_device);

    device.dispatch(shader, bindings, workgroups);
}

/// Check weather all compute passes are complete
pub fn is_complete() -> bool {
    let device = COMPUTE_DEVICE.get_or_init(new_compute_device);

    device.is_complete()
}

#[cfg(feature = "graphics")]
/// Create a care texture, to be used in the [`crate::grapics`] module, from a compute texture binding
///
/// These textures will share the same storage in gpu memory, and as such any modification to the
/// texture from the compute shader will be observed in the graphics module when the texture is
/// rendered.
pub fn get_texture_from_binding<T: wrgpgpu::bindings::texture::TextureBindType>(
    binding: wrgpgpu::TextureBind<image::RgbaImage, T>,
) -> crate::graphics::Texture {
    crate::graphics::Texture::new_from_wgpu(binding.texture)
}
