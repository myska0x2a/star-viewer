//! Loading and handling of stars.
use crate::core::Camera;
use cgmath::{Basis3, Matrix4, Rad, Rotation, Rotation3, Transform, Vector3, Vector4};
use kiddo::float::{distance::SquaredEuclidean, kdtree::KdTree};
use log::{debug, info, trace, warn};
use serde::Deserialize;
use std::io;

pub const PARSEC_LY: f64 = 3.262;

#[derive(Debug, Deserialize, Clone)]
pub struct Star {
    id: u32,
    hip: Option<i32>,
    hd: Option<i32>,
    hr: Option<i32>,
    gl: Option<String>,
    bf: Option<String>,
    proper: Option<String>,
    dist: f64,
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
    pub fn dist(&self) -> f64 {
        return self.dist.clone();
    }

    /// returns the distance in light-years.
    pub fn dist_ly(&self) -> f64 {
        return self.dist.clone() * PARSEC_LY;
    }

    pub fn screencord(&self, camera: &Camera) -> (f32, f32) {
        todo!();
    }
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
    // todo: add reference point (radius centre)
    pub fn get_nearby(&self, radius: f64) -> Vec<&Star> {
        debug!("StarHandler retreiving nearby stars");
        let mut nearby = Vec::new();
        let within = self
            .tree
            .within::<SquaredEuclidean>(&[0f64, 0f64, 0f64], radius.powf(2.0));

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

    pub fn get_nearby_screencoord(
        &self,
        camera: &Camera,
        range: f64,
        detection_radius: f32,
        w: f32,
        h: f32,
    ) -> Vec<(String, Vector4<f32>)> {
        let nearby = self.get_nearby(range);

        let projection_matrix: Matrix4<f32> = Matrix4::from(camera.get_projection_matrix(w, h));


        let rotx = -camera.orientation.y;
        let roty = -camera.orientation.z;
        let rotz = -camera.orientation.x;

        let rotmatx = Basis3::<f32>::from_angle_x(Rad(rotx));
        let rotmaty = Basis3::<f32>::from_angle_y(Rad(roty));
        let rotmatz = Basis3::<f32>::from_angle_z(Rad(rotz));

        let mut positions: Vec<(String, Vector4<f32>)> = Vec::new();

        for star in nearby {
            let mut pos = Vector3::from((star.x as f32, star.y as f32, star.z as f32)) - camera.pos;
            // pos = rotation_matrix.rotate_vector(pos);

            pos = rotmatx.rotate_vector(pos);
            pos = rotmaty.rotate_vector(pos);
            pos = rotmatz.rotate_vector(pos);

            let mut starposndc = projection_matrix * Vector4::from((pos.x, pos.y, pos.z, 1.0));
            // starposndc.x *= w;
            // starposndc.y *= h;

            println!("Star: {}, Screen Pos: {:?}", star.name(), starposndc);

            positions.push((star.name(), starposndc));
        }

        return positions;
    }

    pub fn get_stars(&self) -> &Vec<Star> {
        return &self.stars;
    }

    pub fn get_tree(&self) -> &KdTree<f64, u32, 3, 32, u32> {
        return &self.tree;
    }
}
