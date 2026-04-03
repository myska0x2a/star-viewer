// let rotation: f32 = 2.0;
// let color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
// let window_size: [f32; 2] = [self.window.size().0 as f32, self.window.size().1 as f32];

// let shaderdata = TriangleUniforms::new(&color, &rotation, window_size);

// render_triangle(
//     &self.device,
//     &command_buffer,
//     &color_targets,
//     &shaderdata,
//     &self.triangle_pipeline,
// );

struct TriangleUniforms {
    color: [f32; 4],
    rotation: f32,
    resolution: [f32; 2],
}

impl TriangleUniforms {
    fn new(color: &[f32; 4], rotation: &f32, resolution: [f32; 2]) -> Self {
        TriangleUniforms {
            color: color.clone(),
            rotation: rotation.clone(),
            resolution: resolution,
        }
    }
}

fn build_triangle_pipeline(device: &Device, window: &Window) -> GraphicsPipeline {
    let fs_source = include_bytes!("../shaders/triangle/triangle.frag.spv");
    let vs_source = include_bytes!("../shaders/triangle/triangle.vert.spv");

    let vs_shader = device
        .create_shader()
        .with_code(ShaderFormat::SPIRV, vs_source, ShaderStage::Vertex)
        .with_entrypoint(c"main")
        .with_uniform_buffers(1)
        .build()
        .unwrap();

    let fs_shader = device
        .create_shader()
        .with_code(ShaderFormat::SPIRV, fs_source, ShaderStage::Fragment)
        .with_entrypoint(c"main")
        .build()
        .unwrap();

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
        .build()
        .unwrap();

    drop(vs_shader);
    drop(fs_shader);

    return pipeline;
}

fn render_triangle(
    device: &Device,
    command_buffer: &CommandBuffer,
    color_targets: &[ColorTargetInfo; 1],
    data: &TriangleUniforms,
    pipeline: &GraphicsPipeline,
) {
    let render_pass = device
        .begin_render_pass(&command_buffer, color_targets, None)
        .unwrap();
    render_pass.bind_graphics_pipeline(&pipeline);

    command_buffer.push_vertex_uniform_data(0, data);

    render_pass.draw_primitives(3, 1, 0, 0);
    device.end_render_pass(render_pass);
}


