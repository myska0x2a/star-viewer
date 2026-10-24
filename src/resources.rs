//! App resource loading and management (e.g. Textures, Audio).
use log::info;
use sdl3::gpu::*;
use sdl3::image::ImageIOStream;
use sdl3::image::*;
use sdl3::iostream::IOStream;
use sdl3::pixels::PixelFormat;
use sdl3::render::Canvas;
use sdl3::surface::Surface;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone)]
pub struct ResourceManager {
    root: &'static str,
    textures: HashMap<&'static str, Texture<'static>>,
}

impl ResourceManager {
    pub fn load(device: &Device, path: &'static str) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Loading assets");

        let copy_commands: CommandBuffer = device.acquire_command_buffer()?;
        let copy_pass: CopyPass = device.begin_copy_pass(&copy_commands)?;

        let star_texture: Texture<'_> =
            create_texture_from_image(device, "assets/star_pixel.png", &copy_pass)?;

        device.end_copy_pass(copy_pass);
        copy_commands.submit()?;

        let mut textures: HashMap<&str, Texture<'_>> = HashMap::new();

        textures.insert("star.png", star_texture);

        Ok(Self {
            root: path,
            textures,
        })
    }

    pub fn get_texture(&self, id: &'static str) -> &Texture<'static> {
        self.textures.get(id).unwrap()
    }
}

// https://github.com/vhspace/sdl3-rs/blob/master/examples/gpu-texture.rs
fn create_texture_from_image(
    gpu: &Device,
    image_path: impl AsRef<Path>,
    copy_pass: &CopyPass,
) -> Result<Texture<'static>, Box<dyn std::error::Error>> {
    let io: IOStream<'_> = IOStream::from_file(image_path, "r")?;
    let mut image: Surface<'_> = io.load_png()?;

    let image_size: (u32, u32) = image.size();
    let size_bytes: u32 =
        image.pixel_format().bytes_per_pixel() as u32 * image_size.0 * image_size.1;

    info!("{}", image.pixel_format().is_alpha());

    let texture: Texture<'_> = gpu.create_texture(
        TextureCreateInfo::new()
            .with_format(TextureFormat::R8g8b8a8Unorm)
            .with_type(TextureType::_2D)
            .with_width(image_size.0)
            .with_height(image_size.1)
            .with_layer_count_or_depth(1)
            .with_num_levels(1)
            .with_usage(TextureUsage::SAMPLER),
    )?;

    let transfer_buffer: TransferBuffer = gpu
        .create_transfer_buffer()
        .with_size(size_bytes)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;

    let mut buffer_mem: BufferMemMap<'_, u8> = transfer_buffer.map(gpu, false);
    image.with_lock(|image_bytes: &[u8]| {
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
