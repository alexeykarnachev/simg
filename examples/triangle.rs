#![allow(unused_variables)]

use simg::renderer::*;
use sdl2::event::Event;


pub fn main() {
    let sdl2 = sdl2::init().unwrap();
    let event_pump = Box::leak(Box::new(sdl2.event_pump().unwrap()));
    let renderer = Renderer::new(&sdl2, "triangle", 800, 600);

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        renderer.draw_primitive();
    }
}
