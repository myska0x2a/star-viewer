//! User interface builder.
use crate::{event::*, stars::PARSEC_LY};
use imgui::Ui;
use log::info;
use sdl3::event::*;

struct StarSelectionWindow {
    nya: String,
}

#[derive(Default)]
struct StarRendererStatus {
    stars_loaded: usize,
    range: f32,
}

pub struct GameUi {
    starselectionwindow: Option<StarSelectionWindow>,
    star_renderer_status: StarRendererStatus,
}

impl GameUi {
    pub fn new() -> Self {
        info!("Init UI handler");
        GameUi {
            starselectionwindow: None,
            star_renderer_status: StarRendererStatus::default(),
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
            GameEvent::StarRangeChanged(range, num) => {
                self.star_renderer_status = StarRendererStatus { stars_loaded: num, range };
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
            // let main_menu = ui.begin_main_menu_bar();
            let main_menu = ui.main_menu_bar(|| {
                ui.text(format!("Range: {:2.2} Parsecs - {:.2} ly", self.star_renderer_status.range, self.star_renderer_status.range*PARSEC_LY as f32));
                ui.separator();
                ui.text(format!("Stars Loaded: {}", self.star_renderer_status.stars_loaded));

                ui.separator();
                ui.text(format!("Vertices: {}", self.star_renderer_status.stars_loaded*6));


                // ui.menu("Ooo menu :3", || {
                //     ui.text("u found me! meow :3");
                // });
            });

            // let draw = ui
            //     .get_background_draw_list()
            //     .add_circle([700.0, 700.0], 150.0, [1.0, 0.0, 0.0])
            //     .thickness(4.0)
            //     .build();

            // ui.show_demo_window(&mut true);

        }
    }
}
