// material.rs
use raylib::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Vector3, // Color
    pub albedo: [f32; 2], // que tan colorido es: [color del objeto, color que viene de la luz]
    pub specular: f32, // brillo
}

impl Material {
    pub fn new(diffuse: Vector3, albedo: [f32; 2], specular: f32) -> Self {
        Material {
            diffuse,
            albedo,
            specular,
        }
    }
    
    pub fn black() -> Self {
        Material {
            diffuse: Vector3::zero(),
            albedo: [0.0, 0.0],
            specular: 0.0,
        }
    }
}

pub fn vector3_to_color(v: Vector3) -> Color {
    Color::new(
        (v.x * 255.0).min(255.0) as u8,
        (v.y * 255.0).min(255.0) as u8,
        (v.z * 255.0).min(255.0) as u8,
        255,
    )
}