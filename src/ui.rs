//! User interface builder.
use crate::{
    core::{AppCore, Camera},
    stars::{PARSEC_LY, *},
};
use cgmath::{InnerSpace, Vector2, Vector3, Vector4};
use imgui::Ui;
use log::info;
use nucleo::*;
use sdl3::event::*;
use std::{f32::consts::PI, mem::transmute};
use std::ops::Deref;

pub struct AppUi {
    demo_window_opened: bool,
    positions_window_opened: bool,
    view_controls_window_opened: bool,
    nearby_stars: Vec<Star>,
    selected_star: Option<Star>,
    star_scale: f32,
    fade_labels: bool,
    pub search_window_open: bool,
    current_search: String,
}

impl AppUi {
    pub fn new() -> Self {
        info!("Init UI handler");
        AppUi {
            demo_window_opened: false,
            positions_window_opened: true,
            view_controls_window_opened: true,
            nearby_stars: Vec::default(),
            selected_star: None,
            star_scale: 1.0,
            fade_labels: false,
            search_window_open: true,
            current_search: String::new(),
        }
    }

    pub fn build_ui(&mut self, appcore: &mut AppCore) -> impl FnMut(&mut Ui) {
        |ui| {
            let global_window_size = [1920.0, 1200.0];

            let colour =[150.0, 0.0, 0.0, 255.0];


            let mut focus_keyboard_search = false;
            if ui.is_key_pressed(imgui::Key::Slash) {
                self.search_window_open = true;
                focus_keyboard_search = true;
                ui.open_popup("search");
            }

            if self.search_window_open {
                ui.modal_popup_config("search")
                    .resizable(false)
                    .movable(false)
                    .title_bar(false)
                    .always_auto_resize(true)
                    .build(|| {
                        ui.child_window("search window").size([400.0, 600.0]).build(|| {
                        if focus_keyboard_search {
                            ui.set_keyboard_focus_here();
                        }
                        let text_input = ui.input_text("search", &mut self.current_search).build();
                        if text_input && !self.current_search.is_empty() {
                            appcore.star_handler.nucleo.pattern.reparse(
                                0,
                                &self.current_search,
                                nucleo::pattern::CaseMatching::Smart,
                                nucleo::pattern::Normalization::Smart,
                                false, // might need to work something out for optimization
                            );
                        }

                        appcore.star_handler.nucleo.tick(10);


                        let matched_item_count = appcore.star_handler.nucleo.snapshot().matched_item_count(); 
                        let mut stars: Vec<Star> = Vec::new();
                        for item in appcore.star_handler.nucleo.snapshot().matched_items(0..matched_item_count) {
                            stars.push(item.data.clone());    
                        }

                        if !self.current_search.is_empty() {
                            let mut index = 0;
                            for star in stars {
                                ui.separator();
                                ui.text_colored([255.0/255.0, 0.0, 0.0, 255.0/255.0], format!("{}", star.name()));
                                if star_info_small(ui, &star, appcore) {
                                    self.selected_star = Some(star);
                                    self.search_window_open = false;
                                }
                            }
                        }
                        })
                    });
            }

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
                            ui.slider("move speed", 0.0, 0.3, &mut appcore.camera.move_speed);
                            ui.separator();

                            let deg = (appcore.camera.fov * (180.0 / PI)).floor();
                            let min = ((appcore.camera.fov * (180.0 / PI)) % 1.0) * 60.0;

                            ui.text(format!("camera fov: {:3}° {:2.0}'", deg, min));
                            ui.separator();

                            ui.text("reload distance (parsecs):");
                            ui.slider(" ", 0.0, 2.0, &mut appcore.camera.reload_distance);
                            ui.checkbox("Fade labels", &mut self.fade_labels);
                        });
                }
            });

            // star information window
            let mut star_window = true;
            if let Some(star) = &self.selected_star {
                ui.window(format!("{}", star.name()))
                    .size([400.0, 1000.0], imgui::Condition::FirstUseEver)
                    .movable(false)
                    .collapsible(false)
                    .opened(&mut star_window)
                    .position([global_window_size[0]-440.0, 40.0], imgui::Condition::FirstUseEver)
                    .build(|| {
                        star_info_small(ui, star, appcore);

                        {
                            let (hra, mra, sra) = angle_to_dms(star.ra);
                            ui.text(format!("Right Ascension: {}h {}m {:.0}s", hra, mra, sra));
                            if ui.is_item_hovered() {
                                ui.tooltip_text("Angle of the star from the prime meridian around the equator,\nat the march equinox, measured in hours, minutes and seconds of arc");
                            }

                            let (ddec, mdec, sdec) = angle_to_dms(star.dec);
                            ui.text(format!("Declination: {}° {}' {:.0}''", ddec, mdec, sdec));
                            if ui.is_item_hovered() {
                                ui.tooltip_text("Angle of the star from the equator, measured in degrees, minutes and seconds of arc");
                            }
                        }

                        ui.separator();
                        ui.text(format!("Common name: {}", star.proper.clone().unwrap_or("None".to_owned())));
                        ui.text(format!("Bayer/Flamsteed designation: {}", star.bf.clone().unwrap_or("None".to_owned())));
                        ui.text(format!("Gleise catalogue: {}", star.gl.clone().unwrap_or("None".to_owned())));
                        let hd = star.hd.clone();
                        if let Some(hd) = hd {
                            ui.text(format!("Henry Draper catalogue: HD {}", hd));
                        } else {
                            ui.text(format!("Henry Draper catalogue: None"));
                        }
                        let hr = star.hr.clone();
                        if let Some(hr) = hr {
                            ui.text(format!("Harvard Bright Star Catalogue: HR {}", hr));
                        } else {
                            ui.text(format!("Harvard Bright Star Catalogue: None"));
                        }
                        let hip = star.hip.clone();
                        if let Some(hip) = hip {
                            ui.text(format!("Hipparcos Catalogue: HIP {}", hip));
                        } else {
                            ui.text(format!("Hipparcos Catalogue: None"));
                        }
                        ui.separator();

                        ui.text(format!("Absolute Magnitude: {}", star.absmag));
                        ui.text(format!("Luminosity: {:.2} suns", star.absmag));



                        let ci = star.ci.clone();
                        if let Some(ci) = ci {
                            ui.text(format!("Colour Index: {}", ci));
                            // conversion of CI to temperature
                            // https://en.wikipedia.org/wiki/Color_index 
                            let comp1 = 1.0 / (0.92*ci + 1.7);
                            let comp2 = 1.0 / (0.92*ci + 0.62);
                            let temp = (comp1 + comp2) * 4600.0;
                            ui.text(format!("Estimated Temperature: {:.0} K", temp));
                        } else {
                            ui.text(format!("Colour Index: Unknown"));
                            ui.text(format!("Estimated Temperature: Unknown"));
                        }
                        let spec = star.spec.clone();
                        if let Some(spec) = spec {
                            ui.text(format!("Spectral Type: {}", spec));
                        } else {
                            ui.text(format!("Spectral Type: Unknown"));
                        }

                        ui.text(format!("Radial Velocity: {} km/sec", star.rv));

                        ui.separator();

                    });

                if !star_window {
                    self.selected_star = None;
                }
            }

            // imgui demo window
            if self.demo_window_opened {
                ui.show_demo_window(&mut true);
            }

            // drawing star labels
            for star in &self.nearby_stars {
                let projected = star.get_screencoord(&appcore.camera, 1920.0, 1200.0);
                if let Some(scrpos) = projected {
                    let star_radius = get_star_radius(&star, appcore.camera.pos) * 3.0;
                    let mut label_colour = 255;
                    if self.fade_labels {
                        label_colour = (20 / (star.dist(appcore.camera.pos) as i32).clamp(1, 99999))
                            .clamp(0, 255) as u8;
                    }

                    if (star.dist(appcore.camera.pos) < 3.0) {
                        let draw_text = ui.get_background_draw_list().add_text(
                            [scrpos.x + star_radius, scrpos.y + star_radius],
                            imgui::ImColor32::from_rgb(label_colour, label_colour, label_colour),
                            format!("{}", star.name()),
                        );
                    }
                }
            }

            // highlighting selected star
            if let Some(star) = &self.selected_star {
                if let Some(scrpos) = star.get_screencoord(&appcore.camera, 1920.0, 1200.0) {
                    let star_radius = get_star_radius(&star, appcore.camera.pos);
                    let draw = ui
                        .get_background_draw_list()
                        .add_circle(
                            [scrpos.x as f32, scrpos.y as f32],
                            (5.0 * star_radius)
                                .clamp(global_window_size[0] / 200.0, global_window_size[0]),
                            [1.0, 0.0, 0.0],
                        )
                        .thickness(1.0)
                        .build();
                }
            }

            // movement controls
            if appcore.camera.movable {
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

            // detecting clicks on stars
            if ui.is_mouse_clicked(imgui::MouseButton::Left) && !self.search_window_open {
                // self.selected_star = None;
                let mouse_pos = Vector2::from(ui.io().mouse_pos);
                for star in &self.nearby_stars {
                    if let Some(scrpos) = star.get_screencoord(&appcore.camera, 1920.0, 1200.0) {
                        let mouse_star_delta = scrpos.xy() - mouse_pos;
                        let mouse_distance = (mouse_star_delta.x * mouse_star_delta.x
                            + mouse_star_delta.y * mouse_star_delta.y)
                            .sqrt();

                        // todo: depth/distance priority
                        if mouse_distance
                            < (5.0 * get_star_radius(star, appcore.camera.pos))
                                .clamp(global_window_size[0] / 40.0, global_window_size[0])
                        {
                            self.selected_star = Some(star.clone());
                        }
                    }
                }
            }

            // controls
            ui.window("controls")        
            .position([20.0, (global_window_size[1] - 130.0)], imgui::Condition::FirstUseEver)
            .draw_background(false)
            .movable(false)
            .resizable(false)
            .title_bar(false)
            .build(|| {
                ui.text_colored(colour, "Move: WASD + Mouse\nShow/Recapture Cursor: ESC/ENTER\nSearch for stars: Slash (/)\n\nSelect a star by clicking on it"); 
            });
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

fn star_info_small(ui: &Ui, star: &Star, appcore: &mut AppCore) -> bool {
    if ui.button(format!("go to star##{}", star.id)) {
        appcore.camera.pos = Vector3 { x: star.x as f32, y: star.y as f32, z: (star.z + 1.0) as f32 };
        appcore.camera.orientation = Vector3::from([0.0, 0.0, 0.0]);
        return true;
    }

    ui.text(format!(
        "distance from camera: {:.2} lightyears",
        star.dist_ly(appcore.camera.pos)
    ));
    ui.text(format!(
        "distance from sol: {:.2} lightyears",
        star.dist_ly(Vector3::from([0.0, 0.0, 0.0]))
    ));

    return false;
}

fn angle_to_dms(angle: f32) -> (f32, f32, f32) {
    // calculating h:m:s from h angle
    let d = angle.floor();
    let mut m = (angle - d) * 60.0;
    let s = (m - m.floor()) * 60.0;
    m = m.floor();

    return (d, m, s);
}
