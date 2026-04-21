//! User interface builder.
use crate::{core::GameCore, event::*, stars::PARSEC_LY};
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
    demo_window_opened: bool,
}

impl GameUi {
    pub fn new() -> Self {
        info!("Init UI handler");
        GameUi {
            starselectionwindow: None,
            star_renderer_status: StarRendererStatus::default(),
            demo_window_opened: false,
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

    pub fn render_ui(&mut self, ev: &EventSender, game: &mut GameCore) -> impl FnMut(&mut Ui) {
        |ui| {
            let main_menu = ui.main_menu_bar(|| {
                ui.menu("Settings", || {;
                    // ui.text(format!("Free move: {}", game.settings.free_move));
                    ui.checkbox("Free Move", &mut game.settings.free_move);
                    ui.checkbox("Demo Window", &mut self.demo_window_opened);
                });
                
                ui.separator();
                ui.text(format!("Range: {:2.2} Parsecs - {:.2} ly", self.star_renderer_status.range, self.star_renderer_status.range*PARSEC_LY as f32));
                ui.separator();
                ui.text(format!("Stars Loaded: {}", self.star_renderer_status.stars_loaded));

                ui.separator();
                ui.text(format!("Vertices: {}", self.star_renderer_status.stars_loaded*6));

                // let text = "haiiiii";
                // let avail = ui.content_region_max()[0] - ui.calc_text_size(text)[0];
                // ui.set_cursor_pos([avail, 0.0]);
                // ui.text(text);

                // ui.menu("Ooo menu :3", || {
                //     ui.text("u found me! meow :3");
                // });
            });

            // let draw = ui
            //     .get_background_draw_list()
            //     .add_circle([700.0, 700.0], 150.0, [1.0, 0.0, 0.0])
            //     .thickness(4.0)
            //     .build();
            
            if self.demo_window_opened {
                ui.show_demo_window(&mut true);
            }
        }
    }
}

pub fn text_align_right<T: AsRef<str>>(ui: &Ui, text: T) {
    let avail = ui.content_region_max()[0] - ui.calc_text_size(&text)[1];
    ui.set_cursor_pos([avail, 0.0]);
    ui.text(text);
}
