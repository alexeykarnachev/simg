#![allow(unused_variables)]

use nalgebra::Vector2;
use sdl2::event::Event;
use simg::renderer::color::*;
use simg::renderer::*;

pub fn main() {
    let width = 800.0;
    let height = 600.0;
    let sdl2 = sdl2::init().unwrap();
    let event_pump = Box::leak(Box::new(sdl2.event_pump().unwrap()));
    let mut renderer =
        Renderer::new(&sdl2, "triangle", width as u32, height as u32);

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        renderer.begin_drawing();
        renderer.clear_color(WHITE);
        renderer.draw_triangle(
            Vector2::new(width / 2.0, height),
            Vector2::new(0.0, height / 2.0),
            Vector2::new(width, height / 2.0),
            GREEN,
        );
        renderer.draw_triangle(
            Vector2::new(width / 2.0, 0.0),
            Vector2::new(width, height / 2.0),
            Vector2::new(0.0, height / 2.0),
            RED,
        );
        renderer.end_drawing();
        renderer.swap_window();
    }
}
