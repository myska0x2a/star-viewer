//! Game logic manager.
use crate::event::*;
use crate::fleet::*;
use crate::graphics::rendering::GameRenderer;
use crate::stars::*;
use crate::util::*;
use log::{info, warn};
use sdl3::Sdl;
use sdl3::event::*;

pub struct GameSettings {
    pub free_move: bool,
}

pub struct GameCore {
    pub star_handler: StarHandler,
    pub fleet_handler: FleetHandler,
    pub settings: GameSettings,
}

impl GameCore {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Init game core...");
        Ok(GameCore {
            star_handler: StarHandler::new(),
            fleet_handler: FleetHandler::new(),
            settings: GameSettings { free_move: true },
        })
    }

    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Load game core...");
        self.star_handler
            .load(String::from("data/hygdata_v41.csv"))?;
        Ok(())
    }

    pub fn handle_game_event(
        &mut self,
        event: GameEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            GameEvent::Reload => {
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
