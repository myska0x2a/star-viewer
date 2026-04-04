use crate::stars::*;
use crate::util::GameUtils;
use imgui::Ui;
use imgui_sdl3::ImGuiSdl3;
use log::{error, info};
use sdl3::EventSubsystem;
use sdl3::event::EventSender;
use sdl3::{EventPump, Sdl, event::Event, gpu::*, pixels::Color, video::Window};
use cgmath::{ PerspectiveFov, Rad };

struct Camera {
    pos: [f64; 3],
    orientation: [f64; 3],
}

pub struct GameRenderer {
    window: Window,
    device: Device,
    imgui: ImGuiSdl3,
    star_pipeline: GraphicsPipeline,
    star_buffer: Buffer,
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

        let (star_pipeline, star_buffer) = build_star_pipeline(&device, &window, star_handler)?;

        return Ok(GameRenderer {
            window,
            device,
            imgui,
            star_pipeline,
            star_buffer,
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


            render_stars(
                &self.device,
                &self.window,
                &command_buffer,
                &triangle_color_target,
                &self.star_pipeline,
                &self.star_buffer,
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

fn build_star_pipeline(
    device: &Device,
    window: &Window,
    star_handler: &StarHandler,
) -> Result<(GraphicsPipeline, Buffer), Box<dyn std::error::Error>> {
    let stars = star_handler.get_nearby(10.0);

    let max_stars = stars.len();

    let mut star_data: Vec<StarVertexData> = Vec::new();

    for star in stars {
        star_data.push(StarVertexData::from(star));
        println!("{}", star.name());
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

    return Ok((pipeline, star_buffer));
}

fn render_stars(
    device: &Device,
    window: &Window,
    command_buffer: &CommandBuffer,
    color_targets: &[ColorTargetInfo; 1],
    pipeline: &GraphicsPipeline,
    star_buffer: &Buffer,
) {
    let rotation = Rad(30.0);
    let projection_matrix = PerspectiveFov {
        fovy: rotation,
        aspect: 1.7,
        near: 1.0,
        far: 3.0,
    };

    let render_pass = device
        .begin_render_pass(&command_buffer, color_targets, None)
        .unwrap();

    render_pass.bind_graphics_pipeline(pipeline);
    render_pass.bind_vertex_storage_buffers(0, &[star_buffer.clone()]);
    command_buffer.push_vertex_uniform_data(0, &projection_matrix);
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
