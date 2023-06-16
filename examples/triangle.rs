#![allow(unused_variables)]
#![allow(unused_mut)]

use nalgebra::Vector2;
use sdl2::keyboard::Scancode;
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
    let font_size = 40;
    let font = Font::new(font_bytes, font_size);
    let font_tex = renderer.load_texture_from_font(&font);

    let mut camera = Camera2D::new(Vector2::new(0.0, 0.0));
    camera.zoom = 1.0;
    // camera.position.x += width * 0.5;

    let mut text = String::with_capacity(1024);

    let mut update = move || {
        input.update();
        text.push_str(&input.text_input);

        if input.scancodes.is_just_repeated(Scancode::Backspace) {
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

        let cursor_max_x = width / 2.1;
        let cursor_min_x = -width / 2.0;
        let mut cursor = Vector2::new(cursor_min_x, height / 2.3);
        for (_, symbol) in text.char_indices() {
            let glyph = font.advance_glyph(&mut cursor, symbol);
            if cursor.x > cursor_max_x {
                cursor.x = cursor_min_x;
                cursor.y -= font_size as f32;
            }
            renderer
                .draw_glyph(glyph, Some(Color::new(1.0, 1.0, 1.0, 0.0)));
        }
        renderer.draw_rect(
            font.get_cursor_rect(cursor),
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
