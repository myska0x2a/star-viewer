//! Rendering manager.
use crate::core::{AppCore, Camera};
use crate::graphics::navcube::CubeRenderer;
use crate::graphics::star_renderer::StarRenderer;
use crate::resources::ResourceManager;
use crate::stars::*;
use crate::ui::AppUi;

use imgui::sys::{
    ImGuiCol_, ImGuiCol_Border, ImGuiCol_Button, ImGuiCol_ButtonActive, ImGuiCol_ButtonHovered,
    ImGuiCol_CheckMark, ImGuiCol_FrameBg, ImGuiCol_FrameBgActive, ImGuiCol_FrameBgHovered,
    ImGuiCol_Header, ImGuiCol_ModalWindowDimBg, ImGuiCol_SliderGrab, ImGuiCol_SliderGrabActive,
    ImGuiCol_Tab, ImGuiCol_TitleBgActive,
};
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
    pub fn init(sdl: &Sdl, appcore: &AppCore) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Init renderer");

        let mut window: Window = sdl
            .video()?
            .window("stars-app", 1000, 1000)
            .fullscreen()
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let device: Device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&mut window)?;

        let mut imgui: ImGuiSdl3 = ImGuiSdl3::new(&device, &window, |ctx: &mut imgui::Context| {
            ctx.set_ini_filename(None);
            ctx.set_log_filename(None);

            let font: imgui::FontSource<'_> = imgui::FontSource::TtfData {
                data: include_bytes!("../../assets/Neco-Medium.otf"),
                size_pixels: 20.0,
                config: None,
            };

            ctx.fonts()
                // .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
                .add_font(&[font]);

            let style: &mut imgui::Style = ctx.style_mut();
            style.frame_rounding = 7.0;
            style.grab_rounding = 7.0;
            style.window_rounding = 10.0;
            style.tab_border_size = 0.5;

            let colour: &mut [[f32; 4]; 53] = &mut style.colors;

            let primary: [f32; 4] = [202.0 / 255.0, 0.0, 0.0, 255.0 / 255.0];

            let el_bg: [f32; 4] = [74.0 / 255.0, 74.0 / 255.0, 74.0 / 255.0, 138.0 / 255.0];
            let el_hv: [f32; 4] = [116.0 / 255.0, 116.0 / 255.0, 116.0 / 255.0, 102.0 / 255.0];
            let el_ac: [f32; 4] = [0.606, 0.606, 0.606, 0.4];

            colour[ImGuiCol_FrameBg as usize] = el_bg;
            colour[ImGuiCol_FrameBgHovered as usize] = el_hv;
            colour[ImGuiCol_FrameBgActive as usize] = el_ac;
            colour[ImGuiCol_SliderGrab as usize] = el_ac;
            colour[ImGuiCol_SliderGrabActive as usize] = primary;

            colour[ImGuiCol_Button as usize] = el_bg;
            colour[ImGuiCol_ButtonHovered as usize] = el_hv;
            colour[ImGuiCol_ButtonActive as usize] = primary;

            colour[ImGuiCol_CheckMark as usize] = primary;

            colour[ImGuiCol_TitleBgActive as usize] = primary;
            colour[ImGuiCol_Header as usize] = primary;
            colour[ImGuiCol_Tab as usize] = primary;

            colour[ImGuiCol_Border as usize] = primary;

            colour[ImGuiCol_ModalWindowDimBg as usize] = [0.637, 0.368, 0.368, 0.350];
        });

        let mouse: sdl3::mouse::MouseUtil = sdl.mouse();
        mouse.set_relative_mouse_mode(&window, true);

        let mut star_renderer: StarRenderer<'_> = StarRenderer::load(&device, &window, appcore)?;
        let mut cube_renderer: CubeRenderer = CubeRenderer::load(&device, &window)?;

        Ok(Self {
            window,
            device,
            imgui,
            star_renderer,
            cube_renderer,
            mousefocus: true,
        })
    }

    pub fn render(
        &mut self,
        sdl: &mut Sdl,
        resources: &ResourceManager,
        appcore: &mut AppCore,
        appui: &mut AppUi,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut event: sdl3::EventPump = sdl.event_pump()?;
        let event_pump: &mut sdl3::EventPump = &mut event;

        let mut command_buffer: CommandBuffer = self.device.acquire_command_buffer()?;

        if let Ok(swapchain) = command_buffer.wait_and_acquire_swapchain_texture(&self.window) {
            let imgui_color_target: [ColorTargetInfo; 1] = [ColorTargetInfo::default()
                .with_texture(&swapchain)
                .with_load_op(LoadOp::LOAD)
                .with_store_op(StoreOp::STORE)
                .with_clear_color(Color::RGB(128, 128, 128))];

            let star_color_target: [ColorTargetInfo; 1] = [ColorTargetInfo::default()
                .with_texture(&swapchain)
                .with_load_op(LoadOp::CLEAR)
                .with_store_op(StoreOp::STORE)
                .with_clear_color(Color::RGB(0, 0, 0))];

            let camera: Camera = appcore.camera.clone();

            self.star_renderer.render(
                &self.device,
                &self.window,
                &mut command_buffer,
                &star_color_target,
                &camera,
                resources,
            )?;

            self.imgui.render(
                sdl,
                &self.device,
                &self.window,
                &event_pump,
                &mut command_buffer,
                &imgui_color_target,
                appui.build_ui(appcore),
            );

            command_buffer.submit()?;
        } else {
            error!("Renderer: swapchain unavailable.");
            command_buffer.cancel();
        };

        Ok(())
    }

    pub fn close(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut command_buffer: CommandBuffer = self.device.acquire_command_buffer()?;
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
    let len_bytes: usize = std::mem::size_of_val(data);

    let buffer: Buffer = gpu
        .create_buffer()
        // plus four to ensure it never goes below the minimum
        // buffer size (of 4)
        .with_size(len_bytes as u32 + 4)
        .with_usage(usage)
        .build()?;

    let mut map: BufferMemMap<'_, T> = transfer_buffer.map::<T>(gpu, true);
    let mem: &mut [T] = map.mem_mut();
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
