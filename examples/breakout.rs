#![allow(unused_variables)]
#![allow(unused_mut)]

use core::f32::consts::PI;
use nalgebra::Vector2;
use rand::Rng;
use sdl2::keyboard::Keycode;
use simg::color::*;
use simg::emscripten::*;
use simg::geometry::*;
use simg::glyph_atlas::*;
use simg::input::*;
use simg::program::Program;
use simg::program::ProgramArg::ColorArg;
use simg::renderer::Projection::*;
use simg::renderer::*;
use simg::shapes::*;
use std::ops::Add;
use std::time::Instant;

const GAME_DT: f32 = 0.005;

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

const PADDLE_WIDTH: f32 = CELL_WIDTH * 2.0;
const PADDLE_HEIGHT: f32 = CELL_HEIGHT;
const PADDLE_ELEVATION: f32 = CELL_HEIGHT * 6.0;
const PADDLE_SPEED: f32 = 600.0;

const BALL_RADIUS: f32 = 8.0;
const INIT_BALL_SPEED: f32 = 500.0;

pub const POSTFX_FRAG_SRC: &str = include_str!("./assets/postfx.frag");
pub const FONT: &[u8] = include_bytes!(
    "../assets/fonts/share_tech_mono/ShareTechMono-Regular.ttf"
);

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

#[derive(PartialEq)]
enum State {
    NotStarted,
    Started,
    Finished,
}

struct Game {
    prev_ticks: u32,
    should_quit: bool,
    timer: sdl2::TimerSubsystem,

    input: Input,
    renderer: Renderer,
    glyph_atlas: GlyphAtlas,
    glyph_tex: u32,
    // postfx: Program,
    frame: Rectangle,
    field: Rectangle,

    blocks: Vec<Block>,
    paddle: Rectangle,

    ball: Circle,
    ball_speed: f32,
    ball_velocity: Vector2<f32>,

    state: State,
    scores: u32,
}

impl Game {
    pub fn new() -> Self {
        let sdl2 = sdl2::init().unwrap();
        let timer = sdl2.timer().unwrap();
        let input = Input::new(&sdl2);
        let mut renderer = Renderer::new(
            &sdl2,
            "Breakout",
            WINDOW_WIDTH as u32,
            WINDOW_HEIGHT as u32,
        );

        let glyph_atlas = GlyphAtlas::new(FONT, 48);
        let glyph_tex =
            renderer.load_texture_from_glyph_atlas(&glyph_atlas);

        // let postfx = renderer.load_screen_rect_program(POSTFX_FRAG_SRC);
        let field = Rectangle::from_center(
            Vector2::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0),
            Vector2::new(FIELD_WIDTH, FIELD_HEIGHT),
        );
        let frame = Rectangle::from_center(
            field.get_center(),
            Vector2::new(FIELD_FRAME_WIDTH, FIELD_FRAME_HEIGHT),
        );
        let paddle = Rectangle::from_bot_center(
            field
                .get_bot_center()
                .add(Vector2::new(0.0, PADDLE_ELEVATION)),
            Vector2::new(PADDLE_WIDTH, PADDLE_HEIGHT),
        );

        let ball = Circle::from_bot(field.get_center(), BALL_RADIUS);

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
            prev_ticks: timer.ticks(),
            should_quit: false,
            timer,
            input,
            renderer,
            glyph_atlas,
            glyph_tex,
            // postfx,
            frame,
            field,
            blocks,
            paddle,
            ball,
            ball_speed: INIT_BALL_SPEED,
            ball_velocity: Vector2::zeros(),
            state: State::NotStarted,
            scores: 0,
        }
    }

    pub fn update(&mut self) {
        self.update_input();

        let mut dt =
            (self.timer.ticks() - self.prev_ticks) as f32 / 1000.0;
        while dt > 0.0 {
            self.update_game(dt.min(GAME_DT));
            dt -= GAME_DT;
        }
        self.prev_ticks = self.timer.ticks();

        self.update_renderer();
    }

    fn update_input(&mut self) {
        self.input.update();
        self.should_quit |= self.input.should_quit;
    }

    fn update_game(&mut self, dt: f32) {
        use Keycode::*;
        use State::*;

        let field_left_x = self.field.get_left_x();
        let field_right_x = self.field.get_right_x();
        let field_bot_y = self.field.get_bot_y();
        let field_top_y = self.field.get_top_y();

        if self.state != Finished {
            if self.input.keycodes.is_pressed(Right) {
                self.paddle.translate_x_assign(PADDLE_SPEED * dt);
            } else if self.input.keycodes.is_pressed(Left) {
                self.paddle.translate_x_assign(-PADDLE_SPEED * dt);
            }

            let paddle_left_x = self.paddle.get_left_x();
            let paddle_right_x = self.paddle.get_right_x();
            if paddle_left_x < field_left_x {
                self.paddle
                    .translate_x_assign(field_left_x - paddle_left_x);
            } else if paddle_right_x > field_right_x {
                self.paddle
                    .translate_x_assign(field_right_x - paddle_right_x);
            }
        }

        let paddle_left_x = self.paddle.get_left_x();
        let paddle_right_x = self.paddle.get_right_x();
        let paddle_bot_y = self.paddle.get_bot_y();
        let paddle_top_y = self.paddle.get_top_y();

        if self.state == NotStarted
            && self.input.keycodes.is_just_pressed(Space)
        {
            self.state = Started;
            let mut rng = rand::thread_rng();
            let angle = -PI / 2.0 + rng.gen_range(-PI / 5.0..=PI / 5.0);
            self.ball_velocity =
                Vector2::new(angle.cos(), angle.sin()) * self.ball_speed;
        }

        if self.state != NotStarted {
            let step = self.ball_velocity * dt;
            self.ball.center += step;

            let ball_left_x = self.ball.get_left_x();
            let ball_right_x = self.ball.get_right_x();
            let ball_top_y = self.ball.get_top_y();
            let ball_bot_y = self.ball.get_bot_y();
            if let Some(mtv) =
                get_circle_rectangle_mtv(&self.ball, &self.paddle)
            {
                self.ball.center += mtv;
                let k = (self.ball.center.x - paddle_left_x)
                    / (paddle_right_x - paddle_left_x);
                let angle = if self.ball.center.y >= paddle_bot_y {
                    PI * (0.75 - 0.5 * k)
                } else {
                    PI * (0.5 * k - 0.75)
                };
                self.ball_velocity =
                    Vector2::new(angle.cos(), angle.sin())
                        * self.ball_speed;
            } else if ball_left_x < field_left_x {
                self.ball_velocity = reflect(&self.ball_velocity, &RIGHT);
                self.ball.center.x = field_left_x + self.ball.radius;
            } else if ball_right_x > field_right_x {
                self.ball_velocity = reflect(&self.ball_velocity, &LEFT);
                self.ball.center.x = field_right_x - self.ball.radius;
            } else if ball_bot_y < field_bot_y {
                self.ball_velocity = reflect(&self.ball_velocity, &UP);
                self.ball.center.y = field_bot_y + self.ball.radius;
            } else if ball_top_y > field_top_y {
                self.ball_velocity = reflect(&self.ball_velocity, &DOWN);
                self.ball.center.y = field_top_y - self.ball.radius;
            }

            self.ball_velocity =
                self.ball_velocity.normalize() * self.ball_speed;
        }

        if self.state == Started {
            for block in self.blocks.iter_mut().filter(|b| b.is_alive) {
                if let Some(mtv) =
                    get_circle_rectangle_mtv(&self.ball, &block.rect)
                {
                    self.ball.center += mtv;
                    self.ball_velocity =
                        reflect(&self.ball_velocity, &mtv);
                    self.scores += block.score;
                    block.is_alive = false;
                    break;
                }
            }
        }
    }

    fn update_renderer(&mut self) {
        self.renderer
            .start_new_batch(ProjScreen, Some(self.glyph_tex));

        self.renderer.draw_rect(self.frame, None, Some(WHITE));
        self.renderer.draw_rect(self.field, None, Some(BLACK));
        self.renderer.draw_rect(self.paddle, None, Some(WHITE));
        self.renderer.draw_circle(self.ball, None, Some(WHITE));

        let mut scores_pos = self.field.get_top_right();
        scores_pos.x -= 90.0;
        scores_pos.y -= 50.0;
        let scores = format!("{:03}", self.scores);
        for glyph in self.glyph_atlas.iter_text_glyphs(scores_pos, &scores)
        {
            self.renderer.draw_glyph(glyph, Some(WHITE.with_alpha(0.0)));
        }

        for block in self.blocks.iter().filter(|b| b.is_alive) {
            self.renderer.draw_rect(
                block.draw_rect,
                None,
                Some(block.color),
            );
        }

        // self.postfx
        //     .set_arg("u_color", ColorArg(Color::new(0.0, 0.0, 0.0, 1.0)));
        // self.renderer.end_drawing(BLACK, Some(&self.postfx));
        self.renderer.end_drawing(BLACK, None);

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
        set_main_loop_callback(move || {
            update();
        });
    }
}
