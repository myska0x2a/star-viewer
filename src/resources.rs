//! Game resource loading and management (e.g. Textures, Audio).
use log::info;
use sdl3::gpu::*;
use sdl3::surface::Surface;
use std::collections::HashMap;
use std::io::Error;
use std::path::Path;

#[derive(Clone)]
pub struct ResourceManager {
    root: &'static str,
    textures: HashMap<&'static str, Texture<'static>>,
}

impl ResourceManager {
    pub fn load(device: &Device, path: &'static str) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Loading assets");

        let copy_commands = device.acquire_command_buffer()?;
        let copy_pass = device.begin_copy_pass(&copy_commands)?;
        let star_texture = create_texture_from_image(device, "star.bmp", &copy_pass)?;

        let mut textures = HashMap::new();
        textures.insert("star.bmp", star_texture);

        Ok(Self {
            root: path,
            textures,
        })
    }

    pub fn get_texture(&self, id: &'static str) -> &Texture<'static> {
        let texture = self.textures.get(id);
        return texture.unwrap();
    }
}

// https://github.com/vhspace/sdl3-rs/blob/master/examples/gpu-texture.rs
pub fn create_texture_from_image(
    gpu: &Device,
    image_path: impl AsRef<Path>,
    copy_pass: &CopyPass,
) -> Result<Texture<'static>, Box<dyn std::error::Error>> {
    let image = Surface::load_bmp(image_path.as_ref())?;
    let image_size = image.size();
    let size_bytes = image.pixel_format().bytes_per_pixel() as u32 * image_size.0 * image_size.1;

    let texture = gpu.create_texture(
        TextureCreateInfo::new()
            .with_format(TextureFormat::R8g8b8a8Unorm)
            .with_type(TextureType::_2D)
            .with_width(image_size.0)
            .with_height(image_size.1)
            .with_layer_count_or_depth(1)
            .with_num_levels(1)
            .with_usage(TextureUsage::SAMPLER),
    )?;

    let transfer_buffer = gpu
        .create_transfer_buffer()
        .with_size(size_bytes)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;

    let mut buffer_mem = transfer_buffer.map::<u8>(gpu, false);
    image.with_lock(|image_bytes| {
        buffer_mem.mem_mut().copy_from_slice(image_bytes);
    });
    buffer_mem.unmap();

    copy_pass.upload_to_gpu_texture(
        TextureTransferInfo::new()
            .with_transfer_buffer(&transfer_buffer)
            .with_offset(0),
        TextureRegion::new()
            .with_texture(&texture)
            .with_layer(0)
            .with_width(image_size.0)
            .with_height(image_size.1)
            .with_depth(1),
        false,
    );

    Ok(texture)
}
