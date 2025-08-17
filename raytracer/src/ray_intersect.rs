// ray_intersect.rs
use raylib::prelude::{Color, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Color,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Intersect {
    pub material: Material,
    pub is_intersecting: bool,
}

impl Intersect {
    pub fn new(material: Material) -> Self {
        Intersect {
            material,
            is_intersecting: true,
        }
    }

    pub fn empty() -> Self {
        Intersect {
            material: Material {
                diffuse: Color::new(0, 0, 0, 0),
            },
            is_intersecting: false,
        }
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect;
}