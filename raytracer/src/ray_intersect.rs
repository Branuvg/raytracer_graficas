// ray_intersect.rs
use raylib::prelude::*;

pub trait RayIntersect {
    // CAMBIO: Ahora devuelve la distancia a la intersección (f32) si existe.
    // Option<f32> significa que puede devolver una distancia (Some(distancia)) o nada (None).
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Option<f32>;
}