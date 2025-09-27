// src/main.rs
#![allow(unused_imports)]
#![allow(dead_code)]
use raylib::prelude::*;
use std::f32::consts::PI;
// Importamos los traits de Rayon para la paralelización
use rayon::prelude::*;
use std::mem::size_of;

mod framebuffer;
mod ray_intersect;
mod cube; // Cambiado de sphere a cube
mod camera;
mod material;
mod light;
mod snell;
mod textures;
use framebuffer::Framebuffer;
use ray_intersect::{RayIntersect, Intersect};
use cube::Cube; // Cambiado de Sphere a Cube
use camera::Camera;
use material::{Material, vector3_to_color};
use light::Light;
use snell::{reflect, refract};
use textures::TextureManager;

fn procedural_sky(dir: Vector3) -> Vector3 {
    let d = dir.normalized();
    let t = (d.y + 1.0) * 0.5;
    let green = Vector3::new(0.1, 0.6, 0.2);
    let white = Vector3::new(1.0, 1.0, 1.0);
    let blue = Vector3::new(0.3, 0.5, 1.0);
    if t < 0.54 {
        let k = t / 0.55;
        green * (1.0 - k) + white * k
    } else if t < 0.55 {
        white
    } else if t < 0.8 {
        let k = (t - 0.55) / (0.25);
        white * (1.0 - k) + blue * k
    } else {
        blue
    }
}
fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Cube], // Cambiado de Sphere a Cube
) -> f32 {
    let light_direction = (light.position - intersect.point).normalized();
    let shadow_ray_origin = intersect.point;
    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_direction);
        if shadow_intersect.is_intersecting {
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
    objects: &[Cube], // Cambiado de Sphere a Cube
    light: &Light,
    depth: u32,
    texture_manager: &TextureManager,
) -> Vector3 {
    if depth > 3 {
        return procedural_sky(*ray_direction);
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
        return procedural_sky(*ray_direction);
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
    let mut reflection_color = procedural_sky(*ray_direction);
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
    let color = diffuse * intersect.material.albedo[0] + specular * intersect.material.albedo[1] + reflection_color * reflectivity + refraction_color * transparency;
    color
}

/// **FUNCIÓN OPTIMIZADA**
/// Renderiza la escena en paralelo usando múltiples hilos.
/// Devuelve un vector de colores de píxeles que se pueden cargar en el framebuffer.
pub fn render(
    width: i32,
    height: i32,
    objects: &[Cube],
    camera: &Camera,
    light: &Light,
    texture_manager: &TextureManager,
) -> Vec<Color> {
    let aspect_ratio = width as f32 / height as f32;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();
    // Genera un iterador paralelo para las filas de píxeles (y).
    // flat_map crea un nuevo iterador paralelo para cada píxel (x, y) en la pantalla.
    (0..height)
        .into_par_iter()
        .flat_map(|y| (0..width).into_par_iter().map(move |x| (x, y)))
        .map(|(x, y)| {
            // El cálculo para cada píxel es el mismo que antes.
            let screen_x = (2.0 * x as f32) / width as f32 - 1.0;
            let screen_y = -(2.0 * y as f32) / height as f32 + 1.0;
            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;
            let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();
            let rotated_direction = camera.basis_change(&ray_direction);
            let pixel_color_vec = cast_ray(
                &camera.eye,
                &rotated_direction,
                objects,
                light,
                0,
                texture_manager,
            );
            vector3_to_color(pixel_color_vec)
        })
        .collect() // Recolecta los resultados de todos los hilos en un solo Vec<Color>.
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raytracer Class - Cubes (Optimized)")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    // Hacemos que la ventana intente correr a 60 FPS, aunque el renderizado puede tardar más.
    window.set_target_fps(60);
    let mut texture_manager = TextureManager::new();
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/bricks_normal.png");
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
    
    let framebuffer = Framebuffer::new(window_width as i32, window_height as i32);
    
    let glass = Material {
        diffuse: Vector3::new(1.0, 1.0, 1.0), albedo: [0.0,5.0], specular: 125.0,
        reflectivity: 0.1, transparency: 0.9, refractive_index: 1.5,
        texture: Some("assets/glass.png".to_string()), normal_map_id: None,
    };
    let dirt = Material {
        diffuse: Vector3::new(0.4, 0.26, 0.13), albedo: [0.8, 0.2], specular: 1.0,
        reflectivity: 0.0, transparency: 0.0, refractive_index: 1.0,
        texture: Some("assets/dirt.png".to_string()), normal_map_id: None,
    };
    let grass = Material {
        diffuse: Vector3::new(0.2, 0.6, 0.2), albedo: [0.7, 0.3], specular: 2.0,
        reflectivity: 0.05, transparency: 0.0, refractive_index: 1.0,
        texture: Some("assets/grass.png".to_string()), normal_map_id: None,
    };
    let leaves = Material {
        diffuse: Vector3::new(0.1, 0.5, 0.1), albedo: [0.6, 0.4], specular: 3.0,
        reflectivity: 0.02, transparency: 0.3, refractive_index: 1.2,
        texture: Some("assets/leaves.png".to_string()), normal_map_id: None,
    };
    let magma = Material {
        diffuse: Vector3::new(1.0, 0.3, 0.0), albedo: [0.9, 0.1], specular: 50.0,
        reflectivity: 0.2, transparency: 0.0, refractive_index: 1.0,
        texture: Some("assets/magma.png".to_string()), normal_map_id: None,
    };
    let oak = Material {
        diffuse: Vector3::new(0.6, 0.4, 0.2), albedo: [0.8, 0.2], specular: 5.0,
        reflectivity: 0.0, transparency: 0.0, refractive_index: 1.0,
        texture: Some("assets/oak.png".to_string()), normal_map_id: None,
    };
    let wood_planks = Material {
        diffuse: Vector3::new(0.6, 0.4, 0.2), albedo: [0.8, 0.2], specular: 5.0,
        reflectivity: 0.0, transparency: 0.0, refractive_index: 1.0,
        texture: Some("assets/wood_planks.png".to_string()), normal_map_id: None,
    };
    let stone = Material {
        diffuse: Vector3::new(0.5, 0.5, 0.5), albedo: [0.7, 0.3], specular: 8.0,
        reflectivity: 0.05, transparency: 0.0, refractive_index: 1.0,
        texture: Some("assets/stone.png".to_string()), normal_map_id: None,
    };
    let diamond_ore = Material {
        diffuse: Vector3::new(0.4, 0.4, 0.4), albedo: [0.5, 0.5], specular: 80.0,
        reflectivity: 0.3, transparency: 0.0, refractive_index: 1.0,
        texture: Some("assets/diamond_ore.png".to_string()), normal_map_id: None,
    };
    let obsidian = Material {
        diffuse: Vector3::new(0.1, 0.05, 0.15), albedo: [0.8, 0.2], specular: 10.0,
        reflectivity: 0.1, transparency: 0.0, refractive_index: 1.0,
        texture: Some("assets/obsidian.png".to_string()), normal_map_id: None,
    };
    let water = Material {
        diffuse: Vector3::new(0.0, 0.3, 0.9), albedo: [0.6, 0.4], specular: 30.0,
        reflectivity: 0.15, transparency: 0.6, refractive_index: 1.2,
        texture: Some("assets/water.png".to_string()), normal_map_id: None,
    };

    let objects = [
        Cube::new(Vector3::new(-5.0, 0.0, 0.0), 1.0, diamond_ore.clone()),
        Cube::new(Vector3::new(-4.0, 0.0, 0.0), 1.0, dirt.clone()),
        Cube::new(Vector3::new(-3.0, 0.0, 0.0), 1.0, glass.clone()),
        Cube::new(Vector3::new(-2.0, 0.0, 0.0), 1.0, grass.clone()),
        Cube::new(Vector3::new(-1.0, 0.0, 0.0), 1.0, leaves.clone()),
        Cube::new(Vector3::new(0.0, 0.0, 0.0),  1.0, oak.clone()),
        Cube::new(Vector3::new(1.0, 0.0, 0.0), 1.0, obsidian.clone()),
        Cube::new(Vector3::new(2.0, 0.0, 0.0), 1.0, stone.clone()),
        Cube::new(Vector3::new(3.0, 0.0, 0.0), 1.0, water.clone()),
        Cube::new(Vector3::new(4.0, 0.0, 0.0), 1.0, wood_planks.clone()),
        Cube::new(Vector3::new(5.0, 0.0, 0.0), 1.0, magma.clone()),
    ];
    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 5.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    );
    let rotation_speed = PI / 100.0;
    let zoom_speed = 0.1;
    let light = Light::new(
        Vector3::new(5.0, 5.0, 5.0),
        Vector3::new(1.0, 1.0, 1.0),
        1.5,
    );

    // **OPTIMIZACIÓN:** Creamos la textura una sola vez, fuera del bucle principal.
    let mut texture = window
        .load_texture_from_image(&raylib_thread, &framebuffer.color_buffer)
        .expect("No se pudo cargar la textura desde el framebuffer");

    while !window.window_should_close() {
        if window.is_key_down(KeyboardKey::KEY_LEFT) { camera.orbit(rotation_speed, 0.0); }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) { camera.orbit(-rotation_speed, 0.0); }
        if window.is_key_down(KeyboardKey::KEY_UP) { camera.orbit(0.0, -rotation_speed); }
        if window.is_key_down(KeyboardKey::KEY_DOWN) { camera.orbit(0.0, rotation_speed); }
        if window.is_key_down(KeyboardKey::KEY_W) { camera.zoom(zoom_speed); }
        if window.is_key_down(KeyboardKey::KEY_S) { camera.zoom(-zoom_speed); }
        
        // 1. Llama a la función de renderizado en paralelo.
        let pixel_data = render(
            framebuffer.width,
            framebuffer.height,
            &objects,
            &camera,
            &light,
            &texture_manager,
        );
        
        // 2. **OPTIMIZACIÓN:** Convertimos el vector de colores a un slice de bytes.
        // La estructura `Color` de Raylib es `repr(C)`, por lo que esta conversión es segura.
        let pixel_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                pixel_data.as_ptr() as *const u8,
                pixel_data.len() * size_of::<Color>(),
            )
        };

        // 3. **OPTIMIZACIÓN Y CORRECCIÓN:** Actualizamos la textura existente en la GPU con los nuevos datos.
        // El nombre correcto del método es `update_texture`.
        let _ = texture.update_texture(pixel_bytes);
        
        // 4. Inicia el dibujado en pantalla.
        {
            let mut d = window.begin_drawing(&raylib_thread);
            d.clear_background(Color::BLACK);
            // Dibujamos la textura que acabamos de actualizar.
            d.draw_texture(&texture, 0, 0, Color::WHITE);
            
            // 5. Dibuja el contador de FPS.
            d.draw_text(
                &format!("FPS: {}", d.get_fps()),
                10, // Posición X
                10, // Posición Y
                20, // Tamaño de fuente
                Color::BLACK, // Color del texto
            );
        } // El dibujado termina aquí cuando `d` se destruye.
    }
}