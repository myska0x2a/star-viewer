use crate::event::*;
use crate::stars::*;
use crate::util::GameUtils;
use cgmath::{Matrix4, PerspectiveFov, Rad};
use imgui::Ui;
use imgui_sdl3::ImGuiSdl3;
use log::{error, info};
use sdl3::EventSubsystem;
use sdl3::event::EventSender;
use sdl3::keyboard::Keycode::*;
use sdl3::surface::Surface;
use sdl3::video::ProgressState;
use sdl3::{EventPump, Sdl, event::Event, gpu::*, pixels::Color, video::Window};
use std::f32::consts::PI;
use std::path::Path;

#[derive(Copy, Clone)]
struct Camera {
    pos: [f32; 3],
    orientation: [f32; 3],
    velocity: [f32; 3],
    fov: f32,
}

impl Camera {
    fn increment_pos(&mut self, x: f32, y: f32, z: f32) {
        self.orientation[0] += x;
        self.orientation[1] += y;
        self.orientation[2] += z;
    }

    fn increment_vel(&mut self) {
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
    window: Window,
    device: Device,
    imgui: ImGuiSdl3,
    star_pipeline: GraphicsPipeline,
    star_buffer: Buffer,
    star_texture: Texture<'a>,
    star_sampler: Sampler,
    num_stars: usize,
    camera: Camera,
    pub range: f32,
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

            ctx.fonts()
                .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
        });

        let (star_pipeline, star_sampler, star_texture) =
            Self::build_star_pipeline(&device, &window)?;
        let (star_buffer, num_stars) = Self::load_star_buffer(&device, &window, star_handler, 20.0)?;

        return Ok(GameRenderer {
            window,
            device,
            imgui,
            star_pipeline,
            star_buffer,
            star_texture,
            star_sampler,
            num_stars,
            camera: Camera::default(),
            range: 20.0,
        });
    }


    pub fn render(
        &mut self,
        sdl: &mut Sdl,
        ui_callback: impl FnMut(&mut Ui),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut event = sdl.event_pump()?;
        let event_pump = &mut event;

        let mut command_buffer = self.device.acquire_command_buffer()?;

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

            self.render_stars(
                &command_buffer,
                &triangle_color_target,
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

    fn build_star_pipeline (
        device: &Device,
        window: &Window,
    ) -> Result<(GraphicsPipeline, Sampler, Texture<'a>), Box<dyn std::error::Error>> {
        let fs_source = include_bytes!("../shaders/stars/stars.frag.spv");
        let vs_source = include_bytes!("../shaders/stars/stars.vert.spv");

        // info!("{}", size_of::<Buffer>());

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
            // .with_samplers(1)
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

        let copy_commands = device.acquire_command_buffer()?;
        let copy_pass = device.begin_copy_pass(&copy_commands)?;

        // Load up a texture to put on the cube
        let star_texture = create_texture_from_image(&device, "./star.bmp", &copy_pass)?;

        // And configure a sampler for pulling pixels from that texture in the frag shader
        let star_texture_sampler = device.create_sampler(
            SamplerCreateInfo::new()
                .with_min_filter(Filter::Nearest)
                .with_mag_filter(Filter::Nearest)
                .with_mipmap_mode(SamplerMipmapMode::Nearest)
                .with_address_mode_u(SamplerAddressMode::Repeat)
                .with_address_mode_v(SamplerAddressMode::Repeat)
                .with_address_mode_w(SamplerAddressMode::Repeat),
        )?;

        return Ok((pipeline, star_texture_sampler, star_texture));
    }

    fn render_stars(
        &mut self,
        command_buffer: &CommandBuffer,
        color_targets: &[ColorTargetInfo; 1],
    ) {
        self.camera.increment_vel();

        #[repr(align(16))]
        #[derive(Copy, Clone)]
        struct UniformData {
            projection_matrix: Matrix4<f32>,
            position: [f32; 3],
            rotation: [f32; 3],
        }

        let window_size = self.window.size();

        let fov = Rad(self.camera.fov);
        let projection_matrix = PerspectiveFov {
            fovy: fov,
            aspect: window_size.0 as f32 / window_size.1 as f32,
            near: 0.1,
            far: 300.0,
        };

        let uniform_data = UniformData {
            projection_matrix: Matrix4::from(projection_matrix),
            position: self.camera.pos,
            rotation: self.camera.orientation,
        };

        let render_pass = self.device
            .begin_render_pass(&command_buffer, color_targets, None)
            .unwrap();

        let buffer = self.star_buffer.clone();

        render_pass.bind_graphics_pipeline(&self.star_pipeline);
        render_pass.bind_vertex_storage_buffers(0, &[buffer]);

        // render_pass.bind_fragment_samplers(
        //         0,
        //         &[TextureSamplerBinding::new()
        //             .with_texture(&star_texture)
        //             .with_sampler(&star_sampler)],
        // );

        command_buffer.push_vertex_uniform_data(0, &uniform_data);
        render_pass.draw_primitives(self.num_stars * 6, self.num_stars * 2, 0, 0);

        self.device.end_render_pass(render_pass);
    }

    pub fn reload_stars(
        &mut self,
        star_handler: &StarHandler,
        range: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        (self.star_buffer, self.num_stars) =
            Self::load_star_buffer(&self.device, &self.window, star_handler, range)?;
        Ok(())
    }

    fn load_star_buffer(
        device: &Device,
        window: &Window,
        star_handler: &StarHandler,
        range: f32,
    ) -> Result<(Buffer, usize), Box<dyn std::error::Error>> {
        let stars = star_handler.get_nearby(range as f64);

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

        return Ok((star_buffer, max_stars));
    }

    pub fn handle_ui_event(&mut self, event: &Event) {
        self.imgui.handle_event(&event);

        match event {
            Event::KeyDown {
                timestamp,
                window_id,
                keycode,
                scancode,
                keymod,
                repeat,
                which,
                raw,
            } => {
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

            Event::MouseMotion {
                timestamp,
                window_id,
                which,
                mousestate,
                x,
                y,
                xrel,
                yrel,
            } => {
                self.camera.orientation[2] += xrel / 200.0;
                self.camera.orientation[1] += yrel / 200.0;
            }

            Event::KeyUp { .. } => {
                self.camera.velocity = [0.0, 0.0, 0.0];
            }

            Event::MouseWheel {
                timestamp,
                window_id,
                which,
                x,
                y,
                direction,
                mouse_x,
                mouse_y,
            } => {
                let new_fov = self.camera.fov + (y / 10.0);
                self.camera.fov = new_fov.clamp(0.0000001, 3.14)
            }

            _ => {}
        }
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

// https://github.com/vhspace/sdl3-rs/blob/master/examples/gpu-cube.rs
// fn create_buffer_with_data<T: Copy>(
//     gpu: &Device,
//     transfer_buffer: &TransferBuffer,
//     copy_pass: &CopyPass,
//     usage: BufferUsageFlags,
//     data: &[T],
// ) -> Result<Buffer, Box<dyn std::error::Error>> {
//     let len_bytes = std::mem::size_of_val(data);

//     let buffer = gpu
//         .create_buffer()
//         .with_size(len_bytes as u32)
//         .with_usage(usage)
//         .build()?;

//     let mut map = transfer_buffer.map::<T>(gpu, true);
//     let mem = map.mem_mut();
//     for (index, &value) in data.iter().enumerate() {
//         mem[index] = value;
//     }

//     map.unmap();

//     copy_pass.upload_to_gpu_buffer(
//         TransferBufferLocation::new()
//             .with_offset(0)
//             .with_transfer_buffer(transfer_buffer),
//         BufferRegion::new()
//             .with_offset(0)
//             .with_size(len_bytes as u32)
//             .with_buffer(&buffer),
//         true,
//     );

//     Ok(buffer)
// }


fn create_texture_from_image(
    gpu: &Device,
    image_path: impl AsRef<Path>,
    copy_pass: &CopyPass,
) -> Result<Texture<'static>, Box<dyn std::error::Error>> {
    let image = Surface::load_bmp(image_path.as_ref())?;
    let image_size = image.size();
    let size_bytes = image.pixel_format().bytes_per_pixel() as u32 * image_size.0 * image_size.1;

    let texture = gpu.create_texture(
        TextureCreateInfo::new()
            .with_format(TextureFormat::R8g8b8a8Unorm)
            .with_type(TextureType::_2D)
            .with_width(image_size.0)
            .with_height(image_size.1)
            .with_layer_count_or_depth(1)
            .with_num_levels(1)
            .with_usage(TextureUsage::SAMPLER),
    )?;

    let transfer_buffer = gpu
        .create_transfer_buffer()
        .with_size(size_bytes)
        .with_usage(TransferBufferUsage::UPLOAD)
        .build()?;

    let mut buffer_mem = transfer_buffer.map::<u8>(gpu, false);
    image.with_lock(|image_bytes| {
        buffer_mem.mem_mut().copy_from_slice(image_bytes);
    });
    buffer_mem.unmap();

    copy_pass.upload_to_gpu_texture(
        TextureTransferInfo::new()
            .with_transfer_buffer(&transfer_buffer)
            .with_offset(0),
        TextureRegion::new()
            .with_texture(&texture)
            .with_layer(0)
            .with_width(image_size.0)
            .with_height(image_size.1)
            .with_depth(1),
        false,
    );

    Ok(texture)
}
