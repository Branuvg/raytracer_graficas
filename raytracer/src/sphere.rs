use crate::ray_intersect::RayIntersect;
use raylib::prelude::*;

pub struct Sphere {
    pub center: Vector3,
    pub radius: f32,
    pub color: Color,
}

impl RayIntersect for Sphere {
    // CAMBIO: La función ahora devuelve Option<f32> para cumplir con el trait.
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Option<f32> {
        let oc = *ray_origin - self.center;
        let a = ray_direction.dot(*ray_direction);
        let b = 2.0 * oc.dot(*ray_direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            // No hay intersección, devuelve None.
            None
        } else {
            // Calcula las dos posibles distancias.
            let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
            let t2 = (-b + discriminant.sqrt()) / (2.0 * a);

            // Devuelve la intersección positiva más cercana.
            if t1 > 0.001 {
                Some(t1)
            } else if t2 > 0.001 {
                Some(t2)
            } else {
                None
            }
        }
    }
}
