use raylib::prelude::*;
use std::f32::consts::PI;
// Importamos Maze para poder usarlo en la función
use crate::maze::Maze;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
}

// La función ahora acepta el laberinto y el tamaño del bloque para verificar colisiones.
// Devuelve 'true' si hay una colisión (juego terminado), y 'false' en caso contrario.
pub fn process_events(
    window: &RaylibHandle,
    player: &mut Player,
    maze: &Maze,
    block_size: usize,
) -> bool {
    const MOVE_SPEED: f32 = 10.0;
    const ROTATION_SPEED: f32 = PI / 10.0;

    // --- Rotación (no necesita chequeo de colisión) ---
    if window.is_key_pressed(KeyboardKey::KEY_LEFT) {
        player.a -= ROTATION_SPEED;
    }
    if window.is_key_pressed(KeyboardKey::KEY_RIGHT) {
        player.a += ROTATION_SPEED;
    }

    // --- Movimiento (con chequeo de colisión) ---
    let mut next_pos = player.pos;
    let mut moved = false;

    if window.is_key_pressed(KeyboardKey::KEY_UP) {
        next_pos.x += MOVE_SPEED * player.a.cos();
        next_pos.y += MOVE_SPEED * player.a.sin();
        moved = true;
    }
    if window.is_key_pressed(KeyboardKey::KEY_DOWN) {
        next_pos.x -= MOVE_SPEED * player.a.cos();
        next_pos.y -= MOVE_SPEED * player.a.sin();
        moved = true;
    }

    // Si el jugador intentó moverse, verificamos la nueva posición.
    if moved {
        // Convertimos las coordenadas del mundo (flotantes) a coordenadas de la cuadrícula del laberinto (enteros).
        let grid_x = next_pos.x as usize / block_size;
        let grid_y = next_pos.y as usize / block_size;

        // Verificamos si la nueva posición está dentro de un muro.
        // También nos aseguramos de no salirnos de los límites del mapa.
        if grid_y < maze.len() && grid_x < maze[grid_y].len() && maze[grid_y][grid_x] != ' ' {
            // ¡Colisión detectada! Devolvemos 'true' para indicar que el juego debe terminar.
            return true;
        } else {
            // No hay colisión, así que actualizamos la posición del jugador.
            player.pos = next_pos;
        }
    }

    // Si no hubo colisión en este fotograma, devolvemos 'false'.
    false
}