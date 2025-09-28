// textures.rs
use raylib::prelude::*;
use std::collections::HashMap;

struct CpuTexture {
    width: i32,
    height: i32,
    pixels: Vec<Vector3>, // Normalized RGB values
}

impl CpuTexture {
    fn from_image(image: &Image) -> Self {
        // Safe: Raylib handles pixel format internally
        let colors = image.get_image_data(); // Vec<Color>
        let pixels = colors
            .iter()
            .map(|c| {
                Vector3::new(
                    c.r as f32 / 255.0,
                    c.g as f32 / 255.0,
                    c.b as f32 / 255.0,
                )
            })
            .collect();

        CpuTexture {
            width: image.width,
            height: image.height,
            pixels,
        }
    }
}

pub struct TextureManager {
    cpu_textures: HashMap<String, CpuTexture>,
    textures: HashMap<String, Texture2D>, // Store GPU textures for rendering
    skybox_textures: Option<SkyboxTextures>,
}

#[derive(Clone)]
pub struct SkyboxTextures {
    pub front: String,
    pub back: String,
    pub left: String,
    pub right: String,
    pub top: String,
    pub bottom: String,
}

impl TextureManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_texture(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        path: &str,
    ) {
        if self.textures.contains_key(path) {
            return;
        }

        let image = Image::load_image(path)
            .unwrap_or_else(|_| panic!("Failed to load image {}", path));

        let texture = rl
            .load_texture_from_image(thread, &image)
            .unwrap_or_else(|_| panic!("Failed to load texture {}", path));

        let cpu_texture = CpuTexture::from_image(&image);

        self.cpu_textures.insert(path.to_string(), cpu_texture);
        self.textures.insert(path.to_string(), texture);
    }

    pub fn load_skybox(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        skybox: SkyboxTextures,
    ) {
        self.load_texture(rl, thread, &skybox.front);
        self.load_texture(rl, thread, &skybox.back);
        self.load_texture(rl, thread, &skybox.left);
        self.load_texture(rl, thread, &skybox.right);
        self.load_texture(rl, thread, &skybox.top);
        self.load_texture(rl, thread, &skybox.bottom);
        self.skybox_textures = Some(skybox);
    }

    pub fn sample_skybox(&self, direction: Vector3) -> Vector3 {
        if let Some(ref skybox) = self.skybox_textures {
            // Mapear la dirección a las caras del cubo
            let abs_x = direction.x.abs();
            let abs_y = direction.y.abs();
            let abs_z = direction.z.abs();
            
            let (u, v, texture_path) = if abs_x > abs_y && abs_x > abs_z {
                // X face
                if direction.x > 0.0 {
                    // Right
                    let u = -direction.z / abs_x * 0.5 + 0.5;
                    let v = -direction.y / abs_x * 0.5 + 0.5;
                    (u, v, &skybox.right)
                } else {
                    // Left
                    let u = direction.z / abs_x * 0.5 + 0.5;
                    let v = -direction.y / abs_x * 0.5 + 0.5;
                    (u, v, &skybox.left)
                }
            } else if abs_y > abs_z {
                // Y face
                if direction.y > 0.0 {
                    // Top
                    let u = direction.x / abs_y * 0.5 + 0.5;
                    let v = -direction.z / abs_y * 0.5 + 0.5;
                    (u, v, &skybox.top)
                } else {
                    // Bottom
                    let u = direction.x / abs_y * 0.5 + 0.5;
                    let v = direction.z / abs_y * 0.5 + 0.5;
                    (u, v, &skybox.bottom)
                }
            } else {
                // Z face
                if direction.z > 0.0 {
                    // Front
                    let u = direction.x / abs_z * 0.5 + 0.5;
                    let v = -direction.y / abs_z * 0.5 + 0.5;
                    (u, v, &skybox.front)
                } else {
                    // Back
                    let u = -direction.x / abs_z * 0.5 + 0.5;
                    let v = -direction.y / abs_z * 0.5 + 0.5;
                    (u, v, &skybox.back)
                }
            };
            
            // Asegurar que u y v estén en el rango [0, 1]
            let u = u.max(0.0).min(1.0);
            let v = v.max(0.0).min(1.0);
            
            let cpu_texture = self.cpu_textures.get(texture_path).unwrap();
            let tx = (u * (cpu_texture.width - 1) as f32) as u32;
            let ty = (v * (cpu_texture.height - 1) as f32) as u32;
            
            let index = (ty * cpu_texture.width as u32 + tx) as usize;
            if index < cpu_texture.pixels.len() {
                cpu_texture.pixels[index]
            } else {
                Vector3::one()
            }
        } else {
            // Fallback a sky procedural si no hay skybox
            let d = direction.normalized();
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
    }

    pub fn get_pixel_color(
        &self,
        path: &str,
        tx: u32,
        ty: u32,
    ) -> Vector3 {
        if let Some(cpu_texture) = self.cpu_textures.get(path) {
            let x = tx.min(cpu_texture.width as u32 - 1) as i32;
            let y = ty.min(cpu_texture.height as u32 - 1) as i32;

            if x < 0 || y < 0 || x >= cpu_texture.width || y >= cpu_texture.height {
                return Vector3::one(); // default white
            }

            let index = (y * cpu_texture.width + x) as usize;
            if index < cpu_texture.pixels.len() {
                cpu_texture.pixels[index]
            } else {
                Vector3::one()
            }
        } else {
            Vector3::one()
        }
    }

    pub fn get_texture(
        &self,
        path: &str,
    ) -> Option<&Texture2D> {
        self.textures.get(path)
    }

    pub fn get_normal_from_map(
        &self,
        path: &str,
        tx: u32,
        ty: u32,
    ) -> Option<Vector3> {
        if let Some(cpu_texture) = self.cpu_textures.get(path) {
            let x = tx.min(cpu_texture.width as u32 - 1) as i32;
            let y = ty.min(cpu_texture.height as u32 - 1) as i32;

            if x < 0 || y < 0 || x >= cpu_texture.width || y >= cpu_texture.height {
                return None;
            }

            let index = (y * cpu_texture.width + x) as usize;
            if index < cpu_texture.pixels.len() {
                let color = cpu_texture.pixels[index];
                let normal = Vector3::new(
                    color.x * 2.0 - 1.0,
                    color.y * 2.0 - 1.0,
                    color.z,
                );
                Some(normal.normalized())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        TextureManager {
            cpu_textures: HashMap::new(),
            textures: HashMap::new(),
            skybox_textures: None,
        }
    }
}