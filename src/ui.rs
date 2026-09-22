//! User interface builder.
use crate::{
    core::{AppCore, Camera},
    stars::{PARSEC_LY, *},
};
use cgmath::{Vector4, Vector3};
use imgui::Ui;
use log::info;
use sdl3::event::*;
use std::{f32::consts::PI, mem::transmute};

#[derive(Default)]
struct StarRendererStatus {
    stars_loaded: usize,
    range: f32,
}

pub struct AppUi {
    star_renderer_status: StarRendererStatus,
    demo_window_opened: bool,
    positions_window_opened: bool,
    camera_controls_window_opened: bool,
    nearby_stars: Vec<Star>,
    star_scale: f32,
    free_move: bool,
}

impl AppUi {
    pub fn new() -> Self {
        info!("Init UI handler");
        AppUi {
            star_renderer_status: StarRendererStatus::default(),
            demo_window_opened: false,
            positions_window_opened: true,
            camera_controls_window_opened: true,
            nearby_stars: Vec::default(),
            star_scale: 1.0,
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
                    appcore.camera.orientation.x as f32 * (180.0 / PI),
                    appcore.camera.orientation.y as f32 * (180.0 / PI),
                    appcore.camera.orientation.z as f32 * (180.0 / PI)
                ));
                ui.separator();
                ui.text(format!(
                    "camera xyz: ({:.2}, {:.2}, {:.2})",
                    appcore.camera.pos.x, appcore.camera.pos.y, appcore.camera.pos.z,
                ));
                ui.separator();
                ui.text(format!("camera zoom: {:.2}", appcore.camera.fov));
                ui.separator();

                let mut nearby_window = false;

                ui.checkbox("detect star positions", &mut self.positions_window_opened);
                ui.checkbox("camera controls", &mut self.camera_controls_window_opened);

                if self.camera_controls_window_opened {
                    ui.window("camera controls")
                        .size([400.0, 400.0], imgui::Condition::FirstUseEver)
                        .position([20.0, 500.0], imgui::Condition::FirstUseEver)
                        .build(|| {
                            ui.slider(
                                "reload distance",
                                0.0,
                                50.0,
                                &mut appcore.camera.reload_distance,
                            );
                        });
                }

                if self.positions_window_opened {
                    ui.window("nearby window")
                        .size([400.0, 400.0], imgui::Condition::FirstUseEver)
                        .position([20.0, 40.0], imgui::Condition::FirstUseEver)
                        .build(|| {
                            ui.text("window! meow");
                            ui.text(format!("range: {}", appcore.camera.range));
                            if (ui.button("get nearby")) {
                                let window_size = ui.window_size();
                                self.nearby_stars = appcore.star_handler.get_nearby(
                                    appcore.camera.range as f64,
                                    appcore.camera.pos,
                                ).into_iter().cloned().collect();
                            }

                            ui.slider("star scale", 0.0, 10.0, &mut self.star_scale);

                            ui.text(format!(
                                "screen dimensions: {} w {} h",
                                ui.window_size()[0],
                                ui.window_size()[1]
                            ));


                        });
                }

                let window = ui.window("miau");
            });

            for star in &self.nearby_stars {
                let projected = star.get_screencoord(&appcore.camera, 1920.0, 1200.0);
                if let Some(scrpos) = projected {
                    ui.text(format!(
                        "{}: {:.2} x {:.2} y {:.2} z",
                        star.name(), scrpos.x, scrpos.y, scrpos.z
                    ));

                    // let draw = ui
                    //     .get_background_draw_list()
                    //     .add_circle([scrpos.x as f32, scrpos.y as f32], 5.0, [1.0, 0.0, 0.0])
                    //     .thickness(1.0)
                    //     .build();

                    let draw_text = ui
                        .get_background_draw_list()
                        .add_text([scrpos.x+10.0, scrpos.y+10.0], imgui::ImColor32::from_rgb(255, 255, 255), format!("{}", star.name()));
                }
            }


            if self.demo_window_opened {
                ui.show_demo_window(&mut true);
            }

            let base_speed = 0.073;

            if ui.is_key_down(imgui::Key::W) {
                appcore.camera.translate(0.0, 0.0, -1.0);
            }
            if ui.is_key_down(imgui::Key::S) {
                appcore.camera.translate(0.0, 0.0, 1.0);
            }
            if ui.is_key_down(imgui::Key::A) {
                appcore.camera.translate(-1.0, 0.0, 0.0);
            }
            if ui.is_key_down(imgui::Key::D) {
                appcore.camera.translate(1.0, 0.0, 0.0);
            }
        }
    }
}

pub fn text_align_right<T: AsRef<str>>(ui: &Ui, text: T) {
    let avail = ui.content_region_max()[0] - ui.calc_text_size(&text)[1];
    ui.set_cursor_pos([avail, 0.0]);
    ui.text(text);
}
