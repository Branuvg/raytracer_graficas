// src/main.rs
#![allow(unused_imports)]
#![allow(dead_code)]
use raylib::prelude::*;
use std::f32::consts::PI;
use rayon::prelude::*;
use std::mem::size_of;

mod framebuffer;
mod ray_intersect;
mod cube;
mod camera; //poner sphere si se quiere usar esferas
mod material;
mod light;
mod snell;
mod textures;
use framebuffer::Framebuffer;
use ray_intersect::{RayIntersect, Intersect};
use cube::Cube;
use camera::Camera;
use material::{Material, vector3_to_color};
use light::Light;
use snell::{reflect, refract};
use textures::{TextureManager, SkyboxTextures};

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Cube], //poner sphere si se quiere usar esferas
) -> f32 {
    let light_direction = (light.position - intersect.point).normalized();
    let shadow_ray_origin = intersect.point + intersect.normal * 0.001; // Bias para evitar auto-intersección
    let light_distance = (light.position - shadow_ray_origin).length();
    
    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_direction);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            return 0.7;
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
    objects: &[Cube],
    light: &Light,
    emissive_objects: &[&Cube],
    depth: u32,
    texture_manager: &TextureManager,
) -> Vector3 {
    if depth > 1 { // Limitar profundidad para rendimiento
        return texture_manager.sample_skybox(*ray_direction);
    }
    
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;
    for object in objects {
        let tmp = object.ray_intersect(ray_origin, ray_direction);
        if tmp.is_intersecting && tmp.distance < zbuffer {
            zbuffer = tmp.distance;
            intersect = tmp;
        }
    }
    
    if !intersect.is_intersecting {
        return texture_manager.sample_skybox(*ray_direction);
    }
    
    // El material emite su propio color, que se suma a la luz reflejada.
    let emission = intersect.material.emission;
    
    // Cálculo de Iluminación (Directa e Indirecta)
    let mut total_diffuse_intensity = 0.0;
    let mut total_specular = Vector3::zero();

    // Crear una lista de todas las fuentes de luz para esta intersección
    let mut lights: Vec<Light> = vec![*light];

    // Añadir luces desde los objetos emisivos
    for emissive_cube in emissive_objects {
        let cube_center = (emissive_cube.min_bounds + emissive_cube.max_bounds) * 0.5;
        let diff_vec = cube_center - intersect.point;
        if diff_vec.dot(diff_vec) < 0.01 { continue; }
        
        lights.push(Light::new(
            cube_center,
            emissive_cube.material.emission.normalized(),
            emissive_cube.material.emission.length()
        ));
    }
    
    let view_direction = (*ray_origin - intersect.point).normalized();
    let mut normal = intersect.normal;
    // ... (lógica de normal map sin cambios) ...
    if let Some(normal_map_path) = &intersect.material.normal_map_id {
        let texture = texture_manager.get_texture(normal_map_path).unwrap();
        let width = texture.width() as u32; let height = texture.height() as u32;
        let tx = (intersect.u * width as f32) as u32; let ty = (intersect.v * height as f32) as u32;
        if let Some(tex_normal) = texture_manager.get_normal_from_map(normal_map_path, tx, ty) {
            let tangent = Vector3::new(normal.y, -normal.x, 0.0).normalized();
            let bitangent = normal.cross(tangent);
            let transformed_normal_x = tex_normal.x * tangent.x + tex_normal.y * bitangent.x + tex_normal.z * normal.x;
            let transformed_normal_y = tex_normal.x * tangent.y + tex_normal.y * bitangent.y + tex_normal.z * normal.y;
            let transformed_normal_z = tex_normal.x * tangent.z + tex_normal.y * bitangent.z + tex_normal.z * normal.z;
            normal = Vector3::new(transformed_normal_x, transformed_normal_y, transformed_normal_z).normalized();
        }
    }
    
    // Iterar sobre todas las luces (la principal y las de los objetos emisivos)
    for current_light in &lights {
        let light_direction = (current_light.position - intersect.point).normalized();
        let reflection_direction = reflect(&-light_direction, &normal).normalized();
        
        let shadow_intensity = cast_shadow(&intersect, current_light, objects);
        let light_intensity = current_light.intensity * (1.0 - shadow_intensity);
        
        total_diffuse_intensity += normal.dot(light_direction).max(0.0) * light_intensity;
        
        let specular_intensity = view_direction.dot(reflection_direction).max(0.0).powf(intersect.material.specular) * light_intensity;
        total_specular += current_light.color * specular_intensity;
    }

    let diffuse_color = if let Some(texture_path) = &intersect.material.texture {
        let texture = texture_manager.get_texture(texture_path).unwrap();
        let width = texture.width() as u32; let height = texture.height() as u32;
        let tx = (intersect.u * width as f32) as u32; let ty = (intersect.v * height as f32) as u32;
        texture_manager.get_pixel_color(texture_path, tx, ty)
    } else {
        intersect.material.diffuse
    };
    let diffuse = diffuse_color * total_diffuse_intensity;
    let specular = total_specular;
    
    // Reflejo
    let mut reflection_color = Vector3::zero();
    let reflectivity = intersect.material.reflectivity;
    if reflectivity > 0.0 {
        let reflect_direction = reflect(ray_direction, &normal);
        let reflect_origin = offset_origin(&intersect, &reflect_direction);
        reflection_color = cast_ray(&reflect_origin, &reflect_direction, objects, light, emissive_objects, depth + 1, texture_manager);
    }
    
    // Transparencia
    let mut refraction_color = Vector3::zero();
    let transparency = intersect.material.transparency;
    if transparency > 0.0 {
        let refract_direction = refract(ray_direction, &normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_direction);
        refraction_color = cast_ray(&refract_origin, &refract_direction, objects, light, emissive_objects, depth + 1, texture_manager);
    }
    
    // Color final = Emisión + Luz Recibida (Difusa y Especular) + Reflejos + Refracciones
    let color = emission + 
                diffuse * intersect.material.albedo[0] + 
                specular * intersect.material.albedo[1] + 
                reflection_color * reflectivity + 
                refraction_color * transparency;
    color
}

pub fn render(
    width: i32,
    height: i32,
    objects: &[Cube],
    camera: &Camera,
    light: &Light,
    emissive_objects: &[&Cube],
    texture_manager: &TextureManager,
) -> Vec<Color> {
    let aspect_ratio = width as f32 / height as f32;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();
    let camera_eye = camera.eye;

    (0..height)
        .into_par_iter()
        .flat_map(|y| (0..width).into_par_iter().map(move |x| (x, y)))
        .map(|(x, y)| {
            let screen_x = (2.0 * x as f32) / width as f32 - 1.0;
            let screen_y = -(2.0 * y as f32) / height as f32 + 1.0;
            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;
            let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();
            let rotated_direction = camera.basis_change(&ray_direction);
            let pixel_color_vec = cast_ray(
                &camera_eye,
                &rotated_direction,
                objects,
                light,
                emissive_objects,
                0,
                texture_manager,
            );
            vector3_to_color(pixel_color_vec)
        })
        .collect()
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raytracer Minecraft")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    
    let mut texture_manager = TextureManager::new();
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/grass.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/glass.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/magma.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/diamond_ore.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/oak.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/wood_planks.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/stone.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/obsidian.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/water.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/leaves.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/dirt.png");

    let skybox = SkyboxTextures {
        front: "assets/skybox/front.png".to_string(),
        back: "assets/skybox/back.png".to_string(),
        left: "assets/skybox/left.png".to_string(),
        right: "assets/skybox/right.png".to_string(),
        top: "assets/skybox/top.png".to_string(),
        bottom: "assets/skybox/bottom.png".to_string(),
    };

    texture_manager.load_skybox(&mut window, &raylib_thread, skybox);
    
    let zero_emission = Vector3::zero();

    let glass = Material {
        diffuse: Vector3::new(1.0, 1.0, 1.0), albedo: [0.0,5.0], specular: 125.0, reflectivity: 0.1, 
        transparency: 0.9, refractive_index: 1.5, texture: Some("assets/glass.png".to_string()), 
        normal_map_id: None, emission: zero_emission,
    };
    let dirt = Material {
        diffuse: Vector3::new(0.4, 0.26, 0.13), albedo: [0.8, 0.2], specular: 1.0, reflectivity: 0.0, 
        transparency: 0.0, refractive_index: 1.0, texture: Some("assets/dirt.png".to_string()), 
        normal_map_id: None, emission: zero_emission,
    };
    let grass = Material {
        diffuse: Vector3::new(0.2, 0.6, 0.2), albedo: [0.7, 0.3], specular: 2.0, reflectivity: 0.0, 
        transparency: 0.0, refractive_index: 1.0, texture: Some("assets/grass.png".to_string()), 
        normal_map_id: None, emission: zero_emission,
    };
    let leaves = Material {
        diffuse: Vector3::new(0.1, 0.5, 0.1), albedo: [0.6, 0.4], specular: 3.0, reflectivity: 0.0, 
        transparency: 0.0, refractive_index: 1.2, texture: Some("assets/leaves.png".to_string()), 
        normal_map_id: None, emission: zero_emission,
    };
    let magma = Material {
        diffuse: Vector3::new(1.0, 0.3, 0.0), albedo: [0.9, 0.1], specular: 50.0, reflectivity: 0.0, 
        transparency: 0.0, refractive_index: 1.0, texture: Some("assets/magma.png".to_string()), 
        normal_map_id: None, emission: Vector3::new(1.0, 0.4, 0.1) * 0.5, // <-- Emite luz naranja
    };
    let oak = Material {
        diffuse: Vector3::new(0.6, 0.4, 0.2), albedo: [0.8, 0.2], specular: 5.0, reflectivity: 0.0, 
        transparency: 0.0, refractive_index: 1.0, texture: Some("assets/oak.png".to_string()), 
        normal_map_id: None, emission: zero_emission,
    };
    let wood_planks = Material {
        diffuse: Vector3::new(0.6, 0.4, 0.2), albedo: [0.8, 0.2], specular: 5.0, reflectivity: 0.0, 
        transparency: 0.0, refractive_index: 1.0, texture: Some("assets/wood_planks.png".to_string()), 
        normal_map_id: None, emission: zero_emission,
    };
    let stone = Material {
        diffuse: Vector3::new(0.5, 0.5, 0.5), albedo: [0.7, 0.3], specular: 8.0, reflectivity: 0.0, 
        transparency: 0.0, refractive_index: 0.5, texture: Some("assets/stone.png".to_string()), 
        normal_map_id: None, emission: zero_emission,
    };
    let diamond_ore = Material {
        diffuse: Vector3::new(0.4, 0.4, 0.4), albedo: [0.6, 0.4], specular: 20.0, reflectivity: 0.01, 
        transparency: 0.0, refractive_index: 0.5, texture: Some("assets/diamond_ore.png".to_string()), 
        normal_map_id: None, emission: zero_emission,
    };
    let obsidian = Material {
        diffuse: Vector3::new(0.1, 0.05, 0.15), albedo: [0.8, 0.2], specular: 10.0, reflectivity: 0.1, 
        transparency: 0.0, refractive_index: 1.0, texture: Some("assets/obsidian.png".to_string()), 
        normal_map_id: None, emission: zero_emission,
    };
    let water = Material {
        diffuse: Vector3::new(0.0, 0.3, 0.9), albedo: [0.6, 0.4], specular: 30.0, reflectivity: 0.15, 
        transparency: 0.6, refractive_index: 1.2, texture: Some("assets/water.png".to_string()), 
        normal_map_id: None, emission: zero_emission,
    };

    let mut objects: Vec<Cube> = Vec::new();
    // ... (la creación de la escena con bucles no cambia) ...
    for x_int in -5..=-1 {
        for z_int in -5..=1 {
            if (x_int == -4 || x_int == -3) && (z_int == 0 || z_int == -1) { continue; }
            if x_int == -2 && z_int == -4 { continue; } // Dejar hueco para el magma
            objects.push(Cube::new(Vector3::new(x_int as f32, 0.0, z_int as f32), 1.0, stone.clone()));
        }
    }
    objects.push(Cube::new(Vector3::new(-3.5, -0.5, -0.5), 2.0, water.clone()));
    objects.push(Cube::new(Vector3::new(-2.0, 0.0, -4.0), 1.0, magma.clone()));

    for y in 1..=3 {
        objects.push(Cube::new(Vector3::new(-5.0, y as f32, -4.0), 1.0, stone.clone()));
        objects.push(Cube::new(Vector3::new(-1.0, y as f32, -4.0), 1.0, stone.clone()));
    }
    objects.push(Cube::new(Vector3::new(-4.0, 1.0, -5.0), 1.0, diamond_ore.clone()));
    objects.push(Cube::new(Vector3::new(-4.0, 2.0, -5.0), 1.0, stone.clone()));
    objects.push(Cube::new(Vector3::new(-3.0, 1.0, -5.0), 1.0, diamond_ore.clone()));
    objects.push(Cube::new(Vector3::new(-3.0, 2.0, -5.0), 1.0, diamond_ore.clone()));
    objects.push(Cube::new(Vector3::new(-2.0, 1.0, -5.0), 1.0, stone.clone()));
    objects.push(Cube::new(Vector3::new(-2.0, 2.0, -5.0), 1.0, stone.clone()));
    for x in -4..=-2 { objects.push(Cube::new(Vector3::new(x as f32, 3.0, -4.0), 1.0, stone.clone())); }
    for y in 1..=3 {
        objects.push(Cube::new(Vector3::new(0.0, y as f32, 1.0), 1.0, obsidian.clone()));
        objects.push(Cube::new(Vector3::new(0.0, y as f32, -2.0), 1.0, obsidian.clone()));
    }
    for z in -1..=0 {
        objects.push(Cube::new(Vector3::new(0.0, 0.0, z as f32), 1.0, obsidian.clone()));
        objects.push(Cube::new(Vector3::new(0.0, 4.0, z as f32), 1.0, obsidian.clone()));
    }
    for x_int in 1..=5 {
        for z_int in -4..=1 {
            let material = if (2..=4).contains(&x_int) && (-1..=0).contains(&z_int) { grass.clone() } else { dirt.clone() };
            objects.push(Cube::new(Vector3::new(x_int as f32, 0.0, z_int as f32), 1.0, material));
        }
    }
    objects.push(Cube::new(Vector3::new(3.0, 0.0, -2.0), 1.0, wood_planks.clone()));
    objects.push(Cube::new(Vector3::new(3.0, 0.0, -3.0), 1.0, wood_planks.clone()));
    for x in 2..=4 {
        for z in -4..=-2 { objects.push(Cube::new(Vector3::new(x as f32, 3.0, z as f32), 1.0, wood_planks.clone())); }
        for y in 1..=2 { objects.push(Cube::new(Vector3::new(x as f32, y as f32, -4.0), 1.0, wood_planks.clone())); }
    }
    for y in 1..=2 {
        objects.push(Cube::new(Vector3::new(2.0, y as f32, -2.0), 1.0, wood_planks.clone()));
        objects.push(Cube::new(Vector3::new(4.0, y as f32, -2.0), 1.0, wood_planks.clone()));
    }
    objects.push(Cube::new(Vector3::new(2.0, 1.0, -3.0), 1.0, wood_planks.clone()));
    objects.push(Cube::new(Vector3::new(2.0, 2.0, -3.0), 1.0, glass.clone()));
    objects.push(Cube::new(Vector3::new(4.0, 1.0, -3.0), 1.0, wood_planks.clone()));
    objects.push(Cube::new(Vector3::new(4.0, 2.0, -3.0), 1.0, glass.clone()));
    for y in 1..=3 { objects.push(Cube::new(Vector3::new(5.0, y as f32, 1.0), 1.0, oak.clone())); }
    objects.push(Cube::new(Vector3::new(5.0, 5.0, 1.0), 1.0, leaves.clone()));
    objects.push(Cube::new(Vector3::new(5.0, 4.0, 1.0), 1.0, leaves.clone()));
    objects.push(Cube::new(Vector3::new(6.0, 4.0, 1.0), 1.0, leaves.clone()));
    objects.push(Cube::new(Vector3::new(4.0, 4.0, 1.0), 1.0, leaves.clone()));
    objects.push(Cube::new(Vector3::new(5.0, 4.0, 2.0), 1.0, leaves.clone()));
    objects.push(Cube::new(Vector3::new(5.0, 4.0, -1.0), 1.0, leaves.clone()));

    // Pre-filtrar los cubos que emiten luz
    let emissive_cubes: Vec<&Cube> = objects.iter().filter(|c| c.material.emission.dot(c.material.emission) > 0.0).collect();
    
    let mut camera = Camera::new(Vector3::new(0.0, 8.0, 10.0), Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
    let rotation_speed = PI / 100.0;
    let zoom_speed = 0.1;
    let vertical_speed = 0.1;
    let light = Light::new(Vector3::new(0.5, 5.0, 5.0), Vector3::new(1.0, 1.0, 1.0), 1.2);

    let mut texture = window.load_texture_from_image(&raylib_thread, &Image::gen_image_color(window_width, window_height, Color::BLACK)).expect("No se pudo cargar la textura");

    while !window.window_should_close() {
        let start_time = std::time::Instant::now();
        
        if window.is_key_down(KeyboardKey::KEY_LEFT) { camera.orbit(rotation_speed, 0.0); }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) { camera.orbit(-rotation_speed, 0.0); }
        if window.is_key_down(KeyboardKey::KEY_UP) { camera.orbit(0.0, -rotation_speed); }
        if window.is_key_down(KeyboardKey::KEY_DOWN) { camera.orbit(0.0, rotation_speed); }
        if window.is_key_down(KeyboardKey::KEY_D) { camera.zoom(zoom_speed); }
        if window.is_key_down(KeyboardKey::KEY_A) { camera.zoom(-zoom_speed); }
        if window.is_key_down(KeyboardKey::KEY_W) { camera.eye.y += vertical_speed; camera.center.y += vertical_speed; camera.update_basis(); }
        if window.is_key_down(KeyboardKey::KEY_S) { camera.eye.y -= vertical_speed; camera.center.y -= vertical_speed; camera.update_basis(); }
        
        let pixel_data = render(window_width, window_height, &objects, &camera, &light, &emissive_cubes, &texture_manager);
        
        let pixel_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(pixel_data.as_ptr() as *const u8, pixel_data.len() * size_of::<Color>())
        };

        let _ = texture.update_texture(pixel_bytes);
        
        {
            let mut d = window.begin_drawing(&raylib_thread);
            d.clear_background(Color::BLACK);
            d.draw_texture(&texture, 0, 0, Color::WHITE);
            
            let elapsed = start_time.elapsed().as_millis() as f32 / 1000.0;
            let fps = if elapsed > 0.0 { (1.0 / elapsed).round() as i32 } else { 0 };
            d.draw_text(&format!("FPS: {}", fps), 10, 10, 20, Color::WHITE);
        }
    }
}