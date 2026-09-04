//! App logic manager.
use crate::graphics::rendering::AppRenderer;
use crate::stars::*;
use log::{info, warn};
use sdl3::Sdl;
use sdl3::event::*;
use cgmath::{ Vector3, Matrix3 };

#[derive(Copy, Clone)]
pub struct Camera {
    // pub pos: [f32; 3],
    pub pos: Vector3<f32>,
    pub orientation: Vector3<f32>,
    pub sensitivity: f32,
    pub fov: f32,
}

impl Camera {
    pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.orientation[0] += x*self.sensitivity;
        self.orientation[1] += y*self.sensitivity;
        self.orientation[2] += z*self.sensitivity;

    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        // unitx = self.orientation[0];


        self.pos[0] += x;
        self.pos[1] += y;
        self.pos[2] += z;
    }
}

impl Default for Camera {
    fn default() -> Self {
        return Camera {
            pos: Vector3::new(0.0, 0.0, 0.0),
            orientation: Vector3::new(0.0, 0.0, 0.0),
            sensitivity: 1.0,
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
