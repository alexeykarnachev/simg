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
    let postfx_frag_src = include_str!("./assets/postfx.frag");
    let font_bytes = include_bytes!(
        "../assets/fonts/share_tech_mono/ShareTechMono-Regular.ttf"
    )
    .as_slice();

    let width = 800.0;
    let height = 600.0;
    let sdl2 = sdl2::init().unwrap();
    let mut input = Input::new(&sdl2);
    let mut renderer =
        Renderer::new(&sdl2, "breakout", width as u32, height as u32);

    let mut postfx_program =
        renderer.load_screen_rect_program(postfx_frag_src);

    let font_size = 40;
    let glyph_atlas = GlyphAtlas::new(font_bytes, font_size);
    let glyph_tex = renderer.load_texture_from_glyph_atlas(&glyph_atlas);

    let mut camera = Camera2D::new(Vector2::new(0.0, 0.0));
    camera.zoom = 1.0;

    let mut text = String::with_capacity(1024);

    let mut update = move || {
        input.update();

        renderer.start_new_batch(Proj2D(camera), Some(glyph_tex));
        renderer.end_drawing(PRUSSIAN_BLUE, None);

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
