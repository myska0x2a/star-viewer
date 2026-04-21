//! Rendering manager.
use crate::event::*;
use crate::graphics::star_renderer::StarRenderer;
use crate::resources::ResourceManager;
use crate::stars::*;

use log::{error, info};
use std::marker::Copy;

use imgui::Ui;
use imgui_sdl3::ImGuiSdl3;

use sdl3::keyboard::Keycode::*;
use sdl3::{Sdl, event::Event, gpu::*, pixels::Color, video::Window};

#[derive(Copy, Clone)]
pub struct Camera {
    pub pos: [f32; 3],
    pub orientation: [f32; 3],
    pub velocity: [f32; 3],
    pub fov: f32,
}

impl Camera {
    pub fn increment_pos(&mut self, x: f32, y: f32, z: f32) {
        self.orientation[0] += x;
        self.orientation[1] += y;
        self.orientation[2] += z;
    }

    pub fn increment_vel(&mut self) {
        self.pos[0] += self.velocity[0];
        self.pos[1] += self.velocity[1];
        self.pos[2] += self.velocity[2];
    }
}

impl Default for Camera {
    fn default() -> Self {
        return Camera {
            pos: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            fov: 3.0,
        };
    }
}

pub struct GameRenderer<'a> {
    pub window: Window,
    pub device: Device,
    imgui: ImGuiSdl3,
    pub star_renderer: StarRenderer<'a>,
    pub camera: Camera,
    pub mousefocus: bool,
}

impl<'a> GameRenderer<'a> {
    pub fn init(sdl: &Sdl, star_handler: &StarHandler) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Init renderer");

        let video_subsystem = sdl.video()?;

        let mut window = video_subsystem
            .window("stars-game", 1000, 1000)
            .fullscreen()
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&mut window)?;

        let mut imgui = ImGuiSdl3::new(&device, &window, |ctx| {
            // disable creation of files
            ctx.set_ini_filename(None);
            ctx.set_log_filename(None);

            let font = imgui::FontSource::TtfData { data: include_bytes!("../../assets/ShareTechMono-Regular.ttf"), size_pixels: 20.0, config: None };

            ctx.fonts()
                .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
                // .add_font(&[font]);

        });

        let mouse = sdl.mouse();
        mouse.set_relative_mouse_mode(&window, true);

        let mut star_renderer = StarRenderer::load(&device, &window, star_handler.clone())?;
        // star_renderer.reload(&device, &window, 10.0)?;

        return Ok(GameRenderer {
            window,
            device,
            imgui,
            star_renderer,
            camera: Camera::default(),
            mousefocus: true,
        });
    }

    pub fn render(
        &mut self,
        sdl: &mut Sdl,
        resources: &ResourceManager,
        ui_callback: impl FnMut(&mut Ui),
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
                .with_load_op(LoadOp::CLEAR)
                .with_store_op(StoreOp::STORE)
                .with_clear_color(Color::RGB(0, 0, 0))];

            self.star_renderer.render(
                &self.device,
                &self.window,
                &mut command_buffer,
                &star_color_target,
                &mut self.camera,
                resources,
            )?;

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

    pub fn handle_ui_event(&mut self, event: &Event) -> Result<(), Box<dyn std::error::Error>> {
        self.imgui.handle_event(&event);

        if self.mousefocus {
            match event {
                Event::KeyDown { keycode, .. } => {
                    // x
                    if keycode == &Some(A) {
                        self.camera.velocity[0] = -0.1;
                    }
                    if keycode == &Some(D) {
                        self.camera.velocity[0] = 0.1;
                    }
                    // y
                    if keycode == &Some(W) {
                        self.camera.velocity[1] = -0.1;
                    }
                    if keycode == &Some(S) {
                        self.camera.velocity[1] = 0.1;
                    }
                    // z
                    if keycode == &Some(E) {
                        self.camera.velocity[2] = 0.1;
                    }
                    if keycode == &Some(Q) {
                        self.camera.velocity[2] = -0.1;
                    }
                }

                Event::KeyUp { .. } => {
                    self.camera.velocity = [0.0, 0.0, 0.0];
                }

                Event::MouseMotion { xrel, yrel, .. } => {
                    self.camera.orientation[2] += xrel / 400.0;
                    self.camera.orientation[1] += yrel / 400.0;
                }

                // Event::MouseWheel { x, y, .. } => {
                //     let new_fov = self.camera.fov + (y / 10.0);
                //     self.camera.fov = new_fov.clamp(0.0000001, 3.14);
                //     if x != &0.0f32 {
                //         self.star_renderer.range =
                //             (self.star_renderer.range + x).clamp(0.2, 9999999.0);
                //         self.star_renderer.reload(
                //             &self.device,
                //             &self.window,
                //             self.star_renderer.range,
                //         )?;
                //     }
                // }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn handle_game_event(
        &mut self,
        event: GameEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            GameEvent::PositionChanged(position) => {
                self.camera.pos[0] = position;
                Ok(())
            }
            _ => Ok(()),
        }
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
