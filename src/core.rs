//! App logic manager.
use crate::event::*;
use crate::graphics::rendering::AppRenderer;
use crate::stars::*;
use log::{info, warn};
use sdl3::Sdl;
use sdl3::event::*;

pub struct AppSettings {
    pub free_move: bool,
}

pub struct AppCore {
    pub star_handler: StarHandler,
    pub settings: AppSettings,
}

impl AppCore {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Init app core...");
        Ok(AppCore {
            star_handler: StarHandler::new(),
            settings: AppSettings { free_move: true },
        })
    }

    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Load app core...");
        self.star_handler
            .load(String::from("data/hygdata_v41.csv"))?;
        Ok(())
    }

    pub fn handle_app_event(
        &mut self,
        event: AppEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            AppEvent::Reload => {
                self.star_handler
                    .load(String::from("data/hygdata_v41-reduced.csv"))?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn get_star_handler(&self) -> &StarHandler {
        return &self.star_handler;
    }
}
