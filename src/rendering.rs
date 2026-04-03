use crate::stars::*;
use crate::util::GameUtils;
use imgui::Ui;
use imgui_sdl3::ImGuiSdl3;
use log::{error, info};
use sdl3::EventSubsystem;
use sdl3::event::EventSender;
use sdl3::{EventPump, Sdl, event::Event, gpu::*, pixels::Color, video::Window};

struct ShaderData {
    color: [f32; 4],
    rotation: f32,
    resolution: [f32; 2],
}

impl ShaderData {
    fn new(color: &[f32; 4], rotation: &f32, resolution: [f32; 2]) -> Self {
        ShaderData {
            color: color.clone(),
            rotation: rotation.clone(),
            resolution: resolution,
        }
    }
}

struct Camera {
    pos: [f64; 3],
    orientation: [f64; 3],
}

pub struct GameRenderer {
    window: Window,
    device: Device,
    imgui: ImGuiSdl3,
    triangle_pipeline: GraphicsPipeline,
}

impl GameRenderer {
    pub fn init(sdl: &Sdl) -> Result<Self, Box<dyn std::error::Error>> {
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

        return Ok(GameRenderer {
            window,
            device,
            imgui,
            triangle_pipeline,
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

        let mut rotation: f32 = 2.0;
        let mut color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        let mut window_size: [f32; 2] = [self.window.size().0 as f32, self.window.size().1 as f32];

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

            let shaderdata = ShaderData::new(&color, &rotation, window_size);

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

fn build_star_pipeline(
    device: &Device,
    window: &Window,
    star_handler: &StarHandler,
) -> Result<GraphicsPipeline, Box<dyn std::error::Error>> {
    // leave 1000 as a comfortable margin.
    let max_stars = star_handler.get_stars().len() + 1000;

    let buffer_size = (max_stars * size_of::<StarVertexData>()) as u32;

    // buffer which stores the stars
    let star_buffer = device
        .create_buffer()
        .with_size(buffer_size)
        .with_usage(BufferUsageFlags::COMPUTE_STORAGE_WRITE)
        .build()?;

    let fs_source = include_bytes!("../shaders/stars/stars.vert.spv");
    let vs_source = include_bytes!("../shaders/stars/stars.frag.spv");

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

    return Ok(pipeline);
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
    data: &ShaderData,
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

    // Map the transfer buffer's memory into a place we can copy into, and copy the data
    //
    // Note: We set `cycle` to true since we're reusing the same transfer buffer to
    // initialize both the vertex and index buffer. This makes SDL synchronize the transfers
    // so that one doesn't interfere with the other.
    let mut map = transfer_buffer.map::<T>(gpu, true);
    let mem = map.mem_mut();
    for (index, &value) in data.iter().enumerate() {
        mem[index] = value;
    }

    // Now unmap the memory since we're done copying
    map.unmap();

    // Finally, add a command to the copy pass to upload this data to the GPU
    //
    // Note: We also set `cycle` to true here for the same reason.
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
