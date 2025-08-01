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
    for object in objects {
        if object.ray_intersect(ray_origin, ray_direction) {
            return Color::new(157, 165, 189, 255);
        }
    }
    Color::new(4, 12, 36, 255)
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere]) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    // Los bucles ahora usan i32, que es el tipo de framebuffer.width/height
    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * (x as f32 + 0.5)) / width - 1.0;
            let screen_y = -(2.0 * (y as f32 + 0.5)) / height + 1.0;

            let camera_x = screen_x * aspect_ratio * perspective_scale;
            let camera_y = screen_y * perspective_scale;

            let ray_direction = Vector3::new(camera_x, camera_y, -1.0).normalized();
            
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
        .title("Raytracer en Rust")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as i32, window_height as i32);

    framebuffer.set_background_color(Color::new(4, 12, 36, 255));

    let objects = [Sphere {
        center: Vector3::new(0.0, 0.0, -5.0),
        radius: 1.0,
    }];

    while !rl.window_should_close() {
        framebuffer.clear();

        render(&mut framebuffer, &objects);

        framebuffer.swap_buffers(&mut rl, &thread);
    }
}
