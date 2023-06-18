#![allow(unused_variables)]
#![allow(unused_mut)]

use nalgebra::Vector2;
use resources::POSTFX_FRAG_SRC;
use simg::color::*;
use simg::input::*;
use simg::program::Program;
use simg::program::ProgramArg::ColorArg;
use simg::renderer::Projection::*;
use simg::renderer::*;
use simg::shapes::Rectangle;
use std::ops::Add;
use std::time::Instant;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

const FIELD_ASPECT: f32 = 0.75;
const FIELD_HEIGHT: f32 = 580.0;
const FIELD_WIDTH: f32 = FIELD_HEIGHT * FIELD_ASPECT;

const FIELD_FRAME_THICKNESS: f32 =
    (WINDOW_HEIGHT - FIELD_HEIGHT - 5.0) / 2.0;
const FIELD_FRAME_WIDTH: f32 = FIELD_WIDTH + FIELD_FRAME_THICKNESS * 2.0;
const FIELD_FRAME_HEIGHT: f32 = FIELD_HEIGHT + FIELD_FRAME_THICKNESS * 2.0;

const N_CELLS_X: usize = 14;
const N_CELLS_Y: usize = 54;
const CELL_WIDTH: f32 = FIELD_WIDTH / N_CELLS_X as f32;
const CELL_HEIGHT: f32 = FIELD_HEIGHT / N_CELLS_Y as f32;

const BLOCK_FRAME_THICKNESS: f32 = 1.0;

const PLAYER_WIDTH: f32 = CELL_WIDTH * 2.0;
const PLAYER_HEIGHT: f32 = CELL_HEIGHT;
const PLAYER_ELEVATION: f32 = CELL_HEIGHT * 6.0;

mod resources {
    pub const POSTFX_FRAG_SRC: &str = include_str!("./assets/postfx.frag");
    pub const FONT: &[u8] = include_bytes!(
        "../assets/fonts/share_tech_mono/ShareTechMono-Regular.ttf"
    );
}

struct Block {
    rect: Rectangle,
    is_alive: bool,
    score: u32,

    draw_rect: Rectangle,
    color: Color,
}

impl Block {
    pub fn new(pos: Vector2<f32>, score: u32, color: Color) -> Self {
        let size = Vector2::new(CELL_WIDTH, CELL_HEIGHT);
        let rect = Rectangle::from_bot_center(pos, size);
        let draw_rect = Rectangle::from_center(
            rect.get_center(),
            rect.get_size()
                - Vector2::new(
                    BLOCK_FRAME_THICKNESS * 2.0,
                    BLOCK_FRAME_THICKNESS * 2.0,
                ),
        );

        Self {
            rect,
            is_alive: true,
            score,

            draw_rect,
            color,
        }
    }
}

struct Player {
    pos: Vector2<f32>,
    rect: Rectangle,
}

impl Player {
    pub fn new(pos: Vector2<f32>) -> Self {
        let size = Vector2::new(PLAYER_WIDTH, PLAYER_HEIGHT);
        Self {
            pos,
            rect: Rectangle::from_bot_center(Vector2::zeros(), size),
        }
    }

    pub fn get_rect(&self) -> Rectangle {
        self.rect.translate(&self.pos)
    }
}

struct Game {
    dt: f32,
    prev_upd_time: Instant,
    should_quit: bool,

    input: Input,
    renderer: Renderer,
    postfx: Program,

    frame: Rectangle,
    field: Rectangle,

    blocks: Vec<Block>,
    player: Player,
}

impl Game {
    pub fn new() -> Self {
        let sdl2 = sdl2::init().unwrap();
        let input = Input::new(&sdl2);
        let mut renderer = Renderer::new(
            &sdl2,
            "Breakout",
            WINDOW_WIDTH as u32,
            WINDOW_HEIGHT as u32,
        );
        let postfx = renderer.load_screen_rect_program(POSTFX_FRAG_SRC);
        let field = Rectangle::from_center(
            Vector2::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0),
            Vector2::new(FIELD_WIDTH, FIELD_HEIGHT),
        );
        let frame = Rectangle::from_center(
            field.get_center(),
            Vector2::new(FIELD_FRAME_WIDTH, FIELD_FRAME_HEIGHT),
        );
        let player = Player::new(
            field
                .get_bot_center()
                .add(Vector2::new(0.0, PLAYER_ELEVATION)),
        );

        let mut blocks = Vec::with_capacity(112);
        for i in 0..112 {
            let i_row = i / N_CELLS_X;
            let i_col = i % N_CELLS_X;

            let mut y = field.get_bot_left().y
                + (N_CELLS_Y - 16) as f32 * CELL_HEIGHT;
            let mut x = field.get_bot_left().x + CELL_WIDTH / 2.0;

            y += i_row as f32 * CELL_HEIGHT;
            x += i_col as f32 * CELL_WIDTH;

            let (score, color) = match i_row / 2 {
                0 => (1, YELLOW),
                1 => (3, GREEN),
                2 => (5, ORANGE),
                3 => (7, RED),
                _ => {
                    panic!("Unexpected block index")
                }
            };

            let block = Block::new(Vector2::new(x, y), score, color);
            blocks.push(block);
        }

        Self {
            dt: 0.0,
            prev_upd_time: Instant::now(),
            should_quit: false,
            input,
            renderer,
            postfx,
            frame,
            field,
            blocks,
            player,
        }
    }

    pub fn update(&mut self) {
        self.update_input();
        self.update_world();
        self.update_renderer();
    }

    fn update_input(&mut self) {
        self.input.update();
        self.should_quit |= self.input.should_quit;
    }

    fn update_world(&mut self) {
        self.dt = self.prev_upd_time.elapsed().as_nanos() as f32 / 1.0e9;
        self.prev_upd_time = Instant::now();
    }

    fn update_renderer(&mut self) {
        self.renderer.start_new_batch(ProjScreen, None);

        self.renderer.draw_rect(self.frame, None, Some(WHITE));
        self.renderer.draw_rect(self.field, None, Some(BLACK));
        self.renderer
            .draw_rect(self.player.get_rect(), None, Some(WHITE));

        for block in self.blocks.iter() {
            self.renderer.draw_rect(
                block.draw_rect,
                None,
                Some(block.color),
            );
        }

        self.postfx
            .set_arg("u_color", ColorArg(Color::new(0.0, 0.0, 0.0, 1.0)));
        self.renderer.end_drawing(BLACK, Some(&self.postfx));

        self.renderer.swap_window();
    }
}

pub fn main() {
    let mut game = Game::new();

    let mut update = move || {
        game.update();

        return !game.should_quit;
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
