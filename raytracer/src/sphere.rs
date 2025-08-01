// sphere.rs
use raylib::prelude::*;
use crate::ray_intersect::RayIntersect;

pub struct Sphere {
    pub center: Vector3,
    pub radius: f32,
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> bool {
        
        let oc = *ray_origin - self.center;
        let a = ray_direction.dot(*ray_direction);
        let b = 2.0 * oc.dot(*ray_direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        discriminant >= 0.0
    }
}