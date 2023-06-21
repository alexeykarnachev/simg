#![allow(unused_variables)]
#![allow(unused_mut)]

use core::f32::consts::PI;
use nalgebra::Vector2;
use rand::Rng;
use sdl2::keyboard::Keycode;
use simg::audio_player::AudioPlayer;
use simg::color::*;
use simg::common::Pivot;
use simg::geometry::*;
use simg::glyph_atlas::*;
use simg::input::*;
use simg::program::Program;
use simg::renderer::Projection::*;
use simg::renderer::*;
use simg::shapes::*;

const MSAA: i32 = 16;

const GAME_DT: f32 = 0.001;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const WINDOW_CENTER: Vector2<f32> =
    Vector2::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0);

const FIELD_ASPECT: f32 = 0.75;
const FIELD_HEIGHT: f32 = 580.0;
const FIELD_WIDTH: f32 = FIELD_HEIGHT * FIELD_ASPECT;
const FIELD_SIZE: Vector2<f32> = Vector2::new(FIELD_WIDTH, FIELD_HEIGHT);

const FIELD_FRAME_THICKNESS: f32 =
    (WINDOW_HEIGHT - FIELD_HEIGHT - 5.0) / 2.0;
const FIELD_FRAME_WIDTH: f32 = FIELD_WIDTH + FIELD_FRAME_THICKNESS * 2.0;
const FIELD_FRAME_HEIGHT: f32 = FIELD_HEIGHT + FIELD_FRAME_THICKNESS * 2.0;
const FIELD_FRAME_SIZE: Vector2<f32> =
    Vector2::new(FIELD_FRAME_WIDTH, FIELD_FRAME_HEIGHT);

const N_CELLS_X: usize = 14;
const N_CELLS_Y: usize = 54;
const CELL_WIDTH: f32 = FIELD_WIDTH / N_CELLS_X as f32;
const CELL_HEIGHT: f32 = FIELD_HEIGHT / N_CELLS_Y as f32;

const BLOCK_FRAME_THICKNESS: f32 = 1.0;
const BLOCK_DEATH_ANIM_TIME: f32 = 0.15;

const PADDLE_WIDTH: f32 = CELL_WIDTH * 2.0;
const PADDLE_HEIGHT: f32 = CELL_HEIGHT;
const PADDLE_SIZE: Vector2<f32> =
    Vector2::new(PADDLE_WIDTH, PADDLE_HEIGHT);
const PADDLE_ELEVATION: f32 = CELL_HEIGHT * 6.0;
const PADDLE_MAX_SPEED: f32 = 400.0;
const PADDLE_ACCELERATION: f32 = 8000.0;

const BALL_RADIUS: f32 = 8.0;
const BALL_START_SPEED: f32 = 320.0;
const BALL_SPEAD_INCREAS_FACTOR: f32 = 0.01;
const BALL_DEATH_ANIM_TIME: f32 = 0.15;

pub const POSTFX_FRAG_SRC: &str =
    include_str!("./assets/breakout/postfx.frag");
pub const FONT: &[u8] = include_bytes!(
    "../assets/fonts/share_tech_mono/ShareTechMono-Regular.ttf"
);
pub const BLOCK_DEATH_SOUND: &[u8] =
    include_bytes!("./assets/breakout/block_death.wav");
pub const BALL_DEATH_SOUND: &[u8] =
    include_bytes!("./assets/breakout/ball_death.wav");
pub const BALL_HIT_SOUND: &[u8] =
    include_bytes!("./assets/breakout/ball_hit.wav");
pub const WIN_SOUND: &[u8] = include_bytes!("./assets/breakout/win.wav");
pub const MUSIC: &[u8] = include_bytes!("./assets/breakout/music.wav");

struct Block {
    rect: Rectangle,
    score: u32,
    color: Color,
    draw_rect: Rectangle,
    death_time: Option<f32>,
}

impl Block {
    pub fn new(pos: Vector2<f32>, level: usize) -> Self {
        let size = Vector2::new(CELL_WIDTH, CELL_HEIGHT);
        let rect = Rectangle::from_bot_center(pos, size);

        let (score, color) = match level {
            0 => (1, YELLOW),
            1 => (3, GREEN),
            2 => (5, ORANGE),
            3 => (7, RED),
            _ => {
                panic!("Unexpected block index")
            }
        };

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
            score,
            color,
            draw_rect,
            death_time: None,
        }
    }
}

#[derive(Debug)]
struct Ball {
    circle: Circle,
    speed: f32,
    velocity: Vector2<f32>,
    is_dead: bool,
}

impl Ball {
    pub fn new() -> Self {
        Self {
            circle: Circle::from_bot(WINDOW_CENTER, BALL_RADIUS),
            speed: BALL_START_SPEED,
            velocity: Vector2::zeros(),
            is_dead: false,
        }
    }
}

struct Paddle {
    rect: Rectangle,
    max_speed: f32,
    velocity: f32,
    acceleration: f32,
    is_shrinked: bool,
}

impl Paddle {
    pub fn new() -> Self {
        let x = WINDOW_CENTER.x;
        let y = FIELD_FRAME_THICKNESS + PADDLE_ELEVATION;
        let rect =
            Rectangle::from_bot_center(Vector2::new(x, y), PADDLE_SIZE);

        Self {
            rect,
            max_speed: PADDLE_MAX_SPEED,
            velocity: 0.0,
            acceleration: PADDLE_ACCELERATION,
            is_shrinked: false,
        }
    }
}

#[derive(PartialEq)]
enum State {
    NotStarted,
    Started,
}

struct Game {
    time: f32,
    prev_ticks: u32,
    timer: sdl2::TimerSubsystem,
    should_quit: bool,

    sdl2: sdl2::Sdl,
    input: Input,
    audio_player: AudioPlayer<'static>,
    music: usize,
    block_death_sound: usize,
    ball_death_sound: usize,
    ball_hit_sound: usize,
    win_sound: usize,

    renderer: Renderer,

    glyph_atlas_large: GlyphAtlas,
    glyph_tex_large: u32,
    glyph_atlas_small: GlyphAtlas,
    glyph_tex_small: u32,

    postfx: Program,
    frame: Rectangle,
    field: Rectangle,

    blocks: Vec<Block>,
    paddle: Paddle,
    ball: Ball,

    state: State,
    scores: u32,
    game_over_time: Option<f32>,
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
            MSAA,
        );

        let glyph_atlas_large = GlyphAtlas::new(FONT, 48);
        let glyph_tex_large =
            renderer.load_texture_from_glyph_atlas(&glyph_atlas_large);
        let glyph_atlas_small = GlyphAtlas::new(FONT, 24);
        let glyph_tex_small =
            renderer.load_texture_from_glyph_atlas(&glyph_atlas_small);

        let postfx = renderer.load_screen_rect_program(POSTFX_FRAG_SRC);

        Self {
            time: 0.0,
            prev_ticks: timer.ticks(),
            timer,
            should_quit: false,
            sdl2,
            input,
            audio_player: AudioPlayer::new(),
            music: 0,
            block_death_sound: 0,
            ball_death_sound: 0,
            ball_hit_sound: 0,
            win_sound: 0,
            renderer,
            glyph_atlas_large,
            glyph_tex_large,
            glyph_atlas_small,
            glyph_tex_small,
            postfx,
            frame: Rectangle::zeros(),
            field: Rectangle::zeros(),
            blocks: vec![],
            paddle: Paddle::new(),
            ball: Ball::new(),
            state: State::NotStarted,
            scores: 0,
            game_over_time: None,
        }
    }

    pub fn reset(&mut self) {
        let field = Rectangle::from_center(WINDOW_CENTER, FIELD_SIZE);
        let frame =
            Rectangle::from_center(WINDOW_CENTER, FIELD_FRAME_SIZE);
        let paddle = Paddle::new();
        let ball = Ball::new();

        let mut blocks = Vec::with_capacity(112);
        for i in 0..112 {
            let i_row = i / N_CELLS_X;
            let i_col = i % N_CELLS_X;

            let mut y = field.get_bot_left().y
                + (N_CELLS_Y - 16) as f32 * CELL_HEIGHT;
            let mut x = field.get_bot_left().x + CELL_WIDTH / 2.0;

            y += i_row as f32 * CELL_HEIGHT;
            x += i_col as f32 * CELL_WIDTH;

            let block = Block::new(Vector2::new(x, y), i_row / 2);
            blocks.push(block);
        }

        self.time = 0.0;
        self.frame = frame;
        self.field = field;
        self.blocks = blocks;
        self.paddle = paddle;
        self.ball = ball;
        self.state = State::NotStarted;
        self.scores = 0;
        self.game_over_time = None;
    }

    pub fn update(&mut self) {
        let mut dt =
            (self.timer.ticks() - self.prev_ticks) as f32 / 1000.0;
        while dt > 0.0 {
            self.update_input();
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
        use State::*;

        self.time += dt;

        match self.state {
            NotStarted => {
                self.update_not_started();
            }
            Started => {
                self.update_paddle(dt);
                self.update_ball(dt);
                self.update_blocks();
            }
        };
    }

    fn update_not_started(&mut self) {
        use Keycode::*;

        if self.input.keycodes.is_just_pressed(Space) {
            self.reset();
            if !self.audio_player.is_initialized {
                self.audio_player.init(&self.sdl2);
                self.music =
                    self.audio_player.load_music_from_bytes(MUSIC);
                self.block_death_sound = self
                    .audio_player
                    .load_chunk_from_wav_bytes(BLOCK_DEATH_SOUND);
                self.ball_death_sound = self
                    .audio_player
                    .load_chunk_from_wav_bytes(BALL_DEATH_SOUND);
                self.ball_hit_sound = self
                    .audio_player
                    .load_chunk_from_wav_bytes(BALL_HIT_SOUND);
                self.win_sound =
                    self.audio_player.load_chunk_from_wav_bytes(WIN_SOUND);
            }

            self.audio_player.play_music(self.music);

            self.state = State::Started;
            let mut rng = rand::thread_rng();
            let angle = -PI / 2.0 + rng.gen_range(-PI / 5.0..=PI / 5.0);
            self.ball.velocity =
                Vector2::new(angle.cos(), angle.sin()) * self.ball.speed;
        }
    }

    fn update_paddle(&mut self, dt: f32) {
        use Keycode::*;

        if self.input.keycodes.is_pressed(Right) {
            self.paddle.velocity += self.paddle.acceleration * dt;
        } else if self.input.keycodes.is_pressed(Left) {
            self.paddle.velocity -= self.paddle.acceleration * dt;
        } else {
            self.paddle.velocity = self.paddle.velocity.signum()
                * (self.paddle.velocity.abs() - self.paddle.acceleration)
                    .max(0.0);
        }
        self.paddle.velocity = self.paddle.velocity.signum()
            * self.paddle.max_speed.min(self.paddle.velocity.abs());
        self.paddle
            .rect
            .translate_x_assign(self.paddle.velocity * dt);

        self.paddle
            .rect
            .translate_x_assign(self.paddle.max_speed * dt);
        self.paddle
            .rect
            .translate_x_assign(-self.paddle.max_speed * dt);

        let paddle_min_x = self.paddle.rect.get_min_x();
        let paddle_max_x = self.paddle.rect.get_max_x();
        let field_min_x = self.field.get_min_x();
        let field_max_x = self.field.get_max_x();
        if paddle_min_x < field_min_x {
            self.paddle
                .rect
                .translate_x_assign(field_min_x - paddle_min_x);
        } else if paddle_max_x > field_max_x {
            self.paddle
                .rect
                .translate_x_assign(field_max_x - paddle_max_x);
        }
    }

    fn update_ball(&mut self, dt: f32) {
        let step = self.ball.velocity * dt;

        self.ball.circle.center += step;

        let field_min_x = self.field.get_min_x();
        let field_max_x = self.field.get_max_x();
        let field_min_y = self.field.get_min_y();
        let field_max_y = self.field.get_max_y();

        let paddle_min_x = self.paddle.rect.get_min_x();
        let paddle_max_x = self.paddle.rect.get_max_x();
        let paddle_min_y = self.paddle.rect.get_min_y();
        let paddle_max_y = self.paddle.rect.get_max_y();

        let ball_min_x = self.ball.circle.get_min_x();
        let ball_max_x = self.ball.circle.get_max_x();
        let ball_max_y = self.ball.circle.get_max_y();
        let ball_min_y = self.ball.circle.get_min_y();
        let ball_radius = self.ball.circle.radius;

        if ball_min_x < field_min_x {
            self.ball.velocity = reflect(&self.ball.velocity, &RIGHT);
            self.ball.circle.center.x =
                field_min_x + self.ball.circle.radius;
            self.audio_player.play_chunk(self.ball_hit_sound);
        } else if ball_max_x > field_max_x {
            self.ball.velocity = reflect(&self.ball.velocity, &LEFT);
            self.ball.circle.center.x =
                field_max_x - self.ball.circle.radius;
            self.audio_player.play_chunk(self.ball_hit_sound);
        } else if ball_max_y > field_max_y {
            self.ball.velocity = reflect(&self.ball.velocity, &DOWN);
            self.ball.circle.center.y =
                field_max_y - self.ball.circle.radius;
            if !self.paddle.is_shrinked {
                let mut size = self.paddle.rect.get_size();
                size.x *= 0.5;
                self.paddle.rect = Rectangle::from_center(
                    self.paddle.rect.get_center(),
                    size,
                );
                self.paddle.is_shrinked = true;
            }
            self.audio_player.play_chunk(self.ball_hit_sound);
        } else if ball_min_y < field_min_y {
            self.state = State::NotStarted;
            self.ball.is_dead = true;
            self.game_over_time = Some(self.time);
            // self.ball.velocity = reflect(&self.ball.velocity, &UP);
            // self.ball.circle.center.y =
            //     field_min_y + self.ball.circle.radius;

            self.audio_player.play_chunk(self.ball_death_sound);
            self.audio_player.fade_out_music(800);
        }

        if let Some(mtv) =
            get_circle_rectangle_mtv(&self.ball.circle, &self.paddle.rect)
        {
            self.ball.circle.center += mtv;
            let k = (self.ball.circle.center.x - paddle_min_x)
                / (paddle_max_x - paddle_min_x);
            let angle = if self.ball.circle.center.y >= paddle_min_y {
                PI * (0.75 - 0.5 * k)
            } else {
                PI * (0.5 * k - 0.75)
            };
            self.ball.velocity =
                Vector2::new(angle.cos(), angle.sin()) * self.ball.speed;
            self.audio_player.play_chunk(self.ball_hit_sound);
        }

        self.ball.velocity =
            self.ball.velocity.normalize() * self.ball.speed;
    }

    fn update_blocks(&mut self) {
        let mut n_blocks_dead = 0;
        for block in self.blocks.iter_mut() {
            if block.death_time.is_some() {
                n_blocks_dead += 1;
            } else if let Some(mtv) =
                get_circle_rectangle_mtv(&self.ball.circle, &block.rect)
            {
                self.ball.circle.center += mtv;
                self.ball.velocity = reflect(&self.ball.velocity, &mtv);
                self.scores += block.score;
                block.death_time = Some(self.time);
                self.audio_player.play_chunk(self.block_death_sound);
                break;
            }
        }

        if n_blocks_dead == self.blocks.len() {
            self.game_over_time = Some(self.time);
            self.state = State::NotStarted;

            self.audio_player.play_chunk(self.win_sound);
            self.audio_player.fade_out_music(800);
        } else {
            self.ball.speed = BALL_START_SPEED
                * (1.0 + BALL_SPEAD_INCREAS_FACTOR * n_blocks_dead as f32);
        }
    }

    fn update_renderer(&mut self) {
        // Draw objects and playing filed (w/o texture)
        self.renderer.start_new_batch(ProjScreen, None);
        self.renderer.draw_rect(self.frame, None, Some(WHITE));
        self.renderer.draw_rect(self.field, None, Some(BLACK));
        self.renderer.draw_rect(self.paddle.rect, None, Some(WHITE));
        self.draw_ball();
        self.draw_blocks();

        // Draw large texts:
        self.renderer
            .start_new_batch(ProjScreen, Some(self.glyph_tex_large));
        if let Some(time) = self.game_over_time {
            self.draw_game_over(time);
        }
        self.draw_scores();

        // Draw small texts
        self.renderer
            .start_new_batch(ProjScreen, Some(self.glyph_tex_small));
        if self.state == State::NotStarted {
            self.draw_press_space();
        }

        // Draw postfx and end drawing
        self.renderer.end_drawing(BLACK, Some(&self.postfx));

        self.renderer.swap_window();
    }

    fn draw_ball(&mut self) {
        if let Some(time) = self.game_over_time {
            let dt = self.time - time;
            if dt <= BALL_DEATH_ANIM_TIME {
                let k = (dt / BALL_DEATH_ANIM_TIME).min(1.0);
                let alpha = 1.0 - k;
                let scale = 1.0 + 2.0 * k;
                let circle = Circle::new(
                    self.ball.circle.center,
                    self.ball.circle.radius * scale,
                );
                self.renderer.draw_circle(
                    circle,
                    None,
                    Some(WHITE.with_alpha(alpha)),
                );
            }
        } else {
            self.renderer
                .draw_circle(self.ball.circle, None, Some(WHITE));
        }
    }

    fn draw_blocks(&mut self) {
        for block in self.blocks.iter() {
            if let Some(death_time) = block.death_time {
                let dt = self.time - death_time;
                if dt <= BLOCK_DEATH_ANIM_TIME {
                    let k = (dt / BLOCK_DEATH_ANIM_TIME).min(1.0);
                    let alpha = 1.0 - k;
                    let scale = 1.0 + k;
                    let rect = Rectangle::from_center(
                        block.draw_rect.get_center(),
                        block.draw_rect.get_size() * scale,
                    );
                    self.renderer.draw_rect(
                        rect,
                        None,
                        Some(block.color.with_alpha(alpha)),
                    );
                }
            } else {
                self.renderer.draw_rect(
                    block.draw_rect,
                    None,
                    Some(block.color),
                );
            }
        }
    }

    fn draw_scores(&mut self) {
        let mut scores_pos = self.field.get_top_right();
        scores_pos.x -= 90.0;
        scores_pos.y -= 50.0;
        let scores = format!("{:03}", self.scores);
        for glyph in self
            .glyph_atlas_large
            .iter_text_glyphs(Pivot::BotLeft(scores_pos), &scores)
        {
            self.renderer.draw_glyph(glyph, Some(WHITE.with_alpha(1.0)));
        }
    }

    fn draw_press_space(&mut self) {
        let text = "Press [SPACE]";
        let mut pos = WINDOW_CENTER;
        pos.y -= 50.0;
        let mut alpha = (self.time * 2.0 * PI).sin();
        alpha = (alpha + 2.0) / 3.0;

        for glyph in self
            .glyph_atlas_small
            .iter_text_glyphs(Pivot::Center(pos), &text)
        {
            self.renderer
                .draw_glyph(glyph, Some(WHITE.with_alpha(alpha)));
        }
    }

    fn draw_game_over(&mut self, time: f32) {
        let text = if self.ball.is_dead {
            "Game Over"
        } else {
            "Victory!"
        };
        let alpha = ((self.time - time) / 0.2).min(1.0);
        let pos = WINDOW_CENTER;
        for glyph in self
            .glyph_atlas_large
            .iter_text_glyphs(Pivot::Center(pos), &text)
        {
            self.renderer
                .draw_glyph(glyph, Some(WHITE.with_alpha(alpha)));
        }
    }
}

pub fn main() {
    let mut game = Game::new();
    game.reset();

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
        use simg::emscripten::set_main_loop_callback;
        set_main_loop_callback(move || {
            update();
        });
    }
}
