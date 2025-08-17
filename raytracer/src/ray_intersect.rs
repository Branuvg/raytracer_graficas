// ray_intersect.rs
use raylib::prelude::{Color, Vector3};
use crate::material::Material;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]

pub struct Intersect {
    pub material: Material,
    pub distance: f32,
    pub is_intersecting: bool,
}

impl Intersect {
    pub fn new(material: Material, distance: f32) -> Self {
        Intersect {
            material,
            distance,
            is_intersecting: true,
        }
    }

    pub fn empty() -> Self {
        Intersect {
            material: Material {
                diffuse: Vector3::zero(),
                albedo: [0.0],
            },
            distance: 0.0,
            is_intersecting: false,
        }
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect;
}