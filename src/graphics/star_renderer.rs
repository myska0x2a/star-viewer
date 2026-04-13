//! Module for the rendering of stars specifically.
use crate::stars::*;
use crate::{graphics::rendering::*, resources::ResourceManager};
use cgmath::{Matrix4, PerspectiveFov, Rad};
use sdl3::{gpu::*, video::Window};

pub struct StarRenderer {
    pipeline: GraphicsPipeline,
    star_buffer: Buffer,
    star_handler: StarHandler,
    num_stars: usize,
    pub range: f32,
}

impl StarRenderer {
    pub fn load(
        device: &Device,
        window: &Window,
        star_handler: StarHandler,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let fs_source = include_bytes!("../../shaders/stars/stars.frag.spv");
        let vs_source = include_bytes!("../../shaders/stars/stars.vert.spv");

        let vs_shader = device
            .create_shader()
            .with_code(ShaderFormat::SPIRV, vs_source, ShaderStage::Vertex)
            .with_entrypoint(c"main")
            .with_storage_buffers(1)
            .with_uniform_buffers(1)
            .build()?;

        let fs_shader = device
            .create_shader()
            .with_code(ShaderFormat::SPIRV, fs_source, ShaderStage::Fragment)
            .with_entrypoint(c"main")
            // .with_samplers(1)
            .build()?;

        let swapchain_format = device.get_swapchain_texture_format(&window);

        let pipeline = device
            .create_graphics_pipeline()
            .with_fragment_shader(&fs_shader)
            .with_vertex_shader(&vs_shader)
            .with_primitive_type(PrimitiveType::TriangleList)
            .with_fill_mode(FillMode::Fill)
            .with_target_info(
                GraphicsPipelineTargetInfo::new().with_color_target_descriptions(&[
                    ColorTargetDescription::new().with_format(swapchain_format),
                ]),
            )
            .build()?;

        drop(vs_shader);
        drop(fs_shader);

        let copy_commands = device.acquire_command_buffer()?;
        let copy_pass = device.begin_copy_pass(&copy_commands)?;

        let range = 20.0;

        let (star_buffer, num_stars) = load_star_buffer(device, window, &star_handler, range)?;

        return Ok(Self {
            pipeline: pipeline,
            star_buffer: star_buffer,
            star_handler,
            num_stars: num_stars,
            range,
        });
    }

    pub fn render(
        &mut self,
        device: &Device,
        window: &Window,
        command_buffer: &CommandBuffer,
        color_targets: &[ColorTargetInfo; 1],
        camera: &mut Camera,
        resources: &ResourceManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        camera.increment_vel();

        #[repr(align(16))]
        #[derive(Copy, Clone)]
        struct UniformData {
            projection_matrix: Matrix4<f32>,
            position: [f32; 3],
            rotation: [f32; 3],
        }

        let window_size = window.size();

        let fov = Rad(camera.fov);
        let projection_matrix = PerspectiveFov {
            fovy: fov,
            aspect: window_size.0 as f32 / window_size.1 as f32,
            near: 0.1,
            far: 300.0,
        };

        let uniform_data = UniformData {
            projection_matrix: Matrix4::from(projection_matrix),
            position: camera.pos,
            rotation: camera.orientation,
        };

        let render_pass = device
            .begin_render_pass(&command_buffer, color_targets, None)
            .unwrap();

        let buffer = self.star_buffer.clone();

        render_pass.bind_graphics_pipeline(&self.pipeline);
        render_pass.bind_vertex_storage_buffers(0, &[buffer]);

        let star_texture = resources.get_texture("star.bmp");

        let texture_sampler = device.create_sampler(
            SamplerCreateInfo::new()
                .with_min_filter(Filter::Nearest)
                .with_mag_filter(Filter::Nearest)
                .with_mipmap_mode(SamplerMipmapMode::Nearest)
                .with_address_mode_u(SamplerAddressMode::Repeat)
                .with_address_mode_v(SamplerAddressMode::Repeat)
                .with_address_mode_w(SamplerAddressMode::Repeat),
        )?;

        render_pass.bind_fragment_samplers(
                0,
                &[TextureSamplerBinding::new()
                    .with_texture(&star_texture)
                    .with_sampler(&texture_sampler)],
        );

        command_buffer.push_vertex_uniform_data(0, &uniform_data);
        render_pass.draw_primitives(self.num_stars * 6, self.num_stars * 2, 0, 0);

        device.end_render_pass(render_pass);

        Ok(())
    }

    pub fn reload(
        &mut self,
        device: &Device,
        window: &Window,
        range: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        (self.star_buffer, self.num_stars) =
            load_star_buffer(&device, &window, &self.star_handler, range)?;
        Ok(())
    }
}

fn load_star_buffer(
    device: &Device,
    window: &Window,
    star_handler: &StarHandler,
    range: f32,
) -> Result<(Buffer, usize), Box<dyn std::error::Error>> {
    let stars = star_handler.get_nearby(range as f64);

    let max_stars = stars.len();

    let mut star_data: Vec<StarVertexData> = Vec::new();

    for star in stars {
        star_data.push(StarVertexData::from(star));
    }

    let buffer_size = (max_stars * size_of::<StarVertexData>()) as u32;

    // buffer which stores the stars
    let star_buffer = device
        .create_buffer()
        .with_size(buffer_size)
        .with_usage(BufferUsageFlags::COMPUTE_STORAGE_WRITE)
        .build()?;

    let upload = device
        .create_transfer_buffer()
        .with_size(buffer_size)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;

    {
        let mut map = upload.map::<StarVertexData>(&device, true);
        map.mem_mut().copy_from_slice(&star_data);
        map.unmap();

        let copy_cmd = device.acquire_command_buffer()?;
        let copy_pass = device.begin_copy_pass(&copy_cmd)?;
        copy_pass.upload_to_gpu_buffer(
            TransferBufferLocation::new()
                .with_offset(0)
                .with_transfer_buffer(&upload),
            BufferRegion::new()
                .with_offset(0)
                .with_size(buffer_size)
                .with_buffer(&star_buffer),
            true,
        );
        device.end_copy_pass(copy_pass);
        copy_cmd.submit()?;
    }

    return Ok((star_buffer, max_stars));
}

// data to be passed to the shader.
#[repr(align(16))]
#[derive(Copy, Clone)]
struct StarVertexData {
    position: [f32; 3],
    temperature: f32,
    magnitude: f32,
}

impl From<&Star> for StarVertexData {
    fn from(star: &Star) -> StarVertexData {
        let position = [star.x as f32, star.y as f32, star.z as f32];

        return StarVertexData {
            position,
            temperature: star.ci.unwrap_or(0.3),
            magnitude: 1.0,
        };
    }
}
