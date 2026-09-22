//! Loading and handling of stars.
use crate::core::Camera;
use cgmath::{
    Basis3, InnerSpace, Matrix3, Matrix4, Rad, Rotation, Rotation3, Transform, Vector3, Vector4,
};
use kiddo::float::{distance::SquaredEuclidean, kdtree::KdTree};
use log::{debug, info, trace, warn};
use serde::Deserialize;
use std::{io, ops::Mul};

pub const PARSEC_LY: f64 = 3.262;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Star {
    pub id: u32,
    pub hip: Option<i32>,
    pub hd: Option<i32>,
    pub hr: Option<i32>,
    pub gl: Option<String>,
    pub bf: Option<String>,
    pub proper: Option<String>,
    pub dist: f64,
    pub absmag: f32,
    pub ci: Option<f32>,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Star {
    /// run through labels to find the most human name.
    pub fn name(&self) -> String {
        // common name
        if let Some(proper) = &self.proper {
            return proper.clone();
        }
        // bayer/flamsteed
        if let Some(bf) = &self.bf {
            return bf.clone();
        }
        // gleise
        if let Some(gl) = &self.gl {
            return gl.clone();
        }
        // henry draper
        if let Some(hd) = &self.hd {
            return String::from(format!("HD {}", hd));
        }
        // harvard revised
        if let Some(hr) = &self.hr {
            return String::from(format!("HR {}", hr));
        }

        return String::from("Unnamed star");
    }

    /// returns the distance in parsecs.
    pub fn dist(&self, pos: Vector3<f32>) -> f32 {
        let delta = Vector3::from([self.x as f32, self.y as f32, self.z as f32]) - pos;
        return delta.magnitude();
    }

    /// returns the distance in light-years.
    pub fn dist_ly(&self, pos: Vector3<f32>) -> f32 {
        return self.dist(pos) * PARSEC_LY as f32;
    }

    // retreives the screen coordinate of the star through projection
    pub fn get_screencoord(&self, camera: &Camera, w: f32, h: f32) -> Option<Vector3<f32>> {
        let projection_matrix: Matrix4<f32> =
            Matrix4::from(camera.get_projection_matrix(1920.0, 1200.0));

        let rotmatx = Basis3::<f32>::from_angle_x(Rad(camera.orientation.y));
        let rotmaty = Basis3::<f32>::from_angle_y(Rad(camera.orientation.z));
        let rotmatz = Basis3::<f32>::from_angle_z(Rad(camera.orientation.x));

        // camera space transform
        let mut starpos = Vector3::from((self.x as f32, self.y as f32, self.z as f32)) - camera.pos;
        starpos = rotmatz.rotate_vector(starpos);
        starpos = rotmaty.rotate_vector(starpos);
        starpos = rotmatx.rotate_vector(starpos);

        // projection
        let projection_matrix = camera.get_projection_matrix(1920.0, 1200.0);
        let starposclip = Matrix4::from(projection_matrix)
            * Vector4::from((starpos.x, starpos.y, starpos.z, 1.0));

        // conversion from clip space to screen space
        let screen_position = clip_to_viewport(starposclip, 1920.0, 1200.0);

        return screen_position;
    }
}

// useful: https://www.songho.ca/opengl/gl_viewport.html
fn clip_to_viewport(clip: Vector4<f32>, w: f32, h: f32) -> Option<Vector3<f32>> {
    // conversion to actual ndc coordinates
    // https://stackoverflow.com/questions/22886853/how-to-convert-projected-points-to-screen-coordinatesviewport-matrix

    if ((clip.x > -clip.w) && (clip.x < clip.w))
        || ((clip.y > -clip.w) && (clip.y < clip.w))
        || ((clip.z > -clip.w) && (clip.z < clip.w))
    {
        let ndc = clip.xyz() / clip.w;

        // far/near clip values
        let n = 0.01;
        let f = 1.1;

        // https://stackoverflow.com/questions/57938025/how-does-a-camera-convert-from-clip-space-into-screen-space
        let screenx = (ndc.x + 1.0) * (w / 2.0);
        let screeny = 1200.0 - ((ndc.y + 1.0) * (h / 2.0));
        // let screenz = (ndc.z + 1.0) * (f - n) / 2.0 + n;
        let screenz = (((f - n) / 2.0) * (ndc.z)) + ((f + n) / 2.0);

        return Some(Vector3::from((screenx, screeny, screenz)));
    }

    return None;
}

#[derive(Clone)]
pub struct StarHandler {
    stars: Vec<Star>,
    tree: KdTree<f64, u32, 3, 32, u32>,
    range: f32,
}

impl StarHandler {
    pub fn new() -> StarHandler {
        info!("Init star handler");
        return StarHandler {
            stars: Vec::new(),
            tree: KdTree::new(),
            range: 3.0,
        };
    }

    // load the star data into the handler
    pub fn load(&mut self, path: String) -> Result<(), Box<dyn std::error::Error>> {
        info!("Loading stars from {}", path);
        let mut stars: Vec<Star> = Vec::new();
        let mut tree = KdTree::new();
        let mut id: u32 = 0;
        let mut rdr = csv::Reader::from_path(path)?;

        for result in rdr.deserialize() {
            let star: Star = result?;
            tree.add(&[star.x, star.y, star.z], id);
            stars.push(star);
            id = id + 1;
        }

        self.stars = stars;
        self.tree = tree;

        Ok(())
    }

    // return a list of references to all stars within a specified radius.
    pub fn get_nearby(&self, radius: f64, pos: Vector3<f32>) -> Vec<&Star> {
        debug!("StarHandler retreiving nearby stars");
        let mut nearby = Vec::new();
        let within = self.tree.within::<SquaredEuclidean>(
            &[pos.x as f64, pos.y as f64, pos.z as f64],
            radius.powf(2.0),
        );

        for neighbor in within {
            nearby.push(self.stars.get(neighbor.item as usize).unwrap());
        }

        info!(
            "{} stars retreived from {} parsec radius",
            nearby.len(),
            radius
        );

        return nearby;
    }

    pub fn get_stars(&self) -> &Vec<Star> {
        return &self.stars;
    }

    pub fn get_tree(&self) -> &KdTree<f64, u32, 3, 32, u32> {
        return &self.tree;
    }
}
