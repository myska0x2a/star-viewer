use sdl3::{
    event::Event,
    gpu::{
        Buffer, BufferBinding, BufferRegion, BufferUsageFlags, ColorTargetDescription,
        ColorTargetInfo, CompareOp, CopyPass, CullMode, DepthStencilState, DepthStencilTargetInfo,
        Device, FillMode, Filter, GraphicsPipelineTargetInfo, IndexElementSize, LoadOp,
        PrimitiveType, RasterizerState, SampleCount, SamplerAddressMode, SamplerCreateInfo,
        SamplerMipmapMode, ShaderFormat, ShaderStage, StoreOp, Texture, TextureCreateInfo,
        TextureFormat, TextureRegion, TextureSamplerBinding, TextureTransferInfo, TextureType,
        TextureUsage, TransferBuffer, TransferBufferLocation, TransferBufferUsage, VertexAttribute,
        VertexBufferDescription, VertexElementFormat, VertexInputRate, VertexInputState,
    },
    keyboard::Keycode,
    pixels::Color,
    surface::Surface,
    Error,
};
use std::path::Path;
use crate::graphics::rendering::create_buffer_with_data;
use crate::core::{AppCore, Camera};
use crate::stars::*;
use crate::{graphics::rendering::*, resources::ResourceManager};
use cgmath::{Matrix4, PerspectiveFov, Rad, Vector3};
use log::info;
use sdl3::{ gpu::*, video::Window};


#[repr(packed)]
#[derive(Copy, Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
}

// Below are the vertices and indices that make up the 3D mesh of the cube.
const CUBE_VERTICES: &[Vertex] = &[
    Vertex {
        x: -0.5,
        y: -0.5,
        z: -0.5,
        u: 0.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: -0.5,
        z: -0.5,
        u: 1.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: -0.5,
        u: 1.0,
        v: 1.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: -0.5,
        u: 0.0,
        v: 1.0,
    },
    Vertex {
        x: -0.5,
        y: -0.5,
        z: 0.5,
        u: 0.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: -0.5,
        z: 0.5,
        u: 1.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: 0.5,
        u: 1.0,
        v: 1.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: 0.5,
        u: 0.0,
        v: 1.0,
    },
    Vertex {
        x: -0.5,
        y: -0.5,
        z: 0.5,
        u: 0.0,
        v: 0.0,
    },
    Vertex {
        x: -0.5,
        y: -0.5,
        z: -0.5,
        u: 1.0,
        v: 0.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: -0.5,
        u: 1.0,
        v: 1.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: 0.5,
        u: 0.0,
        v: 1.0,
    },
    Vertex {
        x: 0.5,
        y: -0.5,
        z: 0.5,
        u: 0.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: -0.5,
        z: -0.5,
        u: 1.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: -0.5,
        u: 1.0,
        v: 1.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: 0.5,
        u: 0.0,
        v: 1.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: 0.5,
        u: 0.0,
        v: 0.0,
    },
    Vertex {
        x: -0.5,
        y: 0.5,
        z: -0.5,
        u: 0.0,
        v: 1.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: 0.5,
        u: 1.0,
        v: 0.0,
    },
    Vertex {
        x: 0.5,
        y: 0.5,
        z: -0.5,
        u: 1.0,
        v: 1.0,
    },
];

const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, // front
    4, 5, 6, 4, 6, 7, // back
    8, 9, 10, 10, 11, 8, // left
    12, 13, 14, 14, 15, 12, // right
    16, 17, 18, 18, 19, 17, // top
        // not bothering with bottom since it's not visible
];


pub struct CubeRenderer {
    pipeline: GraphicsPipeline,
    rotation: f32,
}

impl CubeRenderer {
    pub fn load(
        device: &Device,
        window: &Window,
    ) -> Result<Self, Box<dyn std::error::Error>> {

        println!("meow");
        let fs_source = include_bytes!("../../shaders/cube/*.frag.spv");
        let vs_source = include_bytes!("../../shaders/cube/*.vert.spv");

        let vs_shader = device
            .create_shader()
            .with_code(ShaderFormat::SPIRV, vs_source, ShaderStage::Vertex)
            .with_entrypoint(c"main")
            .with_uniform_buffers(1)
            .build()?;

        let fs_shader = device
            .create_shader()
            .with_code(ShaderFormat::SPIRV, fs_source, ShaderStage::Fragment)
            .with_entrypoint(c"main")
            .with_samplers(1)
            .build()?;

        // Create a pipeline, we specify that we want our target format in the swapchain
        // since we are rendering directly to the screen. However, we could specify a texture
        // buffer instead (e.g., for offscreen rendering).
        let swapchain_format = device.get_swapchain_texture_format(&window);
        let pipeline = device
            .create_graphics_pipeline()
            .with_primitive_type(PrimitiveType::TriangleList)
            .with_fragment_shader(&fs_shader)
            .with_vertex_shader(&vs_shader)
            .with_vertex_input_state(
                VertexInputState::new()
                    .with_vertex_buffer_descriptions(&[VertexBufferDescription::new()
                        .with_slot(0)
                        .with_pitch(size_of::<Vertex>() as u32)
                        .with_input_rate(VertexInputRate::Vertex)
                        .with_instance_step_rate(0)])
                    .with_vertex_attributes(&[
                        VertexAttribute::new()
                            .with_format(VertexElementFormat::Float3)
                            .with_location(0)
                            .with_buffer_slot(0)
                            .with_offset(0),
                        VertexAttribute::new()
                            .with_format(VertexElementFormat::Float2)
                            .with_location(1)
                            .with_buffer_slot(0)
                            .with_offset((3 * size_of::<f32>()) as u32),
                    ]),
            )
            .with_rasterizer_state(
                RasterizerState::new()
                    .with_fill_mode(FillMode::Fill)
                    // Turn off culling so that I don't have to get my cube vertex order perfect
                    .with_cull_mode(CullMode::None),
            )
            .with_depth_stencil_state(
                // Enable depth testing
                DepthStencilState::new()
                    .with_enable_depth_test(true)
                    .with_enable_depth_write(true)
                    .with_compare_op(CompareOp::Less),
            )
            .with_target_info(
                GraphicsPipelineTargetInfo::new()
                    .with_color_target_descriptions(&[
                        ColorTargetDescription::new().with_format(swapchain_format)
                    ])
                    .with_has_depth_stencil_target(true)
                    .with_depth_stencil_format(TextureFormat::D16Unorm),
            )
            .build()?;

        // The pipeline now holds copies of our shaders, so we can release them
        drop(vs_shader);
        drop(fs_shader);

        // Next, we create a transfer buffer that is large enough to hold either
        // our vertices or indices since we will be transferring both with it.
        let vertices_len_bytes = std::mem::size_of_val(CUBE_VERTICES);
        let indices_len_bytes = std::mem::size_of_val(CUBE_INDICES);
        let transfer_buffer = device
            .create_transfer_buffer()
            .with_size(vertices_len_bytes.max(indices_len_bytes) as u32)
            .with_usage(TransferBufferUsage::UPLOAD)
            .build()?;

        // We need to start a copy pass in order to transfer data to the GPU
        let copy_commands = device.acquire_command_buffer()?;
        let copy_pass = device.begin_copy_pass(&copy_commands)?;

        // Create GPU buffers to hold our vertices and indices and transfer data to them
        let vertex_buffer = create_buffer_with_data(
            &device,
            &transfer_buffer,
            &copy_pass,
            BufferUsageFlags::VERTEX,
            CUBE_VERTICES,
        )?;
        let index_buffer = create_buffer_with_data(
            &device,
            &transfer_buffer,
            &copy_pass,
            BufferUsageFlags::INDEX,
            CUBE_INDICES,
        )?;

        // We're done with the transfer buffer now, so release it.
        drop(transfer_buffer);

        // let star_texture = resources.get_texture("star.png");

        // let texture_sampler = device.create_sampler(SamplerCreateInfo::new())?;

        // Now complete and submit the copy pass commands to actually do the transfer work
        device.end_copy_pass(copy_pass);
        copy_commands.submit()?;

        let (width, height) = window.size();

        // We'll need to allocate a texture buffer for our depth buffer for depth testing to work
        let mut depth_texture = device.create_texture(
            TextureCreateInfo::new()
                .with_type(TextureType::_2D)
                .with_width(width)
                .with_height(height)
                .with_layer_count_or_depth(1)
                .with_num_levels(1)
                .with_sample_count(SampleCount::NoMultiSampling)
                .with_format(TextureFormat::D16Unorm)
                .with_usage(TextureUsage::SAMPLER | TextureUsage::DEPTH_STENCIL_TARGET),
        )?;

        return Ok(Self {
            pipeline: pipeline,
            rotation: 45.0f32,
        });
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

        Ok(())
    }
}
