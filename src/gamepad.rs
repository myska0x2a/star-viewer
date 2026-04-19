//! WIP gamepad support.
#![allow(unused)]

use crate::event::*;
use log::{Level, debug, error, info, log_enabled, trace};
use sdl3::event::*;
use sdl3::gamepad::Gamepad;
use sdl3::{GamepadSubsystem, Sdl};

pub struct ControllerHandler {
    subsystem: GamepadSubsystem,
    controller: Option<Gamepad>,
}

impl ControllerHandler {
    pub fn init(sdl: &Sdl) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Init controller handler");

        let gamepad_subsystem = sdl.gamepad()?;
        info!("Controller connected: {}", gamepad_subsystem.has_gamepad());

        let gamepads = gamepad_subsystem.gamepads()?;
        info!("Controllers: {}", gamepads.len());

        let mut gamepad: Option<Gamepad> = None;

        for gamepad_id in gamepads {
            gamepad = Some(gamepad_subsystem.get(gamepad_id)?);
        }

        gamepad_subsystem.set_events_processing_state(true);

        return Ok(ControllerHandler {
            subsystem: gamepad_subsystem,
            controller: gamepad,
        });
    }

    pub fn handle_event(&mut self, event: GameEvent) -> Result<(), Box<dyn std::error::Error>> {
        if let GameEvent::StarSelected(text) = event.clone()
            && let Some(gamepad) = &mut self.controller
        {
            info!("Star selected");
            gamepad.set_rumble(0, 6500, 100)?;
        }

        Ok(())
    }
}
