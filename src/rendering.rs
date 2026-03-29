use crate::stars::*;
use crate::util::GameUtils;
use imgui_sdl3::ImGuiSdl3;
use imgui::Ui;
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

pub struct GameRenderer {
    window: Window,
    device: Device,
    imgui: ImGuiSdl3,
}

impl GameRenderer {
    pub fn init(sdl: &Sdl) -> Result<Self, Box<dyn std::error::Error>> {
        let video_subsystem = sdl.video()?;

        // create a new window
        let mut window = video_subsystem
            .window("Hello imgui-rs!", 1000, 1000)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&mut window)?;

        let mut imgui = ImGuiSdl3::new(&device, &window, |ctx| {
            // disable creation of files on disc
            ctx.set_ini_filename(None);
            ctx.set_log_filename(None);

            // setup platform and renderer, and fonts to imgui
            ctx.fonts()
                .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        });

        return Ok(GameRenderer {
            window,
            device,
            imgui,
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
                &self.window,
                &command_buffer,
                &color_targets,
                &shaderdata,
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
            println!("Swapchain unavailable, cancel work");
            command_buffer.cancel();
        }

        Ok(())
    }
}

fn render_triangle(
    device: &Device,
    window: &Window,
    command_buffer: &CommandBuffer,
    color_targets: &[ColorTargetInfo; 1],
    data: &ShaderData,
) {
    let fs_source = include_bytes!("../shaders/triangle.frag.spv");
    let vs_source = include_bytes!("../shaders/triangle.vert.spv");

    // Our shaders, require to be precompiled by a SPIR-V compiler beforehand
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

    let render_pass = device
        .begin_render_pass(&command_buffer, color_targets, None)
        .unwrap();
    render_pass.bind_graphics_pipeline(&pipeline);

    command_buffer.push_vertex_uniform_data(0, data);

    render_pass.draw_primitives(3, 1, 0, 0);
    device.end_render_pass(render_pass);
}
