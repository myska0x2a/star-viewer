//! User interface builder.
use crate::{
    core::{AppCore, Camera},
    stars::{PARSEC_LY, *},
};
use cgmath::{Vector4, Vector3, Vector2, InnerSpace};
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
    view_controls_window_opened: bool,
    nearby_stars: Vec<Star>,
    star_scale: f32,
}

impl AppUi {
    pub fn new() -> Self {
        info!("Init UI handler");
        AppUi {
            star_renderer_status: StarRendererStatus::default(),
            demo_window_opened: false,
            positions_window_opened: true,
            view_controls_window_opened: true,
            nearby_stars: Vec::default(),
            star_scale: 1.0,
        }
    }

    pub fn build_ui(&mut self, appcore: &mut AppCore) -> impl FnMut(&mut Ui) {
        |ui| {
            let main_menu = ui.main_menu_bar(|| {
                ui.menu("Settings", || {
                    ui.checkbox("Demo Window", &mut self.demo_window_opened);
                    ui.checkbox("View Controls", &mut self.view_controls_window_opened);
                });
                ui.separator();
                ui.text(format!(
                    "Visible Stars: {}",
                    self.nearby_stars.len()
                ));

                ui.separator();
                ui.text(format!(
                    "Position: ({:.2} x, {:.2} y, {:.2} z)",
                    appcore.camera.pos.x, appcore.camera.pos.y, appcore.camera.pos.z,
                ));
                ui.separator();

                if self.view_controls_window_opened {
                    ui.window("view controls")
                        .size([400.0, 400.0], imgui::Condition::FirstUseEver)
                        .position([20.0, 40.0], imgui::Condition::FirstUseEver)
                        .build(|| {
                            ui.text(format!(
                                "camera orientation : ({:.2}° x, {:.2}° y, {:.2}° z)",
                                appcore.camera.orientation.x as f32 * (180.0 / PI),
                                appcore.camera.orientation.y as f32 * (180.0 / PI),
                                appcore.camera.orientation.z as f32 * (180.0 / PI)
                            ));
                            ui.separator();
                            
                            let deg = (appcore.camera.fov * (180.0/PI)).floor();
                            let min = ((appcore.camera.fov * (180.0/PI)) % 1.0) * 60.0;

                            ui.text(format!("camera fov: {:3}° {:2.0}'", deg, min));
                            ui.separator();

                            ui.text("reload distance (parsecs):");
                            ui.slider(
                                " ",
                                0.0,
                                2.0,
                                &mut appcore.camera.reload_distance,
                            );
                        });
                }
            });

            for star in &self.nearby_stars {
                let projected = star.get_screencoord(&appcore.camera, 1920.0, 1200.0);
                if let Some(scrpos) = projected {
                    // let draw = ui
                    //     .get_background_draw_list()
                    //     .add_circle([scrpos.x as f32, scrpos.y as f32], 5.0, [1.0, 0.0, 0.0])
                    //     .thickness(1.0)
                    //     .build();

                    if (star.dist(appcore.camera.pos) < 3.0) {
                        let draw_text = ui
                            .get_background_draw_list()
                            .add_text([scrpos.x+10.0, scrpos.y+10.0], imgui::ImColor32::from_rgb(255, 255, 255), format!("{}", star.name()));
                    }
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


            if ui.is_mouse_down(imgui::MouseButton::Left) {
                let intersected_stars: Vec<Star> = Vec::new();
                let mouse_pos = Vector2::from(ui.io().mouse_pos);
                ui.text(format!("mouse pos: {:?}", mouse_pos));
                for star in &self.nearby_stars {
                    if let Some(scrpos) = star.get_screencoord(&appcore.camera, 1920.0, 1200.0) {
                        let mouse_star_delta = scrpos.xy() - mouse_pos;
                        let mouse_distance = (mouse_star_delta.x*mouse_star_delta.x + mouse_star_delta.y*mouse_star_delta.y).sqrt();
                        ui.text(format!("distance from {}: {}", star.name(), mouse_distance));

                    }
                }
            }
        }
    }

    pub fn reload_stars(&mut self, appcore: &AppCore) {
        self.nearby_stars = appcore.star_handler.get_nearby(
            appcore.camera.range as f64,
            appcore.camera.pos,
        ).into_iter().cloned().collect();

    }
}

pub fn text_align_right<T: AsRef<str>>(ui: &Ui, text: T) {
    let avail = ui.content_region_max()[0] - ui.calc_text_size(&text)[1];
    ui.set_cursor_pos([avail, 0.0]);
    ui.text(text);
}
