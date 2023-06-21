use nalgebra::Vector2;
use sdl2::keyboard::Keycode;
use simg::camera::Camera2D;
use simg::color::*;
use simg::common::Pivot;
use simg::geometry::*;
use simg::glyph_atlas::GlyphAtlas;
use simg::input::Input;
use simg::program::Texture;
use simg::renderer::Projection::*;
use simg::renderer::Renderer;
use simg::shapes::*;

const MSAA: i32 = 16;

const GAME_DT: f32 = 0.005;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

const PLAYER_CIRCLE_RADIUS: f32 = 20.0;

const MAX_N_ENEMIES: usize = 100;
const ENEMY_CIRCLE_RADIUR: f32 = 20.0;

const SPAWN_START_PERIOD: f32 = 2.0;
const SPAWN_RADIUS: f32 = 400.0;

const CURSOR_BLINK_PERIOD: f32 = 0.5;

pub const FONT: &[u8] = include_bytes!(
    "../assets/fonts/share_tech_mono/ShareTechMono-Regular.ttf"
);

struct Player {
    circle: Circle,
}

impl Player {
    pub fn new() -> Self {
        Self {
            circle: Circle::new(Vector2::zeros(), PLAYER_CIRCLE_RADIUS),
        }
    }
}

struct Enemy {
    is_alive: bool,
    name: String,
    circle: Circle,
    speed: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            is_alive: false,
            name: String::new(),
            circle: Circle::zeros(),
            speed: 20.0,
        }
    }
}

impl Enemy {
    pub fn reset(&mut self, name: String, position: Vector2<f32>) {
        self.is_alive = true;
        self.name = name;
        self.circle = Circle::new(position, ENEMY_CIRCLE_RADIUR);
    }
}

struct Game {
    dt: f32,
    time: f32,
    prev_ticks: u32,
    timer: sdl2::TimerSubsystem,
    should_quit: bool,

    sdl2: sdl2::Sdl,
    input: Input,
    renderer: Renderer,

    glyph_atlas_small: GlyphAtlas,
    glyph_tex_small: Texture,

    camera: Camera2D,
    player: Player,
    enemies: [Enemy; MAX_N_ENEMIES],
    text_input: String,

    prev_spawn_time: f32,
    spawn_period: f32,
    last_type_time: f32,
}

impl Game {
    pub fn new() -> Self {
        let sdl2 = sdl2::init().unwrap();
        let timer = sdl2.timer().unwrap();
        let input = Input::new(&sdl2);
        let mut renderer = Renderer::new(
            &sdl2,
            "Type and Shoot",
            WINDOW_WIDTH as u32,
            WINDOW_HEIGHT as u32,
            MSAA,
        );

        let glyph_atlas_small = GlyphAtlas::new(FONT, 27);
        let glyph_tex_small =
            renderer.load_texture_from_glyph_atlas(&glyph_atlas_small);

        Self {
            dt: 0.0,
            time: 0.0,
            prev_ticks: timer.ticks(),
            timer,
            should_quit: false,

            sdl2,
            input,
            renderer,

            glyph_atlas_small,
            glyph_tex_small,

            camera: Camera2D::new(Vector2::zeros()),
            player: Player::new(),
            enemies: [(); MAX_N_ENEMIES].map(|_| Enemy::default()),
            text_input: String::with_capacity(32),

            prev_spawn_time: 0.0,
            spawn_period: SPAWN_START_PERIOD,
            last_type_time: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.should_quit = false;
    }

    pub fn update(&mut self) {
        let mut dt =
            (self.timer.ticks() - self.prev_ticks) as f32 / 1000.0;
        while dt > 0.0 {
            self.dt = dt.min(GAME_DT);
            self.time += self.dt;
            dt -= GAME_DT;

            self.input.update();
            self.update_game();
        }
        self.prev_ticks = self.timer.ticks();
        self.should_quit |= self.input.should_quit;

        self.update_renderer();
    }

    fn update_game(&mut self) {
        // ---------------------------------------------------------------
        // Update text input
        if self.input.text_input.len() > 0 {
            self.text_input.push_str(&self.input.text_input);
            self.last_type_time = self.time;
        }
        if self.input.keycodes.is_just_repeated(Keycode::Backspace) {
            self.text_input.pop();
            self.last_type_time = self.time;
        }

        let mut is_text_submited = false;
        if self.input.keycodes.is_just_pressed(Keycode::Return) {
            is_text_submited = true;
        }

        // ---------------------------------------------------------------
        // Update enemies
        let mut free_idx = -1;
        let mut is_all_dead = true;
        let mut is_player_shot = false;
        for (idx, enemy) in self.enemies.iter_mut().enumerate() {
            if !enemy.is_alive && free_idx == -1 {
                free_idx = idx as i32;
            }

            if is_text_submited && enemy.name == self.text_input {
                is_player_shot = true;
                enemy.is_alive = false;
            }

            if enemy.is_alive {
                is_all_dead = false;
                if get_circle_circle_mtv(
                    &enemy.circle,
                    &self.player.circle,
                )
                .is_some()
                {
                    // Kill or attack player here
                } else {
                    let step = -enemy.circle.center.normalize()
                        * enemy.speed
                        * self.dt;
                    enemy.circle.center += step;
                }
            }
        }

        // ---------------------------------------------------------------
        // Update player
        if is_text_submited && !is_player_shot {
            // Player receive damage (text has been submited,
            // but no enemy matched)
        }

        // ---------------------------------------------------------------
        // Spawn new enemy
        if is_all_dead
            || (free_idx != -1
                && self.time - self.prev_spawn_time >= self.spawn_period)
        {
            let name = format!("Enemy_{}", free_idx);
            let position = get_rnd_unit_2d() * SPAWN_RADIUS;
            self.enemies[free_idx as usize].reset(name, position);
            self.prev_spawn_time = self.time;
        }

        // ---------------------------------------------------------------
        // Clear text buffer if it has been submited
        if is_text_submited {
            self.text_input.clear();
        }
    }

    fn update_renderer(&mut self) {
        // ---------------------------------------------------------------
        // Draw player and enemies
        self.renderer.start_new_batch(Proj2D(self.camera), None);
        self.renderer
            .draw_circle(self.player.circle, None, Some(RED));

        for enemy in self.enemies.iter().filter(|e| e.is_alive) {
            self.renderer.draw_circle(enemy.circle, None, Some(YELLOW));
        }

        // ---------------------------------------------------------------
        // Draw enemy names
        self.renderer.start_new_batch(
            Proj2D(self.camera),
            Some(self.glyph_tex_small),
        );

        for enemy in self.enemies.iter().filter(|e| e.is_alive) {
            let mut pos = enemy.circle.get_top();
            pos.y += 10.0;

            let mut n_matched = 0;
            if self.text_input.len() > 0
                && enemy.name.starts_with(&self.text_input)
            {
                n_matched = self.text_input.len();
            }

            let mut glyph_idx = 0;
            for glyph in self
                .glyph_atlas_small
                .iter_text_glyphs(Pivot::BotCenter(pos), &enemy.name)
            {
                let color = if glyph_idx < n_matched {
                    Color::new(0.3, 0.9, 0.2, 1.0)
                } else {
                    Color::gray(0.9, 1.0)
                };
                glyph_idx += 1;

                self.renderer.draw_glyph(glyph, Some(color));
            }
        }

        // ---------------------------------------------------------------
        // Draw text input
        self.renderer
            .start_new_batch(ProjScreen, Some(self.glyph_tex_small));

        let atlas = &self.glyph_atlas_small;
        let color = Color::gray(0.9, 1.0);
        let mut cursor = Vector2::new(20.0, 20.0);

        let pivot = Pivot::BotLeft(cursor);
        for glyph in atlas.iter_text_glyphs(pivot, "> ") {
            cursor += glyph.advance;
            self.renderer.draw_glyph(glyph, Some(color));
        }

        let pivot = Pivot::BotLeft(cursor);
        for glyph in atlas.iter_text_glyphs(pivot, &self.text_input) {
            cursor += glyph.advance;
            self.renderer.draw_glyph(glyph, Some(color));
        }

        // ---------------------------------------------------------------
        // Draw cursor rectangle
        if ((self.time - self.last_type_time) / CURSOR_BLINK_PERIOD) as u32
            % 2
            == 0
        {
            self.renderer.start_new_batch(ProjScreen, None);

            let cursor_rect = Rectangle::from_bot_left(
                Vector2::new(cursor.x, cursor.y + atlas.glyph_descent),
                Vector2::new(
                    atlas.font_size as f32 / 2.0,
                    atlas.glyph_ascent - atlas.glyph_descent,
                ),
            );
            self.renderer.draw_rect(cursor_rect, None, Some(color));
        }

        // ---------------------------------------------------------------
        // Finalize drawing
        self.renderer.end_drawing(Color::gray(0.1, 1.0), None);
        self.renderer.swap_window();
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
