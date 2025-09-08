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
mod snell;
mod textures;

use framebuffer::Framebuffer;
use ray_intersect::{RayIntersect, Intersect};
use sphere::Sphere;
use camera::Camera;
use material::{Material, vector3_to_color};
use light::Light;
use snell::{reflect, refract};
use textures::TextureManager;

fn procedural_sky(dir: Vector3) -> Vector3 { //color del fondo, si se quiere un color solido, usar SKYBOX_COLOR (ultima linea de esta función) y cambiar donde diga //color del fondo
    let d = dir.normalized();
    let t = (d.y + 1.0) * 0.5; // map y [-1,1] → [0,1]

    let green = Vector3::new(0.1, 0.6, 0.2); // grass green
    let white = Vector3::new(1.0, 1.0, 1.0); // horizon haze
    let blue = Vector3::new(0.3, 0.5, 1.0);  // sky blue

    if t < 0.54 {
        // Bottom → fade green to white
        let k = t / 0.55;
        green * (1.0 - k) + white * k
    } else if t < 0.55 {
        // Around horizon → mostly white
        white
    } else if t < 0.8 {
        // Fade white to blue
        let k = (t - 0.55) / (0.25);
        white * (1.0 - k) + blue * k
    } else {
        // Upper sky → solid blue
        blue
    }
}
//const SKYBOX_COLOR: Vector3 = Vector3::new(0.01, 0.3, 0.8);

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
            return 0.7; //cambiar esto a una proporcion de la distancia para que haga el shadow
        }
    }
    0.0
}

const ORIGIN_BIAS: f32 = 1e-4;
fn offset_origin(intersect: &Intersect, ray_direction: &Vector3) -> Vector3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if ray_direction.dot(intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[Sphere],
    light: &Light,
    depth: u32,
    texture_manager: &TextureManager,
) -> Vector3 {
    
    if depth > 3 { //cantidad de rebotes que puede dar el rayo
        return procedural_sky(*ray_direction); //color del fondo
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
        return procedural_sky(*ray_direction);  //color del fondo
    }
    
    let light_direction = (light.position - intersect.point).normalized();
    let view_direction = (*ray_origin - intersect.point).normalized();
    
    let mut normal = intersect.normal;
    if let Some(normal_map_path) = &intersect.material.normal_map_id {
        let texture = texture_manager.get_texture(normal_map_path).unwrap();
        let width = texture.width() as u32;
        let height = texture.height() as u32;
        let tx = (intersect.u * width as f32) as u32;
        let ty = (intersect.v * height as f32) as u32;
        if let Some(tex_normal) = texture_manager.get_normal_from_map(normal_map_path, tx, ty) {
            let tangent = Vector3::new(normal.y, -normal.x, 0.0).normalized();
            let bitangent = normal.cross(tangent);
            let transformed_normal_x = tex_normal.x * tangent.x + tex_normal.y * bitangent.x + tex_normal.z * normal.x;
            let transformed_normal_y = tex_normal.x * tangent.y + tex_normal.y * bitangent.y + tex_normal.z * normal.y;
            let transformed_normal_z = tex_normal.x * tangent.z + tex_normal.y * bitangent.z + tex_normal.z * normal.z;
            
            normal = Vector3::new(transformed_normal_x, transformed_normal_y, transformed_normal_z).normalized();
        }
    }

    let reflection_direction = reflect(&-light_direction, &normal).normalized();
    
    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);
    
    // Difuso
    let diffuse_intensity = normal.dot(light_direction).max(0.0) * light_intensity;

    let diffuse_color = if let Some(texture_path) = &intersect.material.texture {
        let texture = texture_manager.get_texture(texture_path).unwrap();
        let width = texture.width() as u32;
        let height = texture.height() as u32;
        let tx = (intersect.u * width as f32) as u32;
        let ty = (intersect.v * height as f32) as u32;
        let color = texture_manager.get_pixel_color(texture_path, tx, ty);
        color
    } else {
        intersect.material.diffuse
    };

    let diffuse = diffuse_color * diffuse_intensity;
    
    // Especular
    let specular_intensity = view_direction.dot(reflection_direction).max(0.0).powf(intersect.material.specular) * light_intensity;
    let specular = light.color * specular_intensity;
    
    // Reflejo
    let mut reflection_color = procedural_sky(*ray_direction); //color del fondo
    let reflectivity = intersect.material.reflectivity;

    if reflectivity > 0.0 {
        let reflect_direction = reflect(ray_direction, &normal);
        let reflect_origin = intersect.point;
        reflection_color = cast_ray(&reflect_origin, &reflect_direction, objects, light, depth + 1, texture_manager);
    }

    //Transparencia
    let transparency = intersect.material.transparency;
    let mut refraction_color = Vector3::zero();

    if transparency > 0.0 {
        let refract_direction = refract(ray_direction, &normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_direction);

        refraction_color = cast_ray(&refract_origin, &refract_direction, objects, light, depth + 1, texture_manager);
    }

    // Color final
    let color = diffuse * intersect.material.albedo[0] + specular * intersect.material.albedo[1] + reflection_color * reflectivity + refraction_color * transparency; // [diffuse,specular] * albedo (su energia) + reflection_color * reflectivity

    color
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere], camera: &Camera, light: &Light, texture_manager: &TextureManager) {

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

            let pixel_color_vec = cast_ray(&camera.eye, &rotated_direction, objects, light, 0, texture_manager);
            let pixel_color = vector3_to_color(pixel_color_vec);

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

    let mut texture_manager = TextureManager::new();
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/bricks.jpg");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/bricks_normal.png");

    let mut framebuffer = Framebuffer::new(window_width as i32, window_height as i32);

    let rubber = Material {
        diffuse: Vector3::new(0.3, 0.1, 0.1),
        albedo: [0.9,0.1],
        specular: 5.0,
        reflectivity: 0.0,
        transparency: 0.0,
        refractive_index: 0.0,
        texture: Some("assets/bricks.jpg".to_string()),
        normal_map_id: Some("assets/bricks_normal.png".to_string()),
    };

    let ivory = Material {
        diffuse: Vector3::new(0.4, 0.4, 0.3),
        albedo: [0.6,0.3],
        specular: 50.0,
        reflectivity: 0.3,
        transparency: 0.0,
        refractive_index: 0.0,
        texture: None,
        normal_map_id: None,
    };

    let mirror = Material { //NOT A GLASS BALL
        diffuse: Vector3::new(1.0, 1.0, 1.0),
        albedo: [0.0,10.0],
        specular: 1500.0,
        reflectivity: 0.9,
        transparency: 0.1,
        refractive_index: 1.5,
        texture: None,
        normal_map_id: None,
    };

    let glass = Material {
        diffuse: Vector3::new(1.0, 1.0, 1.0),
        albedo: [0.0,5.0],
        specular: 125.0,
        reflectivity: 0.1,
        transparency: 0.9,
        refractive_index: 1.5,
        texture: None,
        normal_map_id: None,
    };

    let objects = [
        Sphere {
            center: Vector3::new(0.0, 0.0, 0.0),
            radius: 1.0,
            material: rubber.clone(),
        },
        Sphere {
            center: Vector3::new(1.0, 1.0, 1.0),
            radius: 0.5,
            material: rubber.clone(),
        },
        Sphere {
            center: Vector3::new(2.0, 0.0, -4.0),
            radius: 1.0,
            material: ivory,
        },
        Sphere {
            center: Vector3::new(2.0, -0.5, -1.0),
            radius: 0.7,
            material: mirror,
        },
        Sphere {
            center: Vector3::new(-1.5, 0.0, -1.0),
            radius: 0.5,
            material: glass,
        },
    ];

    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 5.0),  // eye
        Vector3::new(0.0, 0.0, 0.0),  // center
        Vector3::new(0.0, 1.0, 0.0),  // up
    );
    let rotation_speed = PI / 100.0;
    let zoom_speed = 0.1;

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
        if window.is_key_down(KeyboardKey::KEY_W) {
            camera.zoom(zoom_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            camera.zoom(-zoom_speed);
        }

        render(&mut framebuffer, &objects, &camera, &light, &texture_manager);
        
        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}