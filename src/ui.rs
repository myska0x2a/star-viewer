//! User interface builder.
use crate::{
    core::{AppCore, Camera},
    stars::{PARSEC_LY, *},
};
use imgui::Ui;
use log::info;
use sdl3::event::*;
use std::f32::consts::PI;

#[derive(Default)]
struct StarRendererStatus {
    stars_loaded: usize,
    range: f32,
}

pub struct AppUi {
    star_renderer_status: StarRendererStatus,
    demo_window_opened: bool,
    free_move: bool,
}

impl AppUi {
    pub fn new() -> Self {
        info!("Init UI handler");
        AppUi {
            star_renderer_status: StarRendererStatus::default(),
            demo_window_opened: false,
            free_move: true,
        }
    }

    pub fn build_ui(&mut self, appcore: &mut AppCore) -> impl FnMut(&mut Ui) {
        |ui| {
            let main_menu = ui.main_menu_bar(|| {
                ui.menu("Settings", || {
                    ui.checkbox("Free Move", &mut self.free_move);
                    ui.checkbox("Demo Window", &mut self.demo_window_opened);
                });
                ui.separator();
                ui.text(format!(
                    "Range: {:2.2} Parsecs - {:.2} ly",
                    self.star_renderer_status.range,
                    self.star_renderer_status.range * PARSEC_LY as f32
                ));
                ui.separator();
                ui.text(format!(
                    "Stars Loaded: {}",
                    self.star_renderer_status.stars_loaded
                ));

                ui.separator();
                ui.text(format!(
                    "Vertices: {}",
                    self.star_renderer_status.stars_loaded * 6
                ));

                // ui.slider("camera zoom", 0.1, 3.10, &mut appcore.camera.fov);
                ui.separator();
                ui.text(format!(
                    "camera orientation : ({:.2}°x, {:.2}°y, {:.2}°z)",
                    appcore.camera.orientation.x * (180.0 / PI),
                    appcore.camera.orientation.y * (180.0 / PI),
                    appcore.camera.orientation.z * (180.0 / PI)
                ));
                ui.separator();
                ui.text(format!(
                    "camera xyz: ({:.2}, {:.2}, {:.2})",
                    appcore.camera.pos.x, appcore.camera.pos.y, appcore.camera.pos.z,
                ));
                ui.separator();
                ui.text(format!("camera zoom: {:.2}", appcore.camera.fov));

                let window = ui.window("miau");
            });

            // let draw = ui
            //     .get_background_draw_list()
            //     .add_circle([700.0, 700.0], 150.0, [1.0, 0.0, 0.0])
            //     .thickness(4.0)
            //     .build();

            if self.demo_window_opened {
                ui.show_demo_window(&mut true);
            }

            let speed_boost = 1.0;
            let base_speed = 0.073;

            if ui.is_key_down(imgui::Key::W) {
                appcore
                    .camera
                    .translate(0.0, 0.0, -base_speed * speed_boost);
            }
            if ui.is_key_down(imgui::Key::S) {
                appcore.camera.translate(0.0, 0.0, base_speed * speed_boost);
            }
            if ui.is_key_down(imgui::Key::A) {
                appcore
                    .camera
                    .translate(-base_speed * speed_boost, 0.0, 0.0);
            }
            if ui.is_key_down(imgui::Key::D) {
                appcore.camera.translate(base_speed * speed_boost, 0.0, 0.0);
            }
        }
    }
}

pub fn text_align_right<T: AsRef<str>>(ui: &Ui, text: T) {
    let avail = ui.content_region_max()[0] - ui.calc_text_size(&text)[1];
    ui.set_cursor_pos([avail, 0.0]);
    ui.text(text);
}
