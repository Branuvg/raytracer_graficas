use raylib::prelude::Vector3;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::Material;

pub struct Cube {
    pub center: Vector3,
    pub size: f32,
    pub material: Material,
}

impl Cube {
    pub fn new(center: Vector3, size: f32, material: Material) -> Self {
        Self { center, size, material }
    }

    fn get_uv(&self, point: &Vector3, normal: &Vector3) -> (f32, f32) {
        let local_point = *point - self.center;
        let half_size = self.size * 0.5;
        
        // Determinar qué cara del cubo basándose en la normal
        let (u, v) = if normal.x.abs() > 0.9 {
            // Cara X (izquierda/derecha)
            let u = (local_point.z + half_size) / self.size;
            let v = 1.0 - (local_point.y + half_size) / self.size; // Invertir V para orientación correcta
            (u, v)
        } else if normal.y.abs() > 0.9 {
            // Cara Y (arriba/abajo)
            let u = (local_point.x + half_size) / self.size;
            let v = if normal.y > 0.0 {
                (local_point.z + half_size) / self.size
            } else {
                1.0 - (local_point.z + half_size) / self.size
            };
            (u, v)
        } else {
            // Cara Z (frente/atrás)
            let u = if normal.z > 0.0 {
                1.0 - (local_point.x + half_size) / self.size
            } else {
                (local_point.x + half_size) / self.size
            };
            let v = 1.0 - (local_point.y + half_size) / self.size;
            (u, v)
        };
        
        (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        let half_size = self.size * 0.5;
        let min_bounds = self.center - Vector3::new(half_size, half_size, half_size);
        let max_bounds = self.center + Vector3::new(half_size, half_size, half_size);
        
        // Calcular t para cada par de planos
        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;
        let mut normal = Vector3::zero();
        
        // Eje X
        if ray_direction.x.abs() > 1e-8 {
            let tx1 = (min_bounds.x - ray_origin.x) / ray_direction.x;
            let tx2 = (max_bounds.x - ray_origin.x) / ray_direction.x;
            
            let (tx_near, tx_far) = if tx1 < tx2 { (tx1, tx2) } else { (tx2, tx1) };
            
            if tx_near > tmin {
                tmin = tx_near;
                normal = if tx1 < tx2 { Vector3::new(-1.0, 0.0, 0.0) } else { Vector3::new(1.0, 0.0, 0.0) };
            }
            if tx_far < tmax {
                tmax = tx_far;
            }
        } else {
            // Rayo paralelo al eje X
            if ray_origin.x < min_bounds.x || ray_origin.x > max_bounds.x {
                return Intersect::empty();
            }
        }
        
        if tmin > tmax {
            return Intersect::empty();
        }
        
        // Eje Y
        if ray_direction.y.abs() > 1e-8 {
            let ty1 = (min_bounds.y - ray_origin.y) / ray_direction.y;
            let ty2 = (max_bounds.y - ray_origin.y) / ray_direction.y;
            
            let (ty_near, ty_far) = if ty1 < ty2 { (ty1, ty2) } else { (ty2, ty1) };
            
            if ty_near > tmin {
                tmin = ty_near;
                normal = if ty1 < ty2 { Vector3::new(0.0, -1.0, 0.0) } else { Vector3::new(0.0, 1.0, 0.0) };
            }
            if ty_far < tmax {
                tmax = ty_far;
            }
        } else {
            // Rayo paralelo al eje Y
            if ray_origin.y < min_bounds.y || ray_origin.y > max_bounds.y {
                return Intersect::empty();
            }
        }
        
        if tmin > tmax {
            return Intersect::empty();
        }
        
        // Eje Z
        if ray_direction.z.abs() > 1e-8 {
            let tz1 = (min_bounds.z - ray_origin.z) / ray_direction.z;
            let tz2 = (max_bounds.z - ray_origin.z) / ray_direction.z;
            
            let (tz_near, tz_far) = if tz1 < tz2 { (tz1, tz2) } else { (tz2, tz1) };
            
            if tz_near > tmin {
                tmin = tz_near;
                normal = if tz1 < tz2 { Vector3::new(0.0, 0.0, -1.0) } else { Vector3::new(0.0, 0.0, 1.0) };
            }
            if tz_far < tmax {
                tmax = tz_far;
            }
        } else {
            // Rayo paralelo al eje Z
            if ray_origin.z < min_bounds.z || ray_origin.z > max_bounds.z {
                return Intersect::empty();
            }
        }
        
        if tmin > tmax {
            return Intersect::empty();
        }
        
        // Elegir el t más cercano que sea positivo
        let t = if tmin > 1e-6 { 
            tmin 
        } else if tmax > 1e-6 { 
            tmax 
        } else { 
            return Intersect::empty(); 
        };
        
        let point = *ray_origin + *ray_direction * t;
        let (u, v) = self.get_uv(&point, &normal);
        
        Intersect::new(
            self.material.clone(),
            t,
            normal,
            point,
            u,
            v
        )
    }
}