use core::f32::consts::PI;
use nalgebra::{point, vector, Point2, Vector2};
use rand::seq::SliceRandom;
use sdl2::keyboard::Keycode;
use simg::color::*;
use simg::common::*;
use simg::geometry::*;
use simg::glyph_atlas::GlyphAtlas;
use simg::input::Input;
use simg::renderer::Renderer;
use simg::shapes::*;

const MSAA: i32 = 16;

const GAME_DT: f32 = 0.005;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 800.0;

const FRAME_TOP_SIZE: f32 = 50.0;
const FRAME_BOT_SIZE: f32 = 50.0;

const COMMAND_RECT_WIDTH: f32 = 110.0;
const COMMAND_RECT_HEIGHT: f32 = 50.0;

const PLAYER_CIRCLE_RADIUS: f32 = 10.0;
const ENEMY_CIRCLE_RADIUR: f32 = 10.0;

const N_ENEMIES_MAX: usize = 1024;
const N_BULLETS_MAX: usize = 32;
const BULLET_MAX_TRAVEL_DIST: f32 = 1000.0;

const SPAWN_RADIUS: f32 = 400.0;
const N_SPAWN_POSITIONS: usize = 10;

const FRICTION_K: f32 = 0.95;

const LEVEL0_N_ENEMIES_TO_SPAWN: usize = 2;
const LEVEL0_SPAWN_PER_MINUTE: f32 = 20.0;
const LEVEL0_ENEMY_SPEED: f32 = 20.0;
const LEVEL0_KNOCKBACK_RADIUS: f32 = 300.0;
const LEVEL0_KNOCKBACK_SPEED: f32 = 1500.0;

const CURSOR_BLINK_PERIOD: f32 = 0.5;

const FONT_SMALL_SIZE: u32 = 20;

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
            circle: Circle::new(Point2::origin(), PLAYER_CIRCLE_RADIUS),
        }
    }
}

#[derive(Clone)]
struct Enemy {
    is_alive: bool,
    name: String,
    circle: Circle,
    speed: f32,
    velocity: Vector2<f32>,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            is_alive: false,
            name: String::new(),
            circle: Circle::zeros(),
            speed: 0.0,
            velocity: Vector2::zeros(),
        }
    }
}

impl Enemy {
    pub fn reset(
        &mut self,
        name: String,
        position: Point2<f32>,
        speed: f32,
    ) {
        self.is_alive = true;
        self.name = name;
        self.speed = speed;
        self.velocity = Vector2::zeros();
        self.circle = Circle::new(position, ENEMY_CIRCLE_RADIUR);
    }
}

#[derive(Debug, Copy, Clone)]
struct Bullet {
    is_alive: bool,
    curr_position: Point2<f32>,
    prev_position: Point2<f32>,
    velocity: Vector2<f32>,
    damage: u32,
}

impl Bullet {
    pub fn reset(
        &mut self,
        curr_position: Point2<f32>,
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
            curr_position: Point2::origin(),
            prev_position: Point2::origin(),
            velocity: Vector2::zeros(),
            damage: 0,
        }
    }
}

struct Command {
    name: String,
    cooldown: f32,
    last_use_time: f32,
    duration: f32,
}

impl Command {
    pub fn new(name: &str, cooldown: f32, duration: f32) -> Self {
        Self {
            name: name.to_string(),
            cooldown,
            last_use_time: -f32::MAX,
            duration,
        }
    }

    pub fn new_start() -> Self {
        Self::new("Start", 0.0, 0.0)
    }

    pub fn new_restart() -> Self {
        Self::new("Restart", 0.0, 0.0)
    }

    pub fn new_pause() -> Self {
        Self::new("Pause", 5.0, 0.0)
    }

    pub fn new_continue() -> Self {
        Self::new("Continue", 0.0, 0.0)
    }

    pub fn new_knockback() -> Self {
        Self::new("Knockback", 12.0, 0.0)
    }

    pub fn new_blizzard() -> Self {
        Self::new("Blizzard", 12.0, 5.0)
    }

    pub fn new_armageddon() -> Self {
        Self::new("Armageddon", f32::MAX, 0.0)
    }

    pub fn is_active(&self, time: f32) -> bool {
        time - self.last_use_time <= self.duration
    }

    pub fn try_activate(&mut self, time: f32, text: &str) {
        if self.is_ready(time) && text == self.name {
            self.last_use_time = time;
        }
    }

    pub fn refresh(&mut self) {
        self.last_use_time = -f32::MAX;
    }

    fn is_ready(&self, time: f32) -> bool {
        (time - self.last_use_time) >= (self.cooldown + self.duration)
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
    playing_time: f32,
    curr_state_time: f32,
    prev_ticks: u32,
    timer: sdl2::TimerSubsystem,
    should_quit: bool,

    camera: Camera2D,
    player: Player,
    bullets: [Bullet; N_BULLETS_MAX],
    enemies: [Enemy; N_ENEMIES_MAX],
    spawn_positions: [Point2<f32>; N_SPAWN_POSITIONS],
    text_input: String,
    submited_text_input: Option<String>,
    words: Vec<&'static str>,

    level_idx: usize,
    spawn_per_minute: f32,
    n_enemies_to_spawn: usize,
    n_enemies_spawned: usize,
    n_enemies_killed: usize,
    prev_spawn_time: f32,
    last_type_time: f32,

    start_command: Command,
    restart_command: Command,
    pause_command: Command,
    continue_command: Command,
    knockback_command: Command,
    blizzard_command: Command,
    armageddon_command: Command,
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
        let mut spawn_positions = [Point2::origin(); N_SPAWN_POSITIONS];
        for i in 0..N_SPAWN_POSITIONS {
            let angle = angle_step * (i as f32 + 0.5);
            let position = get_unit_2d_by_angle(angle) * SPAWN_RADIUS;
            spawn_positions[i] = Point2::from(position);
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
            playing_time: 0.0,
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
            spawn_per_minute: LEVEL0_SPAWN_PER_MINUTE,
            n_enemies_to_spawn: LEVEL0_N_ENEMIES_TO_SPAWN,
            n_enemies_spawned: 0,
            n_enemies_killed: 0,
            prev_spawn_time: 0.0,
            last_type_time: 0.0,

            start_command: Command::new_start(),
            restart_command: Command::new_restart(),
            pause_command: Command::new_pause(),
            continue_command: Command::new_continue(),
            knockback_command: Command::new_knockback(),
            blizzard_command: Command::new_blizzard(),
            armageddon_command: Command::new_armageddon(),
        }
    }

    pub fn restart(&mut self) {
        self.state = GameState::StartingLevel;
        self.dt = 0.0;
        self.time = 0.0;
        self.playing_time = 0.0;
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
        self.spawn_per_minute = LEVEL0_SPAWN_PER_MINUTE;
        self.n_enemies_to_spawn = LEVEL0_N_ENEMIES_TO_SPAWN;
        self.n_enemies_spawned = 0;
        self.n_enemies_killed = 0;
        self.prev_spawn_time = 0.0;
        self.last_type_time = 0.0;
        self.pause_command = Command::new_pause();
        self.continue_command = Command::new_continue();
        self.knockback_command = Command::new_knockback();
        self.blizzard_command = Command::new_blizzard();
        self.armageddon_command = Command::new_armageddon();
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
        self.renderer.set_depth_test(false);

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
            self.submited_text_input = Some(self.text_input.clone());
            self.text_input.clear();
        }
    }

    fn update_starting_level_state(&mut self) {
        if let Some(text) = self.submited_text_input.as_ref() {
            self.start_command.try_activate(self.time, text)
        }

        if self.start_command.is_active(self.time) {
            self.change_state(GameState::Playing);
        }

        self.knockback_command.refresh();
        self.blizzard_command.refresh();
        self.armageddon_command.refresh();
    }

    fn update_playing_state(&mut self) {
        self.playing_time += self.dt;

        // ---------------------------------------------------------------
        // Update commands
        if let Some(text) = self.submited_text_input.as_ref() {
            self.pause_command.try_activate(self.playing_time, text);
            self.knockback_command.try_activate(self.playing_time, text);
            self.blizzard_command.try_activate(self.playing_time, text);
            self.armageddon_command
                .try_activate(self.playing_time, text);
        }

        // ---------------------------------------------------------------
        // Update bullets
        let mut free_bullet_idx = -1;
        for (idx, bullet) in self.bullets.iter_mut().enumerate() {
            if !bullet.is_alive && free_bullet_idx == -1 {
                free_bullet_idx = idx as i32;
            }

            if bullet.is_alive {
                let travel_dist =
                    bullet.curr_position.coords.magnitude_squared().sqrt();
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
                        self.n_enemies_killed += 1;
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

            let mut kinematic_speed =
                enemy.velocity.magnitude_squared().sqrt();
            if kinematic_speed <= 0.1 {
                enemy.velocity = Vector2::zeros();
                kinematic_speed = 0.0;
            }

            let dist_to_player = enemy
                .circle
                .center
                .coords
                .metric_distance(&self.player.circle.center.coords);
            let dir_to_player = (self.player.circle.center
                - enemy.circle.center)
                .normalize();
            if get_circle_circle_mtv(&enemy.circle, &self.player.circle)
                .is_some()
            {
                is_loss = true;
            } else if self.knockback_command.is_active(self.playing_time)
                && dist_to_player <= LEVEL0_KNOCKBACK_RADIUS
            {
                enemy.velocity = (-dir_to_player) * LEVEL0_KNOCKBACK_SPEED;
            } else if self.armageddon_command.is_active(self.playing_time)
            {
                enemy.name = enemy.name[0..1].to_string();
            } else if !self.blizzard_command.is_active(self.playing_time)
                && kinematic_speed == 0.0
            {
                let step = dir_to_player * enemy.speed * self.dt;
                enemy.circle.center += step;
            } else {
                let step = enemy.velocity * self.dt;
                enemy.circle.center += step;
                enemy.velocity *= FRICTION_K;
            }
        }

        // ---------------------------------------------------------------
        // Update player
        if let (Some(target), true) =
            (player_shot_target, free_bullet_idx != -1)
        {
            let velocity = target.coords.normalize() * 2000.0;
            let damage = 1;
            self.bullets[free_bullet_idx as usize].reset(
                self.player.circle.center,
                velocity,
                damage,
            );
        }

        // ---------------------------------------------------------------
        // Spawn new enemy
        if self.n_enemies_to_spawn != self.n_enemies_spawned
            && (is_all_dead
                || (free_enemy_idx != -1
                    && self.playing_time - self.prev_spawn_time
                        >= 60.0 / self.spawn_per_minute))
        {
            let name = self.words.choose(&mut rand::thread_rng()).unwrap();
            let idx = self.n_enemies_spawned % self.spawn_positions.len();
            if idx == 0 {
                self.spawn_positions.shuffle(&mut rand::thread_rng());
            }
            let position = self.spawn_positions[idx];
            self.enemies[free_enemy_idx as usize].reset(
                name.to_string(),
                position,
                LEVEL0_ENEMY_SPEED,
            );
            self.prev_spawn_time = self.playing_time;
            self.n_enemies_spawned += 1;
        }

        // ---------------------------------------------------------------
        // Finish the level
        if self.n_enemies_killed == self.n_enemies_to_spawn {
            self.start_new_level();
        }

        // ---------------------------------------------------------------
        // Update game state
        if is_loss {
            self.change_state(GameState::Loss);
        } else if self.pause_command.is_active(self.playing_time) {
            self.change_state(GameState::Pause);
        }
    }

    fn update_pause_state(&mut self) {
        if let Some(text) = self.submited_text_input.as_ref() {
            self.continue_command.try_activate(self.time, text)
        }

        if self.continue_command.is_active(self.time) {
            self.change_state(GameState::Playing);
        }
    }

    fn update_loss_state(&mut self) {
        if let Some(text) = self.submited_text_input.as_ref() {
            self.restart_command.try_activate(self.time, text);
        }

        if self.restart_command.is_active(self.time) {
            self.restart();
        }
    }

    fn draw_scene(&mut self) {
        // ---------------------------------------------------------------
        // Draw player, bullets, enemies
        self.renderer.set_screen_proj();
        self.renderer.set_origin_2d_camera();

        self.renderer.draw_circle(
            self.player.circle,
            None,
            Some(Color::new(0.3, 0.5, 0.0, 1.0)),
        );

        let color = if self.blizzard_command.is_active(self.playing_time) {
            Color::new(0.2, 0.3, 0.6, 1.0)
        } else {
            Color::new(0.5, 0.3, 0.0, 1.0)
        };
        for enemy in self.enemies.iter().filter(|e| e.is_alive) {
            self.renderer.draw_circle(enemy.circle, None, Some(color));
        }

        for bullet in self.bullets.iter().filter(|b| b.is_alive) {
            let circle = Circle::new(bullet.curr_position, 5.0);
            self.renderer.draw_circle(circle, None, Some(RED));
        }

        // ---------------------------------------------------------------
        // Draw enemy names
        self.renderer.set_tex(self.glyph_tex_small, true);

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
                Pivot::bot_center(position),
                &enemy.name,
                &self.text_input,
                CONSOLE_TEXT_COLOR,
                MATCHED_TEXT_COLOR,
            );
        }

        // ---------------------------------------------------------------
        // Draw frame
        self.renderer.set_screen_camera();
        let atlas = &self.glyph_atlas_small;
        let top_frame = self.get_top_frame_rect();
        let bot_frame = self.get_bot_frame_rect();
        self.renderer.draw_rect(top_frame, None, Some(FRAME_COLOR));
        self.renderer.draw_rect(bot_frame, None, Some(FRAME_COLOR));

        // ---------------------------------------------------------------
        // Draw level progress status (bar and text)
        let bot_left = top_frame.get_bot_left();
        let advance = draw_text(
            atlas,
            &mut self.renderer,
            Pivot::bot_left(point![bot_left.x + 8.0, bot_left.y + 8.0]),
            &format!("Level: {}", self.level_idx),
            CONSOLE_TEXT_COLOR,
        );

        let p = 1.0
            - (self.n_enemies_killed as f32
                / self.n_enemies_to_spawn as f32);
        let size = vector![advance * p, 3.0];
        let position = point![bot_left.x + 8.0, bot_left.y + 3.0];
        let rect = Rectangle::from_left_center(position, size);
        self.renderer.draw_rect(rect, None, Some(RED));

        // ---------------------------------------------------------------
        // Draw Blizzard command
        let blizzard_rect = draw_command(
            &self.glyph_atlas_small,
            &mut self.renderer,
            Pivot::bot_right(top_frame.get_bot_right()),
            &self.blizzard_command,
            &self.text_input,
            self.playing_time,
        );

        // ---------------------------------------------------------------
        // Draw Knockback command
        let knockback_rect = draw_command(
            &self.glyph_atlas_small,
            &mut self.renderer,
            Pivot::bot_right(blizzard_rect.get_bot_left()),
            &self.knockback_command,
            &self.text_input,
            self.playing_time,
        );

        // ---------------------------------------------------------------
        // Draw Knockback command
        draw_command(
            &self.glyph_atlas_small,
            &mut self.renderer,
            Pivot::bot_right(knockback_rect.get_bot_left()),
            &self.armageddon_command,
            &self.text_input,
            self.playing_time,
        );

        // ---------------------------------------------------------------
        // Draw console text input
        let mut cursor = point![20.0, 20.0];
        cursor.x += draw_text(
            atlas,
            &mut self.renderer,
            Pivot::bot_left(cursor),
            "> ",
            CONSOLE_TEXT_COLOR,
        );

        cursor.x += draw_text(
            atlas,
            &mut self.renderer,
            Pivot::bot_left(cursor),
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
        self.renderer.set_screen_camera();
        self.renderer.set_tex(self.glyph_tex_small, true);
        self.draw_screen_dim();

        // ---------------------------------------------------------------
        // Draw Start button
        let top_frame = self.get_top_frame_rect();
        draw_command(
            &self.glyph_atlas_small,
            &mut self.renderer,
            Pivot::bot_center(top_frame.get_bot_center()),
            &self.start_command,
            &self.text_input,
            self.time,
        );
    }

    fn draw_playing_state(&mut self) {
        self.renderer.set_screen_camera();
        self.renderer.set_tex(self.glyph_tex_small, true);

        // ---------------------------------------------------------------
        // Draw Pause command
        let top_frame = self.get_top_frame_rect();
        draw_command(
            &self.glyph_atlas_small,
            &mut self.renderer,
            Pivot::bot_center(top_frame.get_bot_center()),
            &self.pause_command,
            &self.text_input,
            self.playing_time,
        );
    }

    fn draw_pause_state(&mut self) {
        self.renderer.set_screen_camera();
        self.renderer.set_tex(self.glyph_tex_small, true);

        self.draw_screen_dim();

        // ---------------------------------------------------------------
        // Draw Continue command
        let top_frame = self.get_top_frame_rect();
        draw_command(
            &self.glyph_atlas_small,
            &mut self.renderer,
            Pivot::bot_center(top_frame.get_bot_center()),
            &self.continue_command,
            &self.text_input,
            self.time,
        );
    }

    fn draw_loss_state(&mut self) {
        self.renderer.set_screen_camera();
        self.renderer.set_tex(self.glyph_tex_small, true);

        self.draw_screen_dim();

        // ---------------------------------------------------------------
        // Draw Restart button
        let top_frame = self.get_top_frame_rect();
        draw_command(
            &self.glyph_atlas_small,
            &mut self.renderer,
            Pivot::bot_center(top_frame.get_bot_center()),
            &self.restart_command,
            &self.text_input,
            self.time,
        );
    }

    fn draw_screen_dim(&mut self) {
        self.renderer.draw_rect(
            Rectangle::from_bot_left(
                Point2::origin(),
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
            LEVEL0_SPAWN_PER_MINUTE + (1.5 * (self.level_idx + 1) as f32);
        self.n_enemies_spawned = 0;
        self.n_enemies_killed = 0;
        self.change_state(GameState::StartingLevel);
    }

    fn change_state(&mut self, state: GameState) {
        if self.state != state {
            self.state = state;
            self.curr_state_time = 0.0;
        }
    }

    fn get_top_frame_rect(&self) -> Rectangle {
        Rectangle::from_top_left(
            point![0.0, WINDOW_HEIGHT],
            vector![WINDOW_WIDTH, FRAME_TOP_SIZE],
        )
    }

    fn get_bot_frame_rect(&self) -> Rectangle {
        Rectangle::from_bot_left(
            Point2::origin(),
            vector![WINDOW_WIDTH, FRAME_BOT_SIZE],
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

fn draw_command(
    glyph_atlas: &GlyphAtlas,
    renderer: &mut Renderer,
    pivot: Pivot,
    command: &Command,
    to_match: &str,
    curr_time: f32,
) -> Rectangle {
    let last_use_time = command.last_use_time;
    let duration = command.duration;
    let cooldown = command.cooldown;

    let (p, bar_color, text_color, matched_color) =
        if command.is_active(curr_time) {
            let p = 1.0 - (curr_time - last_use_time) / duration;

            (p, GREEN, CONSOLE_DIM_TEXT_COLOR, RED)
        } else {
            let mut p = (curr_time - last_use_time - duration) / cooldown;
            p = p.min(1.0);

            let (bar_color, text_color, matched_color) = if p >= 1.0 {
                let c = (curr_time * 2.0 * PI).sin() * 0.3;
                let text_color = CONSOLE_TEXT_COLOR.add_white(c);
                (GREEN, text_color, MATCHED_TEXT_COLOR)
            } else {
                (GREEN.with_alpha(0.4), CONSOLE_DIM_TEXT_COLOR, RED)
            };

            (p, bar_color, text_color, matched_color)
        };

    let size = vector![COMMAND_RECT_WIDTH, COMMAND_RECT_HEIGHT];
    let rect = Rectangle::from_pivot(pivot, size);
    let bot_center = rect.get_bot_center();

    let bar_width = p * (rect.get_width() - 10.0);
    let bar_rect = Rectangle::from_bot_center(
        point![bot_center.x, bot_center.y + 3.0],
        vector![bar_width, 3.0],
    );
    let text_pos = point![bot_center.x, bot_center.y + 9.0];
    let text_pivot = Pivot::bot_center(text_pos);

    renderer.draw_rect(bar_rect, None, Some(bar_color));

    draw_text_with_match(
        glyph_atlas,
        renderer,
        text_pivot,
        &command.name,
        to_match,
        text_color,
        matched_color,
    );

    rect
}

fn get_cursor_rect(
    cursor: &Point2<f32>,
    glyph_atlas: &GlyphAtlas,
) -> Rectangle {
    Rectangle::from_bot_left(
        point![cursor.x, cursor.y + glyph_atlas.glyph_descent],
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
