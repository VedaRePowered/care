use std::sync::OnceLock;

pub use wrgpgpu::*;

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
