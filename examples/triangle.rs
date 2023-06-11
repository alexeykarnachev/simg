#![allow(unused_variables)]

use nalgebra::Vector2;
use simg::input::*;
use simg::renderer::camera::*;
use simg::renderer::color::*;
use simg::renderer::*;

pub fn main() {
    let width = 800.0;
    let height = 600.0;
    let sdl2 = sdl2::init().unwrap();
    let mut input = Input::new(&sdl2);
    let mut renderer =
        Renderer::new(&sdl2, "triangle", width as u32, height as u32);

    let mut camera = Camera2D::new(Vector2::new(0.0, 0.0));
    camera.zoom = 0.5;
    camera.position.x += width * 0.5;

    while !input.should_quit {
        input.update();
        camera.rotation += 0.01;

        renderer.begin_screen_drawing();
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

        renderer.begin_2d_drawing(camera);
        renderer.draw_triangle(
            Vector2::new(0.0, 0.0),
            Vector2::new(width / 2.0, height / 2.0),
            Vector2::new(-width / 2.0, height / 2.0),
            GREEN,
        );
        renderer.end_drawing();

        renderer.swap_window();
    }
}
