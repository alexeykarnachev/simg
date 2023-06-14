#![allow(unused_variables)]
#![allow(unused_mut)]

use nalgebra::Vector2;
use simg::input::*;
use simg::renderer::camera::*;
use simg::renderer::color::*;
use simg::renderer::Projection::*;
use simg::renderer::*;
use simg::shapes::*;

pub fn main() {
    let width = 800.0;
    let height = 600.0;
    let sdl2 = sdl2::init().unwrap();
    let mut input = Input::new(&sdl2);
    let mut renderer =
        Renderer::new(&sdl2, "triangle", width as u32, height as u32);

    let image_bytes = include_bytes!("./assets/box.png");
    let tex = renderer.load_texture_from_image_bytes(image_bytes);

    let font_bytes =
        include_bytes!("../assets/fonts/quicksand/Quicksand-Regular.otf")
            .as_slice();
    // let font = renderer.load_font_from_otf_bytes(font_bytes, 32);
    let font = tex;

    let mut camera = Camera2D::new(Vector2::new(0.0, 0.0));
    camera.zoom = 0.5;
    camera.position.x += width * 0.5;

    let mut update = move || {
        println!("---");
        input.update();
        camera.rotation += 0.001;

        renderer.clear_color(WHITE);

        renderer.start_new_batch(ProjScreen, None);
        renderer.draw_triangle(
            Triangle::new(
                Vector2::new(width / 2.0, height),
                Vector2::new(0.0, height / 2.0),
                Vector2::new(width, height / 2.0),
            ),
            None,
            Some(GREEN),
        );
        renderer.draw_triangle(
            Triangle::new(
                Vector2::new(width / 2.0, 0.0),
                Vector2::new(width, height / 2.0),
                Vector2::new(0.0, height / 2.0),
            ),
            None,
            Some(RED),
        );
        renderer.end_drawing();

        renderer.start_new_batch(Proj2D(camera), Some(font));
        renderer.draw_rect(
            Rectangle::from_top_left(
                Vector2::new(0.0, 0.0),
                Vector2::new(width / 2.0, height / 2.0),
            ),
            Some(Rectangle::from_top_left(
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 1.0),
            )),
            None,
        );
        // renderer.draw_text(
        //     "ABC abc ZALOOPA!!!-- ~~ XYI WWW___===CC!!!1232  WW",
        //     Vector2::new(width / 2.0, height / 2.0),
        //     None,
        // );
        renderer.end_drawing();

        renderer.swap_window();

        return !input.should_quit;
    };

    #[cfg(not(target_os = "emscripten"))]
    {
        while update() {}
    }

    #[cfg(target_os = "emscripten")]
    {
        use simg::emscripten::*;
        set_main_loop_callback(move || {
            update();
        });
    }
}
