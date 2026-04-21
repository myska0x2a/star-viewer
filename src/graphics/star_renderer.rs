//! Module for the rendering of stars specifically.
use crate::stars::*;
use crate::{graphics::rendering::*, resources::ResourceManager};
use cgmath::{Matrix4, PerspectiveFov, Rad};
use sdl3::{gpu::TransferBufferUsage, gpu::*, video::Window};
use log::info;

pub struct StarRenderer<'a> {
    pipeline: GraphicsPipeline,
    star_buffer: Buffer,
    depth_stencil: Texture<'a>,
    star_handler: StarHandler,
    pub num_stars: usize,
    pub range: f32,
}

impl<'a> StarRenderer<'a> {
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
            .with_samplers(1)
            .build()?;

        let swapchain_format = device.get_swapchain_texture_format(&window);

        let pipeline = device
            .create_graphics_pipeline()
            .with_fragment_shader(&fs_shader)
            .with_vertex_shader(&vs_shader)
            .with_primitive_type(PrimitiveType::TriangleList)
            .with_fill_mode(FillMode::Fill)
            .with_depth_stencil_state(
                DepthStencilState::new()
                    .with_enable_depth_test(true)
                    .with_enable_depth_write(true)
                    .with_compare_op(CompareOp::Less),
            )
            .with_target_info(
                GraphicsPipelineTargetInfo::new().with_color_target_descriptions(&[
                    ColorTargetDescription::new().with_format(swapchain_format),
                ])
                .with_has_depth_stencil_target(true)
                .with_depth_stencil_format(TextureFormat::D16Unorm),
            )
            .build()?;

        drop(vs_shader);
        drop(fs_shader);

        let mut depth_stencil = device.create_texture(
            TextureCreateInfo::new()
                .with_type(TextureType::_2D)
                .with_width(window.size().0)
                .with_height(window.size().1)
                .with_layer_count_or_depth(1)
                .with_num_levels(1)
                .with_sample_count(SampleCount::NoMultiSampling)
                .with_format(TextureFormat::D16Unorm)
                .with_usage(TextureUsage::SAMPLER | TextureUsage::DEPTH_STENCIL_TARGET),
        )?;

        let range = 0.0;

        let (star_buffer, num_stars) = load_star_buffer(device, window, &star_handler, range)?;

        return Ok(Self {
            pipeline: pipeline,
            star_buffer: star_buffer,
            depth_stencil,
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

        #[allow(unused)]
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

        let depth_target = DepthStencilTargetInfo::new()
            .with_texture(&mut self.depth_stencil)
            .with_cycle(true)
            .with_clear_depth(1.0)
            .with_clear_stencil(0)
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::STORE)
            .with_stencil_load_op(LoadOp::CLEAR)
            .with_stencil_store_op(StoreOp::STORE);

        let render_pass = device
            .begin_render_pass(&command_buffer, color_targets, Some(&depth_target))
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
    let num_stars = stars.len() + 32;

    let mut star_data: Vec<StarVertexData> = Vec::new();

    for star in stars {
        star_data.push(StarVertexData::from(star));
    }

    let buffer_size = (num_stars * size_of::<StarVertexData>()) as u32;

    let upload = device
        .create_transfer_buffer()
        .with_size(buffer_size)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;

    let copy_cmd = device.acquire_command_buffer()?;
    let copy_pass = device.begin_copy_pass(&copy_cmd)?;

    let star_buffer = create_buffer_with_data(
        &device,
        &upload,
        &copy_pass,
        BufferUsageFlags::COMPUTE_STORAGE_WRITE,
        &star_data,
    )?;

    info!("meow");

    device.end_copy_pass(copy_pass);
    copy_cmd.submit()?;

    return Ok((star_buffer, num_stars));
}

// data to be passed to the shader.
#[allow(unused)]
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
            magnitude: star.absmag,
        };
    }
}
