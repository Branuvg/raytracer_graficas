// main.rs
#![allow(unused_imports)]
#![allow(dead_code)]

use raylib::prelude::*;
use std::f32::consts::PI;

mod framebuffer;
mod ray_intersect;
mod sphere;
mod camera;
mod material;
mod light; 

use framebuffer::Framebuffer;
use ray_intersect::{RayIntersect, Intersect};
use sphere::Sphere;
use camera::Camera;
use material::{Material, vector3_to_color, color_to_vector3};
use light::Light;

fn reflect(incident: &Vector3, normal: &Vector3) -> Vector3 {
    *incident - *normal * 2.0 * incident.dot(*normal)
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Sphere],
) -> f32 {
    let light_direction = (light.position - intersect.point).normalized();
    let shadow_ray_origin = intersect.point;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_direction);
        if shadow_intersect.is_intersecting {
            return 0.8; //cambiar esto a una proporcion de la distancia para que haga el sh
        }
    }
    0.0
}

const SKYBOX_COLOR: Color = Color::new(4, 12, 36, 255);

pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[Sphere],
    light: &Light,
    depth: u32,
) -> Color {
    if depth > 3 {
        return SKYBOX_COLOR;
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let tmp = object.ray_intersect(ray_origin, ray_direction);
        if tmp.is_intersecting {
            if tmp.distance < zbuffer {
                zbuffer = tmp.distance;
                intersect = tmp;
            }
        }
    }

    if !intersect.is_intersecting {
        return SKYBOX_COLOR;  //color del fondo (SKYBOX_COLOR)
    }
    
    let light_direction = (light.position - intersect.point).normalized();
    let view_direction = (*ray_origin - intersect.point).normalized();
    let reflection_direction = reflect(&-light_direction, &-intersect.normal).normalized();

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);
    
    // Difuso
    let diffuse_intensity = intersect.normal.dot(light_direction).max(0.0) * light_intensity;
    let diffuse = intersect.material.diffuse * diffuse_intensity;
    
    // Especular
    let specular_intensity = view_direction.dot(reflection_direction).max(0.0).powf(intersect.material.specular) * light_intensity;
    let specular = light.color * specular_intensity;
    
    // Reflejo
    let reflection_color = color_to_vector3(SKYBOX_COLOR);
    let reflectivity = intersect.material.reflectivity;

    // Color final
    let color = diffuse * intersect.material.albedo[0] + specular * intersect.material.albedo[1] + reflection_color * reflectivity; // [diffuse,specular] * albedo (su energia) + reflection_color * reflectivity

    vector3_to_color(color)
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere], camera: &Camera, light: &Light) {

    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();

            let rotated_direction = camera.basis_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0);

            framebuffer.set_current_color(pixel_color);
            framebuffer.set_pixel(x, y);
        }
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raytracer Class")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as i32, window_height as i32);

    let rubber = Material {
        diffuse: Vector3::new(0.3, 0.1, 0.1),
        albedo: [0.9,0.1],
        specular: 5.0,
        reflectivity: 0.0,
    };

    let ivory = Material {
        diffuse: Vector3::new(0.4, 0.4, 0.3),
        albedo: [0.6,0.3],
        specular: 50.0,
        reflectivity: 0.3,
    };

    let mirror = Material {
        diffuse: Vector3::new(1.0, 1.0, 1.0),
        albedo: [0.0,10.0],
        specular: 1500.0,
        reflectivity: 0.8,
    };

    let objects = [
        Sphere {
            center: Vector3::new(1.0, 0.0, -4.0),
            radius: 1.0,
            material: ivory,
        },
        Sphere {
            center: Vector3::new(0.0, 0.0, 0.0),
            radius: 1.0,
            material: rubber,
        },
        Sphere {
            center: Vector3::new(1.0, 1.0, 1.0),
            radius: 0.5,
            material: rubber,
        },
        Sphere {
            center: Vector3::new(1.0, -1.0, 1.0),
            radius: 0.7,
            material: mirror,
        },
    ];

    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 5.0),  // eye
        Vector3::new(0.0, 0.0, 0.0),  // center
        Vector3::new(0.0, 1.0, 0.0),  // up
    );
    let rotation_speed = PI / 100.0;

    let light = Light::new(
        Vector3::new(5.0, 5.0, 5.0), // position
        Vector3::new(1.0, 1.0, 1.0), // color
        1.5, // intensity
    );

    while !window.window_should_close() {
        framebuffer.clear();

        // camera controls
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.orbit(0.0, rotation_speed);
        }

        render(&mut framebuffer, &objects, &camera, &light);

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}