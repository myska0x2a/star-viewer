use crate::event::*;
use crate::rendering::GameRenderer;
use crate::stars::*;
use crate::util::*;
use sdl3::Sdl;
use sdl3::event::*;
use log::{info, warn};

pub struct GameCore {
    star_handler: StarHandler,
}

impl GameCore {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(GameCore {
            star_handler: StarHandler::new(),
        })
    }

    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.star_handler.load(String::from("hygdata_v41.csv"))?;
        Ok(())
    }

    pub fn handle_game_event(
        &mut self,
        event: GameEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            GameEvent::Reload => { 
                self.star_handler.load(String::from("hygdata_v41-reduced.csv"))?;
                Ok(())
            }
            _ => { Ok(()) },
        }
    }

    pub fn get_star_handler(&self) -> &StarHandler {
        return &self.star_handler
    }
}
