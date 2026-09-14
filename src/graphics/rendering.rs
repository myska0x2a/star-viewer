//! Rendering manager.
use crate::core::{AppCore, Camera};
use crate::graphics::navcube::CubeRenderer;
use crate::graphics::star_renderer::StarRenderer;
use crate::resources::ResourceManager;
use crate::stars::*;
use crate::ui::AppUi;

use log::{error, info};
use std::marker::Copy;

use imgui::Ui;
use imgui_sdl3::ImGuiSdl3;

use sdl3::keyboard::Keycode::*;
use sdl3::{Sdl, event::Event, gpu::*, pixels::Color, video::Window};

pub struct AppRenderer<'a> {
    pub window: Window,
    pub device: Device,
    pub imgui: ImGuiSdl3,
    pub star_renderer: StarRenderer<'a>,
    pub cube_renderer: CubeRenderer,
    pub mousefocus: bool,
}

impl<'a> AppRenderer<'a> {
    pub fn init(sdl: &Sdl, star_handler: &StarHandler) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Init renderer");

        let video_subsystem = sdl.video()?;

        let mut window = video_subsystem
            .window("stars-app", 1000, 1000)
            .fullscreen()
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&mut window)?;

        let mut imgui = ImGuiSdl3::new(&device, &window, |ctx| {
            ctx.set_ini_filename(None);
            ctx.set_log_filename(None);

            let font = imgui::FontSource::TtfData {
                data: include_bytes!("../../assets/ST-Spartak.otf"),
                size_pixels: 20.0,
                config: None,
            };

            ctx.fonts()
                .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
            // .add_font(&[font]);
        });

        let mouse = sdl.mouse();
        mouse.set_relative_mouse_mode(&window, true);

        let mut star_renderer = StarRenderer::load(&device, &window, star_handler)?;
        let mut cube_renderer = CubeRenderer::load(&device, &window)?;

        return Ok(AppRenderer {
            window,
            device,
            imgui,
            star_renderer,
            cube_renderer,
            mousefocus: true,
        });
    }

    pub fn render(
        &mut self,
        sdl: &mut Sdl,
        resources: &ResourceManager,
        appcore: &mut AppCore,
        appui: &mut AppUi,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut event = sdl.event_pump()?;
        let event_pump = &mut event;

        let mut command_buffer = self.device.acquire_command_buffer()?;

        if let Ok(swapchain) = command_buffer.wait_and_acquire_swapchain_texture(&self.window) {
            let imgui_color_target = [ColorTargetInfo::default()
                .with_texture(&swapchain)
                .with_load_op(LoadOp::LOAD)
                .with_store_op(StoreOp::STORE)
                .with_clear_color(Color::RGB(128, 128, 128))];

            let star_color_target = [ColorTargetInfo::default()
                .with_texture(&swapchain)
                .with_load_op(LoadOp::LOAD)
                .with_store_op(StoreOp::STORE)
                .with_clear_color(Color::RGB(128, 128, 128))];

            let camera = appcore.camera.clone();

            self.star_renderer.render(
                &self.device,
                &self.window,
                &mut command_buffer,
                &star_color_target,
                &camera,
                resources,
            )?;

            let ui_callback = appui.build_ui(appcore);

            self.imgui.render(
                sdl,
                &self.device,
                &self.window,
                &event_pump,
                &mut command_buffer,
                &imgui_color_target,
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

// https://github.com/vhspace/sdl3-rs/blob/master/examples/gpu-cube.rs
pub fn create_buffer_with_data<T: Copy>(
    gpu: &Device,
    transfer_buffer: &TransferBuffer,
    copy_pass: &CopyPass,
    usage: BufferUsageFlags,
    data: &[T],
) -> Result<Buffer, Box<dyn std::error::Error>> {
    let len_bytes = std::mem::size_of_val(data);

    let buffer = gpu
        .create_buffer()
        // plus four to ensure it never goes below the minimum
        // buffer size (of 4)
        .with_size(len_bytes as u32 + 4)
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
