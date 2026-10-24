//! Module for the rendering of stars specifically.
use crate::core::{AppCore, Camera};
use crate::stars::*;
use crate::{graphics::rendering::*, resources::ResourceManager};
use cgmath::{Matrix4, PerspectiveFov, Rad, Vector3};
use log::info;
use sdl3::{gpu::TransferBufferUsage, gpu::*, video::Window};

pub struct StarRenderer<'a> {
    pipeline: GraphicsPipeline,
    star_buffer: Buffer,
    depth_stencil: Texture<'a>,
    pub num_stars: usize,
    pub range: f32,
}

impl<'a> StarRenderer<'a> {
    pub fn load(
        device: &Device,
        window: &Window,
        appcore: &AppCore,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let vertex_shader: Shader = device
            .create_shader()
            .with_code(
                ShaderFormat::SPIRV,
                include_bytes!("../../shaders/stars/stars.vert.spv"),
                ShaderStage::Vertex,
            )
            .with_entrypoint(c"main")
            .with_storage_buffers(1)
            .with_uniform_buffers(1)
            .build()?;

        let fragment_shader: Shader = device
            .create_shader()
            .with_code(
                ShaderFormat::SPIRV,
                include_bytes!("../../shaders/stars/stars.frag.spv"),
                ShaderStage::Fragment,
            )
            .with_entrypoint(c"main")
            .with_samplers(1)
            .build()?;

        let swapchain_format: TextureFormat = device.get_swapchain_texture_format(&window);

        let pipeline: GraphicsPipeline = device
            .create_graphics_pipeline()
            .with_fragment_shader(&fragment_shader)
            .with_vertex_shader(&vertex_shader)
            .with_primitive_type(PrimitiveType::TriangleList)
            .with_fill_mode(FillMode::Fill)
            .with_depth_stencil_state(
                DepthStencilState::new()
                    .with_enable_depth_test(true)
                    .with_enable_depth_write(true)
                    .with_compare_op(CompareOp::Less),
            )
            .with_target_info(
                GraphicsPipelineTargetInfo::new()
                    .with_has_depth_stencil_target(true)
                    .with_depth_stencil_format(TextureFormat::D16Unorm)
                    .with_color_target_descriptions(&[ColorTargetDescription::new()
                        .with_format(swapchain_format)
                        .with_blend_state(
                            ColorTargetBlendState::default()
                                .with_enable_blend(true)
                                .with_src_color_blendfactor(BlendFactor::SrcColor)
                                .with_dst_color_blendfactor(BlendFactor::OneMinusSrcColor)
                                .with_src_alpha_blendfactor(BlendFactor::SrcAlpha)
                                .with_dst_alpha_blendfactor(BlendFactor::OneMinusSrcAlpha)
                                .with_alpha_blend_op(BlendOp::Add)
                                .with_color_blend_op(BlendOp::Add),
                        )]),
            )
            .build()?;

        drop(vertex_shader);
        drop(fragment_shader);

        let mut depth_stencil: Texture<'_> = device.create_texture(
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

        let range: f32 = appcore.camera.range;

        let (star_buffer, num_stars): (Buffer, usize) = load_star_buffer(
            device,
            window,
            &appcore.star_handler,
            range,
            appcore.camera.pos,
        )?;

        Ok(Self {
            pipeline: pipeline,
            star_buffer: star_buffer,
            depth_stencil,
            num_stars: num_stars,
            range,
        })
    }

    pub fn render(
        &mut self,
        device: &Device,
        window: &Window,
        command_buffer: &CommandBuffer,
        color_targets: &[ColorTargetInfo; 1],
        camera: &Camera,
        resources: &ResourceManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[allow(unused)]
        #[repr(align(16))]
        #[derive(Copy, Clone)]
        struct UniformData {
            projection_matrix: Matrix4<f32>,
            position: Vector3<f32>,
            rotation: Vector3<f32>,
        }

        let window_size: (u32, u32) = window.size();
        let projection_matrix: PerspectiveFov<f32> =
            // camera.get_projection_matrix(window_size.0 as f32, window_size.1 as f32);
            camera.get_projection_matrix(1920.0, 1200.0);

        let uniform_data: UniformData = UniformData {
            projection_matrix: Matrix4::from(projection_matrix),
            position: camera.pos,
            rotation: camera.orientation,
        };

        let depth_target: DepthStencilTargetInfo = DepthStencilTargetInfo::new()
            .with_texture(&mut self.depth_stencil)
            .with_cycle(true)
            .with_clear_depth(1.0)
            .with_clear_stencil(0)
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::STORE)
            .with_stencil_load_op(LoadOp::CLEAR)
            .with_stencil_store_op(StoreOp::STORE);

        let render_pass: RenderPass = device
            // .begin_render_pass(&command_buffer, color_targets, Some(&depth_target))
            .begin_render_pass(&command_buffer, color_targets, None)
            .unwrap();

        render_pass.bind_graphics_pipeline(&self.pipeline);
        render_pass.bind_vertex_storage_buffers(0, &[self.star_buffer.clone()]);

        let star_texture: &Texture<'static> = resources.get_texture("star.png");

        let texture_sampler: Sampler = device.create_sampler(SamplerCreateInfo::new())?;

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
        appcore: &AppCore,
    ) -> Result<(), Box<dyn std::error::Error>> {
        (self.star_buffer, self.num_stars) = load_star_buffer(
            &device,
            &window,
            &appcore.star_handler,
            appcore.camera.range,
            appcore.camera.pos,
        )?;
        Ok(())
    }
}

fn load_star_buffer(
    device: &Device,
    window: &Window,
    star_handler: &StarHandler,
    range: f32,
    pos: Vector3<f32>,
) -> Result<(Buffer, usize), Box<dyn std::error::Error>> {
    let stars: Vec<&Star> = star_handler.get_nearby(range as f64, pos);
    let num_stars: usize = stars.len() + 32;

    let mut star_data: Vec<StarVertexData> = Vec::new();

    for star in stars {
        star_data.push(StarVertexData::from(star));
    }

    let buffer_size: u32 = (num_stars * size_of::<StarVertexData>()) as u32;

    let upload: TransferBuffer = device
        .create_transfer_buffer()
        .with_size(buffer_size)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;

    let copy_cmd: CommandBuffer = device.acquire_command_buffer()?;
    let copy_pass: CopyPass = device.begin_copy_pass(&copy_cmd)?;

    let star_buffer: Buffer = create_buffer_with_data(
        &device,
        &upload,
        &copy_pass,
        BufferUsageFlags::COMPUTE_STORAGE_WRITE,
        &star_data,
    )?;

    info!("meow");

    device.end_copy_pass(copy_pass);
    copy_cmd.submit()?;

    Ok((star_buffer, num_stars))
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
    fn from(star: &Star) -> Self {
        Self {
            position: [star.x as f32, star.y as f32, star.z as f32],
            temperature: star.ci.unwrap_or(0.3),
            magnitude: star.absmag,
        }
    }
}
