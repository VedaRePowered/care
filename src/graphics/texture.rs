use std::{fmt::Debug, io::Cursor, path::Path, sync::Arc};

use image::{DynamicImage, EncodableLayout, ImageFormat, ImageReader, RgbaImage};

use crate::math::{Vec2, Vec4};

use super::GRAPHICS_STATE;

#[derive(Debug, Clone)]
/// A high-level object to wrap textures
pub struct Texture(pub(crate) Arc<TextureHandle>);

impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Texture {
    /// Create a new texture by loading an image from the filesystem
    pub fn new(filename: impl AsRef<Path>) -> Self {
        Self::new_from_image(ImageReader::open(filename).unwrap().decode().unwrap())
    }
    /// Creates a new texture by loading an image from encoded image data of an optionally specified format.
    pub fn new_from_file_format(file_data: &[u8], format_hint: Option<ImageFormat>) -> Self {
        let mut image = ImageReader::new(Cursor::new(file_data))
            .with_guessed_format()
            .unwrap();
        if let Some(fmt) = format_hint {
            image.set_format(fmt);
        }
        Self::new_from_image(image.decode().unwrap())
    }
    /// Create a new texture by filling it up in a single colour
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
    /// Create a new texture out of an image from the image crate
    pub fn new_from_image(img: DynamicImage) -> Self {
        Self::new_from_data(img.width(), img.height(), img.to_rgba8().as_bytes())
    }
    /// Create a new texture out of a size and raw data
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
            data,
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
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Texture(Arc::new(TextureHandle {
            size: Vec2::new(width, height),
            texture: Arc::new(texture),
            view,
            sampler,
        }))
    }
    pub(crate) fn new_from_wgpu(texture: Arc<wgpu::Texture>) -> Self {
        let state = GRAPHICS_STATE.get().expect("Graphics not initialized");
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Texture(Arc::new(TextureHandle {
            size: Vec2::new(texture.width(), texture.height()),
            texture,
            view,
            sampler,
        }))
    }
    /// Upload data to a specific region of the texture
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
    /// Upload an image to a specific region of the image
    pub fn upload_image_region(&self, image: RgbaImage, x: u32, y: u32) {
        let (width, height) = (image.width(), image.height());
        self.upload_region(image.as_bytes(), x, y, width, height);
    }
    /// Get the size of the texture
    pub fn size(&self) -> Vec2 {
        self.0.size
    }
}

#[derive(Debug)]
pub(crate) struct TextureHandle {
    pub(crate) size: Vec2,
    pub(crate) texture: Arc<wgpu::Texture>,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
}

impl TextureHandle {
    pub(crate) fn bind_group_entries(&self, i: u32) -> [wgpu::BindGroupEntry<'_>; 2] {
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
