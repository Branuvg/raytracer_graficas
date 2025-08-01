#![allow(unused_imports)]
#![allow(dead_code)]

use raylib::prelude::*;
use std::f32::consts::PI;

mod framebuffer;
mod ray_intersect;
mod sphere;

use framebuffer::Framebuffer;
use ray_intersect::RayIntersect;
use sphere::Sphere;

pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[Sphere],
) -> Color {
    // --- LÓGICA DE PROFUNDIDAD CORREGIDA ---
    let mut closest_t = f32::MAX;
    let mut hit_color = Color::new(4, 12, 36, 255); // Color de fondo por defecto

    // Itera sobre todos los objetos.
    for object in objects {
        // `ray_intersect` ahora devuelve Option<f32>. Usamos `if let` para manejarlo.
        if let Some(t) = object.ray_intersect(ray_origin, ray_direction) {
            // Si la distancia 't' es menor que la más cercana encontrada...
            if t < closest_t {
                // ...actualizamos la distancia y guardamos el color del objeto actual.
                closest_t = t;
                hit_color = object.color;
            }
        }
    }
    // Devolvemos el color del objeto más cercano que fue encontrado.
    hit_color
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere]) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * (x as f32 + 0.5)) / width - 1.0;
            let screen_y = -(2.0 * (y as f32 + 0.5)) / height + 1.0;

            let camera_x = screen_x * aspect_ratio * perspective_scale;
            let camera_y = screen_y * perspective_scale;

            let mut ray_direction = Vector3::new(camera_x, camera_y, -1.0);
            ray_direction.normalize();
            
            let ray_origin = Vector3::new(0.0, 0.0, 0.0);

            let pixel_color = cast_ray(&ray_origin, &ray_direction, objects);

            framebuffer.set_current_color(pixel_color);
            framebuffer.set_pixel(x, y);
        }
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;

    let (mut rl, thread) = raylib::init()
        .size(window_width, window_height)
        .title("Muñeco de Nieve Detallado - Raytracer")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as i32, window_height as i32);

    framebuffer.set_background_color(Color::new(4, 12, 36, 255));

    let objects = [
        Sphere {
            center: Vector3::new(0.0, -1.5, -7.0),
            radius: 1.5,
            color: Color::new(240, 240, 240, 255),
        },
        Sphere {
            center: Vector3::new(0.0, 0.0, -7.0),
            radius: 1.0,
            color: Color::new(240, 240, 240, 255),
        },
        Sphere {
            center: Vector3::new(0.0, 1.2, -7.0),
            radius: 0.7,
            color: Color::new(240, 240, 240, 255),
        },
        Sphere {
            center: Vector3::new(-0.25, 1.4, -6.25),
            radius: 0.1,
            color: Color::BLACK,
        },
        Sphere {
            center: Vector3::new(0.25, 1.4, -6.25),
            radius: 0.1,
            color: Color::BLACK,
        },
        Sphere {
            center: Vector3::new(0.0, 1.15, -6.2),
            radius: 0.15,
            color: Color::ORANGE,
        },
        Sphere {
            center: Vector3::new(-0.4, 0.95, -6.25),
            radius: 0.07,
            color: Color::BLACK,
        },
        Sphere {
            center: Vector3::new(-0.2, 0.85, -6.25),
            radius: 0.07,
            color: Color::BLACK,
        },
        Sphere {
            center: Vector3::new(0.0, 0.8, -6.25),
            radius: 0.07,
            color: Color::BLACK,
        },
        Sphere {
            center: Vector3::new(0.2, 0.85, -6.25),
            radius: 0.07,
            color: Color::BLACK,
        },
        Sphere {
            center: Vector3::new(0.4, 0.95, -6.25),
            radius: 0.07,
            color: Color::BLACK,
        },
    ];

    while !rl.window_should_close() {
        framebuffer.clear();

        render(&mut framebuffer, &objects);

        framebuffer.swap_buffers(&mut rl, &thread);
    }
}
