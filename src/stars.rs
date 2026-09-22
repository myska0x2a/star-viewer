//! Loading and handling of stars.
use crate::core::Camera;
use cgmath::{Basis3, Matrix3, Matrix4, Rad, Rotation, Rotation3, Transform, Vector3, Vector4};
use kiddo::float::{distance::SquaredEuclidean, kdtree::KdTree};
use log::{debug, info, trace, warn};
use serde::Deserialize;
use std::{io, ops::Mul};

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

    pub fn get_nearby_screencoord(
        &self,
        camera: &Camera,
        range: f32,
        detection_radius: f32,
        w: f32,
        h: f32,
    ) -> Vec<(String, Vector3<f32>)> {
        println!("search");

        let projection_matrix: Matrix4<f32> = Matrix4::from(camera.get_projection_matrix(1920.0, 1200.0));

        // println!("nearby projection matrix: {:?}", projection_matrix);

        let mut positions: Vec<(String, Vector3<f32>)> = Vec::new();
        let nearby = self.get_nearby(range as f64, camera.pos);


        let rotx = camera.orientation.y;
        let roty = camera.orientation.z;
        let rotz = camera.orientation.x;

        let rotmatx = Basis3::<f32>::from_angle_x(Rad(rotx));
        let rotmaty = Basis3::<f32>::from_angle_y(Rad(roty));
        let rotmatz = Basis3::<f32>::from_angle_z(Rad(rotz));



        for star in nearby {
            let mut starpos = Vector3::from((star.x as f32, star.y as f32, star.z as f32)) - camera.pos;

            starpos = rotmatx.rotate_vector(starpos);
            starpos = rotmaty.rotate_vector(starpos);
            starpos = rotmatz.rotate_vector(starpos);

            // let rotation_matrix = rot3d(camera.orientation.x, camera.orientation.y, camera.orientation.z);
            // starpos = rotation_matrix * starpos;

            let projection_matrix = camera.get_projection_matrix(1920.0, 1200.0);
            let starposclip = Matrix4::from(projection_matrix) * Vector4::from((starpos.x, starpos.y, starpos.z, 1.0));

            let screen_position = clip_to_viewport(starposclip, 1920.0, 1200.0);
            
            let starndc = starposclip.xyz() / starposclip.w;

            let dist = (star.x*star.x + star.y*star.y + star.z*star.z).sqrt() + 0.1;

            // println!("{} NDC : {}, {}, {}", star.name(), starndc.x, starndc.y, starndc.z);
            // println!("Screen position: {}, {}, {}\n", screen_position.x, screen_position.y, dist);

            positions.push((star.name(), Vector3::from([screen_position.x, screen_position.y, dist as f32/(star.absmag*star.absmag)])));
        }

        println!("search done");

        return positions;
    }

    pub fn get_stars(&self) -> &Vec<Star> {
        return &self.stars;
    }

    pub fn get_tree(&self) -> &KdTree<f64, u32, 3, 32, u32> {
        return &self.tree;
    }

}

// https://www.songho.ca/opengl/gl_viewport.html
fn clip_to_viewport(clip: Vector4<f32>, w: f32, h: f32) -> Vector3<f32> {
    // conversion to actual ndc coordinates
    // https://stackoverflow.com/questions/22886853/how-to-convert-projected-points-to-screen-coordinatesviewport-matrix 
    let ndc = clip.xyz() / clip.w;

    // no ndc conversion
    // let ndc = clip.xyz();

    // far/near clip values
    let n = 0.01;
    let f = 1.1;
    
    // bottom/left corners of viewport
    let x = 0.0;
    let y = 0.0;

    // let viewport_transform = Matrix4::from_cols(
    //         Vector4 {
    //             x: w/2.0,
    //             y: 0.0,
    //             z: 0.0,
    //             w: 0.0,
    //         },
    //         Vector4 {
    //             x: 0.0,
    //             y: h/2.0,
    //             z: 0.0,
    //             w: 0.0,
    //         },
    //         Vector4 {
    //             x: 0.0,
    //             y: 0.0,
    //             z: (f-n)/2.0,
    //             w: 0.0,
    //         },
    //         Vector4 {
    //             x: x+(w/2.0),
    //             y: y+(h/2.0),
    //             z: (f+n)/2.0,
    //             w: 1.0,
    //         },
    //     );

    // let screen_coord = viewport_transform * Vector4::from((ndc.x, ndc.y, ndc.z, 1.0)); 

    // let screenx = (ndc.x + 1.0) * (w/2.0);
    // let screeny = 1200.0 - ((ndc.y + 1.0) * (h/2.0));
    
    // let screenx = ((ndc.x) + 0.5) * (w/2.0);
    // let screeny = 1200.0 - ((ndc.y) + 0.5) * (h/2.0);

    // let screenx = ((w / 2.0) * ndc.x) + (x + (w / 2.0));
    // let screeny = ((h / 2.0) * ndc.y) + (y + (h / 2.0));
    
    // https://stackoverflow.com/questions/57938025/how-does-a-camera-convert-from-clip-space-into-screen-space
    let screenx = (ndc.x + 1.0) * 0.5 * (w - 1.0);
    let screeny = (1.0 - (ndc.y + 1.0) * 0.5) * (h - 1.0);
    let screenz = (ndc.z + 1.0) * (f - n) / 2.0 + n;


    let screen_coord = Vector3::from((screenx, screeny, screenz));

    return screen_coord.xyz();
}




fn rot3d(rx: f32, ry: f32, rz: f32) -> Matrix3<f32> {
    let matx = Matrix3::from_cols(
        Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        Vector3 {
            x: 0.0,
            y: rx.cos(),
            z: rx.sin(),
        },
        Vector3 {
            x: 0.0,
            y: -rx.sin(),
            z: rx.cos(),
        },
    );

    let maty = Matrix3::from_cols(
        Vector3 {
            x: ry.cos(),
            y: 0.0,
            z: -ry.sin(),
        },
        Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        Vector3 {
            x: ry.sin(),
            y: 0.0,
            z: ry.cos(),
        },
    );

    let matz = Matrix3::from_cols(
        Vector3 {
            x: rz.cos(),
            y: rz.sin(),
            z: 0.0,
        },
        Vector3 {
            x: -rz.sin(),
            y: rz.cos(),
            z: 0.0,
        },
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
    );

    return matz * maty * matz;
}

