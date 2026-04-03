use crate::stars::*;
use crate::util::GameUtils;
use imgui::Ui;
use imgui_sdl3::ImGuiSdl3;
use log::{error, info};
use sdl3::EventSubsystem;
use sdl3::event::EventSender;
use sdl3::{EventPump, Sdl, event::Event, gpu::*, pixels::Color, video::Window};

struct Camera {
    pos: [f64; 3],
    orientation: [f64; 3],
}

pub struct GameRenderer {
    window: Window,
    device: Device,
    imgui: ImGuiSdl3,
    triangle_pipeline: GraphicsPipeline,
    star_pipeline: GraphicsPipeline,
}

impl GameRenderer {
    pub fn init(sdl: &Sdl, star_handler: &StarHandler) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Init renderer");

        let video_subsystem = sdl.video()?;

        let mut window = video_subsystem
            .window("Hello imgui-rs!", 1000, 1000)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&mut window)?;

        let mut imgui = ImGuiSdl3::new(&device, &window, |ctx| {
            // disable creation of files
            ctx.set_ini_filename(None);
            ctx.set_log_filename(None);

            ctx.fonts()
                .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        });

        let triangle_pipeline = build_triangle_pipeline(&device, &window);
        let star_pipeline = build_star_pipeline(&device, &window, star_handler)?;

        return Ok(GameRenderer {
            window,
            device,
            imgui,
            triangle_pipeline,
            star_pipeline,
        });
    }

    pub fn handle_ui_event(&mut self, event: &Event) {
        self.imgui.handle_event(&event);
    }

    pub fn render(
        &mut self,
        sdl: &mut Sdl,
        ui_callback: impl FnMut(&mut Ui),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut event = sdl.event_pump()?;
        let event_pump = &mut event;


        let mut command_buffer = self.device.acquire_command_buffer()?;

        let mut star_radius = 0.0;

        if let Ok(swapchain) = command_buffer.wait_and_acquire_swapchain_texture(&self.window) {
            let color_targets = [ColorTargetInfo::default()
                .with_texture(&swapchain)
                .with_load_op(LoadOp::LOAD)
                .with_store_op(StoreOp::STORE)
                .with_clear_color(Color::RGB(128, 128, 128))];

            let triangle_color_target = [ColorTargetInfo::default()
                .with_texture(&swapchain)
                .with_load_op(LoadOp::CLEAR)
                .with_store_op(StoreOp::STORE)
                .with_clear_color(Color::RGB(0, 0, 0))];

            let rotation: f32 = 2.0;
            let color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
            let window_size: [f32; 2] = [self.window.size().0 as f32, self.window.size().1 as f32];

            let shaderdata = TriangleUniforms::new(&color, &rotation, window_size);

            render_triangle(
                &self.device,
                &command_buffer,
                &color_targets,
                &shaderdata,
                &self.triangle_pipeline,
            );

            self.imgui.render(
                sdl,
                &self.device,
                &self.window,
                &event_pump,
                &mut command_buffer,
                &color_targets,
                ui_callback,
            );

            command_buffer.submit()?;
        } else {
            error!("Renderer: swapchain unavailable.");
            command_buffer.cancel();
        }

        Ok(())
    }

    pub fn close(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut command_buffer = self.device.acquire_command_buffer()?;
        command_buffer.cancel();
        drop(command_buffer);
        drop(self);
        Ok(())
    }
}

// data to be passed to the shader.
#[repr(packed)]
#[derive(Copy, Clone)]
struct StarVertexData {
    position: [f32; 3],
    temperature: f32,
    magnitude: f32,
}

impl From<&Star> for StarVertexData {
    fn from(star: &Star) -> StarVertexData {
        let position = [star.x as f32, star.y as f32, star.z as f32];

        return StarVertexData { position, temperature: 2000.0, magnitude: 1.0 }
    }
}

fn build_star_pipeline(
    device: &Device,
    window: &Window,
    star_handler: &StarHandler,
) -> Result<GraphicsPipeline, Box<dyn std::error::Error>> {
    let stars = star_handler.get_nearby(100.0);
    // leave 1000 as a comfortable margin.
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


    let fs_source = include_bytes!("../shaders/stars/stars.frag.spv");
    let vs_source = include_bytes!("../shaders/stars/stars.vert.spv");

    // Our shaders, require to be precompiled by a SPIR-V compiler beforehand
    let vs_shader = device
        .create_shader()
        .with_code(ShaderFormat::SPIRV, vs_source, ShaderStage::Vertex)
        .with_entrypoint(c"main")
        .build()?;

    let fs_shader = device
        .create_shader()
        .with_code(ShaderFormat::SPIRV, fs_source, ShaderStage::Fragment)
        .with_entrypoint(c"main")
        .build()?;

    let swapchain_format = device.get_swapchain_texture_format(&window);

    // Create a pipeline, we specify that we want our target format in the one of the swapchain
    // since we are rendering directly unto the swapchain, however, we could specify one that
    // is different from the swapchain (i.e offscreen rendering)
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

    // The pipeline now holds copies of our shaders, so we can release them
    drop(vs_shader);
    drop(fs_shader);

    return Ok(pipeline)
}

fn render_stars(
    device: &Device,
    window: &Window,
    command_buffer: &CommandBuffer,
    color_targets: &[ColorTargetInfo; 1],
    star_handler: &StarHandler,
) {

    // let stars = star_handler.get_stars();
}


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

// https://github.com/vhspace/sdl3-rs/blob/master/examples/gpu-cube.rs
fn create_buffer_with_data<T: Copy>(
    gpu: &Device,
    transfer_buffer: &TransferBuffer,
    copy_pass: &CopyPass,
    usage: BufferUsageFlags,
    data: &[T],
) -> Result<Buffer, Box<dyn std::error::Error>> {
    // Figure out the length of the data in bytes
    let len_bytes = std::mem::size_of_val(data);

    // Create the buffer with the size and usage we want
    let buffer = gpu
        .create_buffer()
        .with_size(len_bytes as u32)
        .with_usage(usage)
        .build()?;

    let mut map = transfer_buffer.map::<T>(gpu, true);
    let mem = map.mem_mut();
    for (index, &value) in data.iter().enumerate() {
        mem[index] = value;
    }

    map.unmap();

    copy_pass.upload_to_gpu_buffer(
        TransferBufferLocation::new()
            .with_offset(0)
            .with_transfer_buffer(transfer_buffer),
        BufferRegion::new()
            .with_offset(0)
            .with_size(len_bytes as u32)
            .with_buffer(&buffer),
        true,
    );

    Ok(buffer)
}
