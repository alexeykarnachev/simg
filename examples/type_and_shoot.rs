use nalgebra::Vector2;
use simg::camera::Camera2D;
use simg::color::*;
use simg::common::Pivot;
use simg::geometry::*;
use simg::glyph_atlas::GlyphAtlas;
use simg::input::Input;
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

const SPAWN_START_PERIOD: f32 = 1.0;

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
    glyph_tex_small: u32,

    camera: Camera2D,
    player: Player,
    enemies: [Enemy; MAX_N_ENEMIES],

    prev_spawn_time: f32,
    spawn_period: f32,
}

impl Game {
    pub fn new() -> Self {
        let sdl2 = sdl2::init().unwrap();
        let timer = sdl2.timer().unwrap();
        let input = Input::new(&sdl2);
        let mut renderer = Renderer::new(
            &sdl2,
            "Game",
            WINDOW_WIDTH as u32,
            WINDOW_HEIGHT as u32,
            MSAA,
        );

        let glyph_atlas_small = GlyphAtlas::new(FONT, 24);
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

            prev_spawn_time: 0.0,
            spawn_period: SPAWN_START_PERIOD,
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

            self.update_input();
            self.update_game();
        }
        self.prev_ticks = self.timer.ticks();

        self.update_renderer();
    }

    fn update_input(&mut self) {
        self.input.update();
        self.should_quit |= self.input.should_quit;
    }

    fn update_game(&mut self) {
        self.update_enemies();
    }

    fn update_enemies(&mut self) {
        let mut free_idx = -1;
        for (idx, enemy) in self.enemies.iter_mut().enumerate() {
            if !enemy.is_alive && free_idx == -1 {
                free_idx = idx as i32;
            }

            if enemy.is_alive {
                if get_circle_circle_mtv(
                    &enemy.circle,
                    &self.player.circle,
                )
                .is_some()
                {
                } else {
                    let step = -enemy.circle.center.normalize()
                        * enemy.speed
                        * self.dt;
                    enemy.circle.center += step;
                }
            }
        }

        if free_idx != -1
            && self.time - self.prev_spawn_time >= self.spawn_period
        {
            let name = format!("Enemy {}", free_idx);
            let position = get_rnd_unit_2d() * 200.0;
            self.enemies[free_idx as usize].reset(name, position);
            self.prev_spawn_time = self.time;
        }
    }

    fn update_renderer(&mut self) {
        self.renderer.start_new_batch(Proj2D(self.camera), None);
        self.draw_player();
        self.draw_enemies();

        self.renderer.start_new_batch(
            Proj2D(self.camera),
            Some(self.glyph_tex_small),
        );
        self.draw_enemy_names();

        self.renderer.end_drawing(Color::gray(0.1, 1.0), None);
        self.renderer.swap_window();
    }

    fn draw_player(&mut self) {
        self.renderer
            .draw_circle(self.player.circle, None, Some(RED));
    }

    fn draw_enemies(&mut self) {
        for enemy in self.enemies.iter().filter(|e| e.is_alive) {
            self.renderer.draw_circle(enemy.circle, None, Some(YELLOW));
        }
    }

    fn draw_enemy_names(&mut self) {
        for enemy in self.enemies.iter().filter(|e| e.is_alive) {
            let mut pos = enemy.circle.get_top();
            pos.y += 10.0;
            for glyph in self
                .glyph_atlas_small
                .iter_text_glyphs(Pivot::BotCenter(pos), &enemy.name)
            {
                self.renderer
                    .draw_glyph(glyph, Some(Color::gray(0.5, 1.0)));
            }
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
