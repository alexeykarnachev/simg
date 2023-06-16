#![allow(unused_variables)]
#![allow(unused_mut)]

use nalgebra::Vector2;
use sdl2::keyboard::Scancode;
use simg::camera::*;
use simg::color::*;
use simg::glyph_atlas::*;
use simg::input::*;
use simg::program::ProgramArg::ColorArg;
use simg::renderer::Projection::*;
use simg::renderer::*;
use simg::shapes::*;

pub fn main() {
    let image_bytes = include_bytes!("./assets/box.png");
    let postfx_frag_src = include_str!("./assets/postfx.frag");

    let width = 800.0;
    let height = 600.0;
    let sdl2 = sdl2::init().unwrap();
    let mut input = Input::new(&sdl2);
    let mut renderer =
        Renderer::new(&sdl2, "triangle", width as u32, height as u32);

    let mut postfx_program =
        renderer.load_screen_rect_program(postfx_frag_src);

    let tex = renderer.load_texture_from_image_bytes(image_bytes);

    let font_bytes = include_bytes!(
        "../assets/fonts/share_tech_mono/ShareTechMono-Regular.ttf"
    )
    .as_slice();
    let font_size = 40;
    let glyph_atlas = GlyphAtlas::new(font_bytes, font_size);
    let glyph_tex = renderer.load_texture_from_glyph_atlas(&glyph_atlas);

    let mut camera = Camera2D::new(Vector2::new(0.0, 0.0));
    camera.zoom = 1.0;

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

        renderer.start_new_batch(Proj2D(camera), Some(glyph_tex));

        let cursor_max_x = width / 2.1;
        let cursor_min_x = -width / 2.0;
        let mut cursor = Vector2::new(cursor_min_x, height / 2.3);
        for (_, symbol) in text.char_indices() {
            let glyph = glyph_atlas.get_glyph(&cursor, symbol);
            cursor += glyph.advance;
            if cursor.x > cursor_max_x {
                cursor.x = cursor_min_x;
                cursor.y -= font_size as f32;
            }
            renderer
                .draw_glyph(glyph, Some(Color::new(1.0, 1.0, 1.0, 0.0)));
        }

        let cursor_rect = Rectangle::from_bot_left(
            Vector2::new(cursor.x, cursor.y + glyph_atlas.glyph_descent),
            Vector2::new(
                glyph_atlas.font_size as f32 / 10.0,
                glyph_atlas.glyph_ascent - glyph_atlas.glyph_descent,
            ),
        );
        renderer.draw_rect(cursor_rect, None, Some(WHITE));

        let postfx_color =
            Color::new(0.05 * text.len() as f32, 0.0, 0.0, 1.0);
        postfx_program.set_arg("u_color", ColorArg(postfx_color));
        renderer.end_drawing(PRUSSIAN_BLUE, Some(&postfx_program));

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
