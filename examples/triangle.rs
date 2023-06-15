#![allow(unused_variables)]
#![allow(unused_mut)]

use nalgebra::Vector2;
use sdl2::keyboard::{Keycode, Scancode};
use simg::font::*;
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

    let font_bytes = include_bytes!(
        "../assets/fonts/share_tech_mono/ShareTechMono-Regular.ttf"
    )
    .as_slice();
    let font_size = 52;
    let font = Font::new(font_bytes, font_size);
    let font_tex = renderer.load_texture_from_font(&font);

    let mut camera = Camera2D::new(Vector2::new(0.0, 0.0));
    camera.zoom = 0.5;
    camera.position.x += width * 0.5;

    let mut text = String::with_capacity(1024);

    let mut update = move || {
        input.update();
        text.push_str(&input.text_input);

        if input
            .scancodes
            .just_repeated
            .get(&Scancode::Backspace)
            .is_some()
        {
            text.pop();
        }

        if text == "ROTATE!" {
            camera.rotation += 0.001;
        }

        renderer.clear_color(Color::new(0.1, 0.2, 0.3, 1.0));

        renderer.start_new_batch(ProjScreen, None);
        renderer.draw_triangle(
            Triangle::new(
                Vector2::new(width / 2.0, height),
                Vector2::new(0.0, height / 2.0),
                Vector2::new(width, height / 2.0),
            ),
            None,
            Some(BLACK),
        );
        renderer.draw_triangle(
            Triangle::new(
                Vector2::new(width / 2.0, 0.0),
                Vector2::new(width, height / 2.0),
                Vector2::new(0.0, height / 2.0),
            ),
            None,
            Some(GRAY),
        );

        renderer.start_new_batch(Proj2D(camera), Some(tex));
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

        renderer.start_new_batch(Proj2D(camera), Some(font_tex));

        let mut cursor = Vector2::new(width / 2.0, height / 2.0);
        for (_, symbol) in text.char_indices() {
            let glyph = font.advance_glyph(&mut cursor, symbol);
            renderer
                .draw_glyph(glyph, Some(Color::new(1.0, 1.0, 1.0, 0.0)));
        }
        renderer.draw_rect(
            Rectangle::from_bot_left(
                cursor,
                Vector2::new(5.0, font_size as f32),
            ),
            None,
            Some(WHITE),
        );

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
