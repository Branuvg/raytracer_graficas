// sphere.rs
use raylib::prelude::*;
use crate::ray_intersect::{RayIntersect, Intersect, Material};

pub struct Sphere {
    pub center: Vector3,
    pub radius: f32,
    pub material: Material,
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        
        let oc = *ray_origin - self.center;

        let a = ray_direction.dot(*ray_direction);
        let b = 2.0 * oc.dot(*ray_direction);
        let c = oc.dot(oc) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant > 0.0 {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t > 0.0 {
                return Intersect::new(self.material, t);
            }
        }

        Intersect::empty()
    }
}