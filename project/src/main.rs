// main.rs
#![allow(unused_imports)]
#![allow(dead_code)]

mod framebuffer;
mod maze;
mod player;
mod caster;

use raylib::prelude::*;
use std::thread;
use std::time::Duration;
use player::{Player, process_events};
use framebuffer::Framebuffer;
use maze::{Maze,load_maze};
use caster::cast_ray;
use std::f32::consts::PI;


fn draw_cell(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
) {
    if cell == ' ' {
        return;
    }

    framebuffer.set_current_color(Color::RED);

    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            framebuffer.set_pixel(x as i32, y as i32);
        }
    }
}

pub fn render_maze(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    block_size: usize,
    player: &Player,
) {
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = col_index * block_size;
            let yo = row_index * block_size;
            
            draw_cell(framebuffer, xo, yo, block_size, cell);
        }
    }
    //draw player
    framebuffer.set_current_color(Color::WHITE);
    let px = player.pos.x as i32;
    let py = player.pos.y as i32;
    framebuffer.set_pixel(px, py);

    cast_ray(framebuffer, maze, player, block_size);
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
    let block_size = 100;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raycaster Example")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(
        window_width as i32, 
        window_height as i32, 
        Color::new(50, 50, 100, 255)
    );

    // Load the maze once before the loop
    let maze = load_maze("maze.txt");
    let mut player = Player{pos: Vector2::new(150.0,150.0), a: PI/2.0,};

    while !window.window_should_close() {
        // 1. clear framebuffer
        framebuffer.clear();

        // 1.1 process events y chequea la colisión
        // Le pasamos el laberinto y el tamaño del bloque.
        let game_over = process_events(&window, &mut player, &maze, block_size);
        
        // Si la función devuelve 'true', rompemos el bucle.
        if game_over {
            break; // Salir del bucle para cerrar la ventana.
        }

        // 2. draw the maze, passing the maze and block size
        render_maze(&mut framebuffer, &maze, block_size, &player);

        // 3. swap buffers
        framebuffer.swap_buffers(&mut window, &raylib_thread);

        thread::sleep(Duration::from_millis(16));
    }
}
