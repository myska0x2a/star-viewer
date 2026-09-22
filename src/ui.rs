//! User interface builder.
use crate::{
    core::{AppCore, Camera},
    stars::{PARSEC_LY, *},
};
use cgmath::{InnerSpace, Vector2, Vector3, Vector4};
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
    selected_star: Option<Star>,
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
            selected_star: None,
            star_scale: 1.0,
        }
    }

    pub fn build_ui(&mut self, appcore: &mut AppCore) -> impl FnMut(&mut Ui) {
        |ui| {
            // top menu bar
            let main_menu = ui.main_menu_bar(|| {
                ui.menu("Settings", || {
                    ui.checkbox("Demo Window", &mut self.demo_window_opened);
                    ui.checkbox("View Controls", &mut self.view_controls_window_opened);
                });
                ui.separator();
                ui.text(format!("Visible Stars: {}", self.nearby_stars.len()));

                ui.separator();
                ui.text(format!(
                    "Position: ({:.2} x, {:.2} y, {:.2} z)",
                    appcore.camera.pos.x, appcore.camera.pos.y, appcore.camera.pos.z,
                ));
                ui.separator();
                if let Some(star) = &self.selected_star {
                    ui.text(format!("selected star: {}", star.name()));
                } else {
                    ui.text("no star selected");
                }
                ui.separator();

                // view controls window
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

                            let deg = (appcore.camera.fov * (180.0 / PI)).floor();
                            let min = ((appcore.camera.fov * (180.0 / PI)) % 1.0) * 60.0;

                            ui.text(format!("camera fov: {:3}° {:2.0}'", deg, min));
                            ui.separator();

                            ui.text("reload distance (parsecs):");
                            ui.slider(" ", 0.0, 2.0, &mut appcore.camera.reload_distance);
                        });
                }
            });

            // imgui demo window
            if self.demo_window_opened {
                ui.show_demo_window(&mut true);
            }

            // drawing star labels
            for star in &self.nearby_stars {
                let projected = star.get_screencoord(&appcore.camera, 1920.0, 1200.0);
                if let Some(scrpos) = projected {
                    if (star.dist(appcore.camera.pos) < 3.0) {
                        let draw_text = ui.get_background_draw_list().add_text(
                            [scrpos.x + 10.0, scrpos.y + 10.0],
                            imgui::ImColor32::from_rgb(255, 255, 255),
                            format!("{}", star.name()),
                        );
                    }
                }
            }

            // highlighting selected star
            if let Some(star) = &self.selected_star {
                if let Some(scrpos) = star.get_screencoord(&appcore.camera, 1920.0, 1200.0) {
                    let draw = ui
                        .get_background_draw_list()
                        .add_circle(
                            [scrpos.x as f32, scrpos.y as f32],
                            5.0 * get_star_radius(&star, appcore.camera.pos),
                            [1.0, 0.0, 0.0],
                        )
                        .thickness(1.0)
                        .build();
                }
            }

            // movement controls
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

            // detecting clicks on stars
            if ui.is_mouse_clicked(imgui::MouseButton::Left) {
                self.selected_star = None;
                let mouse_pos = Vector2::from(ui.io().mouse_pos);
                for star in &self.nearby_stars {
                    if let Some(scrpos) = star.get_screencoord(&appcore.camera, 1920.0, 1200.0) {
                        let mouse_star_delta = scrpos.xy() - mouse_pos;
                        let mouse_distance = (mouse_star_delta.x * mouse_star_delta.x
                            + mouse_star_delta.y * mouse_star_delta.y)
                            .sqrt();

                        if mouse_distance < 10.0 {
                            self.selected_star = Some(star.clone());
                        }
                    }
                }
            }
        }
    }

    /// reloading the UI nearby star buffer
    pub fn reload_stars(&mut self, appcore: &AppCore) {
        self.nearby_stars = appcore
            .star_handler
            .get_nearby(appcore.camera.range as f64, appcore.camera.pos)
            .into_iter()
            .cloned()
            .collect();
    }
}

fn get_star_radius(star: &Star, pos: Vector3<f32>) -> f32 {
    let dist = star.dist(pos);
    let mut lightmult = star.absmag / (dist * dist);
    lightmult = lightmult.clamp(0.3, 20.0);

    return lightmult;
}
