//! App logic manager.
use std::f32::consts::PI;

use crate::graphics::rendering::AppRenderer;
use crate::stars::*;
use cgmath::Basis3;
use cgmath::PerspectiveFov;
use cgmath::Rad;
use cgmath::Rotation;
use cgmath::{Deg, Matrix3, Rotation3, Vector3, Matrix4};
use log::{info, warn};
use sdl3::Sdl;
use sdl3::event::*;

#[derive(Copy, Clone)]
pub struct Camera {
    // pub pos: [f32; 3],
    pub pos: Vector3<f32>,
    pub orientation: Vector3<f32>,
    pub sensitivity: f32,
    pub fov: f32,
    pub range: f64,
}

impl Camera {
    pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
        if ((self.orientation.y > -PI) && (self.orientation.y < PI))
            || (self.orientation.y * y < 0.0)
        {
            self.orientation.y += y * self.sensitivity;
        }

        self.orientation.z += z * self.sensitivity;

    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        let rotx = -self.orientation.y;
        let roty = -self.orientation.z;
        let rotz = -self.orientation.x;

        let rotmatx = Basis3::<f32>::from_angle_x(Rad(rotx));
        let rotmaty = Basis3::<f32>::from_angle_y(Rad(roty));
        let rotmatz = Basis3::<f32>::from_angle_z(Rad(rotz));

        let mut translation_unit = Vector3 { x, y, z };

        translation_unit = rotmatx.rotate_vector(translation_unit);
        translation_unit = rotmaty.rotate_vector(translation_unit);
        translation_unit = rotmatz.rotate_vector(translation_unit);

        self.pos += translation_unit;
    }

    pub fn get_projection_matrix(&self, w: f32, h: f32) -> PerspectiveFov<f32> {
        let projection_matrix = PerspectiveFov {
            fovy: Rad(self.fov),
            aspect: w / h,
            near: 0.01,
            far: 1.1,
        };

        return projection_matrix;
    }
}

impl Default for Camera {
    fn default() -> Self {
        return Camera {
            pos: Vector3::new(0.0, 0.0, 0.0),
            orientation: Vector3::new(0.0, 0.0, 0.0),
            sensitivity: 1.0,
            fov: 3.1,
            range: 2.0,
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
