//! App logic manager.
use crate::graphics::rendering::AppRenderer;
use crate::stars::*;
use log::{info, warn};
use sdl3::Sdl;
use sdl3::event::*;

#[derive(Copy, Clone)]
pub struct Camera {
    pub pos: [f32; 3],
    pub orientation: [f32; 3],
    pub velocity: [f32; 3],
    pub fov: f32,
}

impl Camera {
    pub fn increment_pos(&mut self, x: f32, y: f32, z: f32) {
        self.orientation[0] += x;
        self.orientation[1] += y;
        self.orientation[2] += z;
    }

    pub fn increment_vel(&mut self) {
        self.pos[0] += self.velocity[0];
        self.pos[1] += self.velocity[1];
        self.pos[2] += self.velocity[2];
    }
}

impl Default for Camera {
    fn default() -> Self {
        return Camera {
            pos: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            fov: 3.0,
        };
    }
}


pub struct AppSettings {
    pub free_move: bool,
}

pub struct AppCore {
    pub star_handler: StarHandler,
    pub settings: AppSettings,
    pub camera: Camera,
}

impl AppCore {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Init app core...");
        Ok(AppCore {
            star_handler: StarHandler::new(),
            settings: AppSettings { free_move: true },
            camera: Camera::default(),
        })
    }

    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Load app core...");
        self.star_handler
            .load(String::from("data/hygdata_v41.csv"))?;
        Ok(())
    }

    pub fn get_star_handler(&self) -> &StarHandler {
        return &self.star_handler;
    }
}
