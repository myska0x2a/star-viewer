use crate::event::*;
use imgui::Ui;
use log::info;
use sdl3::event::*;

struct StarSelectionWindow {
    nya: String,
}

pub struct GameUi {
    starselectionwindow: Option<StarSelectionWindow>,
    position: f32,
}

impl GameUi {
    pub fn new() -> Self {
        info!("Init UI handler");
        GameUi {
            starselectionwindow: None,
            position: 0.0,
        }
    }

    pub fn handle_game_event(
        &mut self,
        event: GameEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            GameEvent::StarSelected(text) => {
                self.open_star_selection_window(text);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn open_star_selection_window(&mut self, text: String) {
        let starselectionwindow = StarSelectionWindow { nya: text };
        self.starselectionwindow = Some(starselectionwindow);
    }

    pub fn close_star_selection_window(&mut self) {
        self.starselectionwindow = None;
    }

    pub fn render_ui(&mut self, ev: &EventSender) -> impl FnMut(&mut Ui) {
        |ui| {
            // ui.show_demo_window(&mut true);

            // let open_alternate = ui.button("Open alternate star selection window");

            // let close = ui.button("Close star selection window");

            // let open = ui.button("Open star selection window");

            // if open {
            //     ev.push_custom_event(GameEvent::StarSelected(String::from("nya :3")));
            // }

            // if open_alternate {
            //     ev.push_custom_event(GameEvent::StarSelected(String::from("Miau :3")));
            // }

            // if close {
            //     &self.close_star_selection_window();
            // }

            // if let Some(window) = &self.starselectionwindow {
            //     let wt = ui
            //         .window("Star selection window")
            //         .size([210.0, 300.0], imgui::Condition::FirstUseEver)
            //         .position([200.0, 200.0], imgui::Condition::FirstUseEver)
            //         .build(|| {
            //             ui.text(format!("{}", window.nya));
            //         });
            // }

            ui.set_mouse_cursor(None);

            let zoomwindow = ui
                .window("Viewer controls")
                .size([230.0, 150.0], imgui::Condition::FirstUseEver)
                .position([100.0, 100.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    if ui.slider("Position", -20.0, 20.0, &mut self.position) {
                        ev.push_custom_event(GameEvent::PositionChanged(self.position.clone()));
                    }
                });
        }
    }
}
