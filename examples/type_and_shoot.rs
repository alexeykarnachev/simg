use core::f32::consts::PI;
use nalgebra::vector;
use nalgebra::Vector2;
use rand::seq::SliceRandom;
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

const PAUSE_PERIOD: f32 = 5.0;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

const FRAME_TOP_SIZE: f32 = 50.0;
const FRAME_BOT_SIZE: f32 = 50.0;

const PLAYER_CIRCLE_RADIUS: f32 = 15.0;

const N_BULLETS_MAX: usize = 32;
const BULLET_MAX_TRAVEL_DIST: f32 = 1000.0;

const N_ENEMIES_MAX: usize = 1024;
const ENEMY_CIRCLE_RADIUR: f32 = 15.0;

const SPAWN_RADIUS: f32 = 400.0;
const N_SPAWN_POSITIONS: usize = 10;

const LEVEL0_N_ENEMIES_TO_SPAWN: usize = 2;
const LEVEL0_SPAWN_PER_MINUTE: f32 = 20.0;

const CURSOR_BLINK_PERIOD: f32 = 0.5;

const FONT_SMALL_SIZE: u32 = 20;

const PAUSE_TEXT: &str = "Pause";
const CONTINUE_TEXT: &str = "Continue";
const START_TEXT: &str = "Start";
const RESTART_TEXT: &str = "Restart";

const CLEAR_COLOR: Color = Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 };
const FRAME_COLOR: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.9 };
const CONSOLE_TEXT_COLOR: Color = Color { r: 0.9, g: 0.9, b: 0.9, a: 1.0 };
const CONSOLE_DIM_TEXT_COLOR: Color =
    Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 };
const MATCHED_TEXT_COLOR: Color = Color { r: 0.2, g: 1.0, b: 0.1, a: 1.0 };
const DIM_SCREEN_COLOR: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.3 };

pub const FONT: &[u8] = include_bytes!(
    "../assets/fonts/share_tech_mono/ShareTechMono-Regular.ttf"
);
static WORDS: &'static str = include_str!("./assets/type_and_shoot/words");

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

#[derive(Clone)]
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
            speed: 0.0,
        }
    }
}

impl Enemy {
    pub fn reset(
        &mut self,
        name: String,
        position: Vector2<f32>,
        speed: f32,
    ) {
        self.is_alive = true;
        self.name = name;
        self.speed = speed;
        self.circle = Circle::new(position, ENEMY_CIRCLE_RADIUR);
    }
}

#[derive(Debug, Copy, Clone)]
struct Bullet {
    is_alive: bool,
    curr_position: Vector2<f32>,
    prev_position: Vector2<f32>,
    velocity: Vector2<f32>,
    damage: u32,
}

impl Bullet {
    pub fn reset(
        &mut self,
        curr_position: Vector2<f32>,
        velocity: Vector2<f32>,
        damage: u32,
    ) {
        self.is_alive = true;
        self.curr_position = curr_position;
        self.prev_position = curr_position;
        self.velocity = velocity;
        self.damage = damage;
    }
}

impl Default for Bullet {
    fn default() -> Self {
        Self {
            is_alive: false,
            curr_position: Vector2::zeros(),
            prev_position: Vector2::zeros(),
            velocity: Vector2::zeros(),
            damage: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum GameState {
    StartingLevel,
    Playing,
    Pause,
    Loss,
}

struct Game {
    sdl2: sdl2::Sdl,
    input: Input,
    renderer: Renderer,

    glyph_atlas_small: GlyphAtlas,
    glyph_tex_small: Texture,

    state: GameState,
    dt: f32,
    time: f32,
    curr_state_time: f32,
    prev_ticks: u32,
    timer: sdl2::TimerSubsystem,
    should_quit: bool,

    camera: Camera2D,
    player: Player,
    bullets: [Bullet; N_BULLETS_MAX],
    enemies: [Enemy; N_ENEMIES_MAX],
    spawn_positions: [Vector2<f32>; N_SPAWN_POSITIONS],
    text_input: String,
    submited_text_input: Option<String>,
    words: Vec<&'static str>,

    level_idx: usize,
    n_enemies_to_spawn: usize,
    spawn_per_minute: f32,
    prev_spawn_time: f32,
    last_type_time: f32,
    last_pause_time: f32,
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

        let glyph_atlas_small = GlyphAtlas::new(FONT, FONT_SMALL_SIZE);
        let glyph_tex_small =
            renderer.load_texture_from_glyph_atlas(&glyph_atlas_small);

        // Generate possible spawn positions
        let angle_step = 2.0 * PI / N_SPAWN_POSITIONS as f32;
        let mut spawn_positions = [Vector2::default(); N_SPAWN_POSITIONS];
        for i in 0..N_SPAWN_POSITIONS {
            let angle = angle_step * (i as f32 + 0.5);
            let position = get_unit_2d_by_angle(angle) * SPAWN_RADIUS;
            spawn_positions[i] = position;
        }

        Self {
            sdl2,
            input,
            renderer,

            glyph_atlas_small,
            glyph_tex_small,

            state: GameState::StartingLevel,
            dt: 0.0,
            time: 0.0,
            curr_state_time: 0.0,
            prev_ticks: timer.ticks(),
            timer,
            should_quit: false,

            camera: Camera2D::default(),
            player: Player::new(),
            bullets: [Bullet::default(); N_BULLETS_MAX],
            enemies: [(); N_ENEMIES_MAX].map(|_| Enemy::default()),
            spawn_positions,
            text_input: String::with_capacity(32),
            submited_text_input: None,
            words: WORDS.lines().collect(),

            level_idx: 0,
            n_enemies_to_spawn: LEVEL0_N_ENEMIES_TO_SPAWN,
            spawn_per_minute: LEVEL0_SPAWN_PER_MINUTE,
            prev_spawn_time: 0.0,
            last_type_time: 0.0,
            last_pause_time: 0.0,
        }
    }

    pub fn restart(&mut self) {
        self.state = GameState::StartingLevel;
        self.dt = 0.0;
        self.time = 0.0;
        self.curr_state_time = 0.0;
        self.timer = self.sdl2.timer().unwrap();
        self.prev_ticks = self.timer.ticks();
        self.should_quit = false;
        self.camera = Camera2D::default();
        self.player = Player::new();
        self.bullets.iter_mut().for_each(|b| b.is_alive = false);
        self.enemies.iter_mut().for_each(|e| e.is_alive = false);
        self.text_input.clear();
        self.submited_text_input = None;
        self.level_idx = 0;
        self.n_enemies_to_spawn = LEVEL0_N_ENEMIES_TO_SPAWN;
        self.spawn_per_minute = LEVEL0_SPAWN_PER_MINUTE;
        self.prev_spawn_time = 0.0;
        self.last_type_time = 0.0;
        self.last_pause_time = 0.0;
    }

    pub fn update(&mut self) {
        use GameState::*;

        // ---------------------------------------------------------------
        // Update game
        let mut dt =
            (self.timer.ticks() - self.prev_ticks) as f32 / 1000.0;
        while dt > 0.0 {
            self.dt = dt.min(GAME_DT);
            self.time += self.dt;
            dt -= GAME_DT;

            self.input.update();
            self.update_text_input();
            match self.state {
                StartingLevel => {
                    self.update_starting_level_state();
                }
                Playing => {
                    self.update_playing_state();
                }
                Pause => {
                    self.update_pause_state();
                }
                Loss => {
                    self.update_loss_state();
                }
            }
        }
        self.prev_ticks = self.timer.ticks();
        self.should_quit |= self.input.should_quit;

        // ---------------------------------------------------------------
        // Update renderer (draw all the stuff)
        self.draw_scene();
        match self.state {
            StartingLevel => {
                self.draw_starting_level_state();
            }
            Playing => {
                self.draw_playing_state();
            }
            Pause => {
                self.draw_pause_state();
            }
            Loss => {
                self.draw_loss_state();
            }
        }

        // ---------------------------------------------------------------
        // Finalize drawing
        self.renderer.end_drawing(CLEAR_COLOR, None);
        self.renderer.swap_window();
    }

    fn update_text_input(&mut self) {
        if self.input.text_input.len() > 0 {
            self.text_input.push_str(&self.input.text_input);
            self.last_type_time = self.time;
        }
        if self.input.keycodes.is_just_repeated(Keycode::Backspace) {
            self.text_input.pop();
            self.last_type_time = self.time;
        }

        self.submited_text_input = None;
        if self.input.keycodes.is_just_pressed(Keycode::Return) {
            let text_input = self.text_input.clone();

            if text_input == PAUSE_TEXT {
                if self.can_pause() {
                    self.change_state(GameState::Pause);
                }
            } else if text_input == CONTINUE_TEXT {
                self.change_state(GameState::Playing);
            }

            self.submited_text_input = Some(text_input);
            self.text_input.clear();
        }
    }

    fn update_starting_level_state(&mut self) {
        if let Some(text) = self.submited_text_input.as_ref() {
            if text == START_TEXT {
                self.change_state(GameState::Playing);
            }
        }
    }

    fn update_playing_state(&mut self) {
        // ---------------------------------------------------------------
        // Update bullets
        let mut free_bullet_idx = -1;
        for (idx, bullet) in self.bullets.iter_mut().enumerate() {
            if !bullet.is_alive && free_bullet_idx == -1 {
                free_bullet_idx = idx as i32;
            }

            if bullet.is_alive {
                let travel_dist =
                    bullet.curr_position.magnitude_squared().sqrt();
                if travel_dist > BULLET_MAX_TRAVEL_DIST {
                    bullet.is_alive = false;
                } else {
                    let step = bullet.velocity * self.dt;
                    bullet.prev_position = bullet.curr_position;
                    bullet.curr_position += step;
                }
            }
        }

        // ---------------------------------------------------------------
        // Update enemies
        let mut free_enemy_idx = -1;
        let mut is_all_dead = true;
        let mut player_shot_target = None;
        let mut is_loss = false;
        for (idx, enemy) in self.enemies.iter_mut().enumerate() {
            // Try receive bullet damage
            if enemy.is_alive {
                for bullet in
                    self.bullets.iter_mut().filter(|b| b.is_alive)
                {
                    let line = Line::new(
                        bullet.prev_position,
                        bullet.curr_position,
                    );
                    if intersect_line_with_circle(&line, &enemy.circle)[0]
                        .is_some()
                    {
                        bullet.is_alive = false;
                        enemy.is_alive = false;
                        break;
                    }
                }
            }

            if !enemy.is_alive {
                if free_enemy_idx == -1 {
                    free_enemy_idx = idx as i32;
                }

                continue;
            }

            is_all_dead = false;

            if let Some(text_input) = self.submited_text_input.as_ref() {
                if enemy.name == text_input.to_owned() {
                    player_shot_target = Some(enemy.circle.center);
                }
            }

            if get_circle_circle_mtv(&enemy.circle, &self.player.circle)
                .is_some()
            {
                is_loss = true;
            } else {
                let dir = (self.player.circle.center
                    - enemy.circle.center)
                    .normalize();
                let step = dir * enemy.speed * self.dt;
                enemy.circle.center += step;
            }
        }

        if is_loss {
            self.change_state(GameState::Loss);
        }

        // ---------------------------------------------------------------
        // Update player
        if let (Some(target), true) =
            (player_shot_target, free_bullet_idx != -1)
        {
            let velocity = target.normalize() * 2000.0;
            let damage = 1;
            self.bullets[free_bullet_idx as usize].reset(
                self.player.circle.center,
                velocity,
                damage,
            );
        }

        // ---------------------------------------------------------------
        // Spawn new enemy
        if self.n_enemies_to_spawn > 0
            && (is_all_dead
                || (free_enemy_idx != -1
                    && self.time - self.prev_spawn_time
                        >= 60.0 / self.spawn_per_minute))
        {
            let name = self.words.choose(&mut rand::thread_rng()).unwrap();
            let idx = self.n_enemies_to_spawn % self.spawn_positions.len();
            if idx == 0 {
                self.spawn_positions.shuffle(&mut rand::thread_rng());
            }
            let position = self.spawn_positions[idx];
            let speed = 40.0;
            self.enemies[free_enemy_idx as usize].reset(
                name.to_string(),
                position,
                speed,
            );
            self.prev_spawn_time = self.time;
            self.n_enemies_to_spawn -= 1;
        }

        // ---------------------------------------------------------------
        // Finish the level
        if self.n_enemies_to_spawn == 0 && is_all_dead {
            self.start_new_level();
        }
    }

    fn update_pause_state(&mut self) {
        self.last_pause_time = self.time;
    }

    fn update_loss_state(&mut self) {
        if let Some(text) = self.submited_text_input.as_ref() {
            if text == RESTART_TEXT {
                self.restart();
            }
        }
    }

    fn draw_scene(&mut self) {
        // ---------------------------------------------------------------
        // Draw player, bullets, enemies
        self.renderer.start_new_batch(Proj2D(self.camera), None);

        self.renderer.draw_circle(
            self.player.circle,
            None,
            Some(Color::new(0.3, 0.5, 0.0, 1.0)),
        );

        for enemy in self.enemies.iter().filter(|e| e.is_alive) {
            self.renderer.draw_circle(
                enemy.circle,
                None,
                Some(Color::new(0.5, 0.3, 0.0, 1.0)),
            );
        }

        for bullet in self.bullets.iter().filter(|b| b.is_alive) {
            let circle = Circle::new(bullet.curr_position, 5.0);
            self.renderer.draw_circle(circle, None, Some(RED));
        }

        // ---------------------------------------------------------------
        // Draw enemy names
        self.renderer.start_new_batch(
            Proj2D(self.camera),
            Some(self.glyph_tex_small),
        );

        for enemy in self.enemies.iter().filter(|e| e.is_alive) {
            let mut position = enemy.circle.get_top();
            position.y += 10.0;

            // Draw the text rectangle
            let mut size =
                self.glyph_atlas_small.get_text_size(&enemy.name);
            size.y = self.glyph_atlas_small.font_size as f32;
            let rect = Rectangle::from_bot_center(position, size);
            self.renderer.draw_rect(rect, None, Some(BLACK));

            // Draw the actual glyphs
            draw_text_with_match(
                &self.glyph_atlas_small,
                &mut self.renderer,
                Pivot::BotCenter(position),
                &enemy.name,
                &self.text_input,
                CONSOLE_TEXT_COLOR,
                MATCHED_TEXT_COLOR,
            );
        }

        // ---------------------------------------------------------------
        // Draw frame
        self.renderer
            .start_new_batch(ProjScreen, Some(self.glyph_tex_small));

        let top_frame = self.get_top_frame_rect();
        let bot_frame = self.get_bot_frame_rect();
        self.renderer.draw_rect(top_frame, None, Some(FRAME_COLOR));
        self.renderer.draw_rect(bot_frame, None, Some(FRAME_COLOR));

        // ---------------------------------------------------------------
        // Draw console text input
        let atlas = &self.glyph_atlas_small;
        let mut cursor = Vector2::new(20.0, 20.0);
        cursor.x += draw_text(
            atlas,
            &mut self.renderer,
            Pivot::BotLeft(cursor),
            "> ",
            CONSOLE_TEXT_COLOR,
        );

        cursor.x += draw_text(
            atlas,
            &mut self.renderer,
            Pivot::BotLeft(cursor),
            &self.text_input,
            CONSOLE_TEXT_COLOR,
        );

        // ---------------------------------------------------------------
        // Draw cursor rectangle
        let time_since_type = self.time - self.last_type_time;
        if (time_since_type / CURSOR_BLINK_PERIOD) as u32 % 2 == 0 {
            self.renderer.draw_rect(
                get_cursor_rect(&cursor, atlas),
                None,
                Some(CONSOLE_TEXT_COLOR),
            );
        }
    }

    fn draw_starting_level_state(&mut self) {
        self.renderer
            .start_new_batch(ProjScreen, Some(self.glyph_tex_small));
        self.draw_screen_dim();

        let atlas = &self.glyph_atlas_small;
        let top_frame = self.get_top_frame_rect();

        // ---------------------------------------------------------------
        // Draw Start button
        let x = top_frame.get_center_x();
        let y = top_frame.get_min_y();

        draw_text_with_match(
            atlas,
            &mut self.renderer,
            Pivot::BotCenter(vector![x, y + 8.0]),
            START_TEXT,
            &self.text_input,
            CONSOLE_TEXT_COLOR,
            MATCHED_TEXT_COLOR,
        );
    }

    fn draw_playing_state(&mut self) {
        self.renderer
            .start_new_batch(ProjScreen, Some(self.glyph_tex_small));

        let atlas = &self.glyph_atlas_small;
        let top_frame = self.get_top_frame_rect();

        // ---------------------------------------------------------------
        // Draw Pause button
        let x = top_frame.get_center_x();
        let y = top_frame.get_min_y();

        let (color, matched_color) = if self.can_pause() {
            (CONSOLE_TEXT_COLOR, MATCHED_TEXT_COLOR)
        } else {
            (CONSOLE_DIM_TEXT_COLOR, RED)
        };

        let width = draw_text_with_match(
            atlas,
            &mut self.renderer,
            Pivot::BotCenter(vector![x, y + 8.0]),
            PAUSE_TEXT,
            &self.text_input,
            color,
            matched_color,
        );

        // ---------------------------------------------------------------
        // Draw Pause progress bar
        let p = (self.time - self.last_pause_time) / PAUSE_PERIOD;
        let width = width * p.min(1.0);
        let color = if p >= 1.0 {
            GREEN
        } else {
            GREEN.with_alpha(0.4)
        };
        self.renderer.draw_rect(
            Rectangle::from_bot_center(
                vector![x, y + 3.0],
                vector![width, 3.0],
            ),
            None,
            Some(color),
        );
    }

    fn draw_pause_state(&mut self) {
        self.renderer
            .start_new_batch(ProjScreen, Some(self.glyph_tex_small));
        self.draw_screen_dim();

        let atlas = &self.glyph_atlas_small;
        let top_frame = self.get_top_frame_rect();

        // ---------------------------------------------------------------
        // Draw Continue button
        let x = top_frame.get_center_x();
        let y = top_frame.get_min_y();

        draw_text_with_match(
            atlas,
            &mut self.renderer,
            Pivot::BotCenter(vector![x, y + 8.0]),
            CONTINUE_TEXT,
            &self.text_input,
            CONSOLE_TEXT_COLOR,
            MATCHED_TEXT_COLOR,
        );
    }

    fn draw_loss_state(&mut self) {
        self.renderer
            .start_new_batch(ProjScreen, Some(self.glyph_tex_small));
        self.draw_screen_dim();

        let atlas = &self.glyph_atlas_small;
        let top_frame = self.get_top_frame_rect();

        // ---------------------------------------------------------------
        // Draw Restart button
        let x = top_frame.get_center_x();
        let y = top_frame.get_min_y();

        draw_text_with_match(
            atlas,
            &mut self.renderer,
            Pivot::BotCenter(vector![x, y + 8.0]),
            RESTART_TEXT,
            &self.text_input,
            CONSOLE_TEXT_COLOR,
            MATCHED_TEXT_COLOR,
        );
    }

    fn draw_screen_dim(&mut self) {
        self.renderer.draw_rect(
            Rectangle::from_bot_left(
                Vector2::zeros(),
                vector![WINDOW_WIDTH, WINDOW_HEIGHT],
            ),
            None,
            Some(DIM_SCREEN_COLOR),
        );
    }

    fn start_new_level(&mut self) {
        self.level_idx += 1;
        self.n_enemies_to_spawn =
            LEVEL0_N_ENEMIES_TO_SPAWN * (self.level_idx + 1);
        self.spawn_per_minute =
            LEVEL0_SPAWN_PER_MINUTE + (3 * (self.level_idx + 1)) as f32;
        self.change_state(GameState::StartingLevel);
    }

    fn change_state(&mut self, state: GameState) {
        if self.state != state {
            self.state = state;
            self.curr_state_time = 0.0;
        }
    }

    fn can_pause(&self) -> bool {
        self.time - self.last_pause_time >= PAUSE_PERIOD
    }

    fn get_top_frame_rect(&self) -> Rectangle {
        Rectangle::from_top_left(
            Vector2::new(0.0, WINDOW_HEIGHT),
            Vector2::new(WINDOW_WIDTH, FRAME_TOP_SIZE),
        )
    }

    fn get_bot_frame_rect(&self) -> Rectangle {
        Rectangle::from_bot_left(
            Vector2::zeros(),
            Vector2::new(WINDOW_WIDTH, FRAME_BOT_SIZE),
        )
    }
}

fn draw_text(
    glyph_atlas: &GlyphAtlas,
    renderer: &mut Renderer,
    pivot: Pivot,
    text: &str,
    text_color: Color,
) -> f32 {
    let mut advance = 0.0;
    for glyph in glyph_atlas.iter_text_glyphs(pivot, text) {
        advance += glyph.advance.x;
        renderer.draw_glyph(glyph, Some(text_color));
    }

    advance
}

fn draw_text_with_match(
    glyph_atlas: &GlyphAtlas,
    renderer: &mut Renderer,
    pivot: Pivot,
    text: &str,
    to_match: &str,
    text_color: Color,
    match_color: Color,
) -> f32 {
    let mut n_matched = 0;
    if to_match.len() > 0 && text.starts_with(to_match) {
        n_matched = to_match.len();
    }

    let mut advance = 0.0;
    let mut glyph_idx = 0;
    for glyph in glyph_atlas.iter_text_glyphs(pivot, text) {
        let color = if glyph_idx < n_matched {
            match_color
        } else {
            text_color
        };
        glyph_idx += 1;

        advance += glyph.advance.x;
        renderer.draw_glyph(glyph, Some(color));
    }

    advance
}

fn get_cursor_rect(
    cursor: &Vector2<f32>,
    glyph_atlas: &GlyphAtlas,
) -> Rectangle {
    Rectangle::from_bot_left(
        Vector2::new(cursor.x, cursor.y + glyph_atlas.glyph_descent),
        Vector2::new(
            glyph_atlas.font_size as f32 / 2.0,
            glyph_atlas.glyph_ascent - glyph_atlas.glyph_descent,
        ),
    )
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
        use simg::emscripten::set_main_loop_callback;
        set_main_loop_callback(move || {
            update();
        });
    }
}
