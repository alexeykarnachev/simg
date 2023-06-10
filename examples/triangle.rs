#![allow(unused_variables)]

use nalgebra::Vector2;
use sdl2::event::Event;
use simg::renderer::color::*;
use simg::renderer::*;

pub fn main() {
    let sdl2 = sdl2::init().unwrap();
    let event_pump = Box::leak(Box::new(sdl2.event_pump().unwrap()));
    let mut renderer = Renderer::new(&sdl2, "triangle", 800, 600);

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        renderer.clear_color(WHITE);
        renderer.push_triangle(
            Vector2::new(0.0, 0.5),
            Vector2::new(-0.5, 0.0),
            Vector2::new(0.5, 0.0),
            GREEN,
        );
        renderer.push_triangle(
            Vector2::new(0.0, -0.5),
            Vector2::new(0.5, 0.0),
            Vector2::new(-0.5, 0.0),
            RED,
        );
        renderer.draw();
        renderer.swap_window();
    }
}
