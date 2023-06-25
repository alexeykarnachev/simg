use nalgebra::{point, Vector2};
use simg::color::*;
use simg::input::Input;
use simg::renderer::Projection::*;
use simg::renderer::Renderer;
use simg::shapes::*;

const MSAA: i32 = 16;

const GAME_DT: f32 = 0.005;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

struct Game {
    dt: f32,
    time: f32,
    prev_ticks: u32,
    timer: sdl2::TimerSubsystem,
    should_quit: bool,

    sdl2: sdl2::Sdl,
    input: Input,

    renderer: Renderer,
}

impl Game {
    pub fn new() -> Self {
        let sdl2 = sdl2::init().unwrap();
        let timer = sdl2.timer().unwrap();
        let input = Input::new(&sdl2);
        let renderer = Renderer::new(
            &sdl2,
            "Game",
            WINDOW_WIDTH as u32,
            WINDOW_HEIGHT as u32,
            MSAA,
        );

        Self {
            dt: 0.0,
            time: 0.0,
            prev_ticks: timer.ticks(),
            timer,
            should_quit: false,
            sdl2,
            input,
            renderer,
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

    fn update_game(&mut self) {}

    fn update_renderer(&mut self) {
        let roll: f32 = 0.0;
        // let pitch: f32 = 0.0;
        let pitch: f32 = self.time * 25.0;
        let yaw: f32 = 0.0;
        // let yaw: f32 = self.time * 25.0;
        let q = nalgebra::UnitQuaternion::from_euler_angles(
            pitch.to_radians(),
            yaw.to_radians(),
            roll.to_radians(),
        );
        println!("{}", q);

        let eye = point![0.0, 0.0, 2.0];
        let target = point![0.0, 0.0, 0.0];
        let eye = (q * (eye - target)) + target.coords;

        self.renderer.start_new_batch(
            Proj3D { eye: eye.into(), target, fovy: 60.0 },
            None,
        );

        let triangle = Triangle::new(
            point![0.0, 0.0, 0.0],
            point![0.5, 0.0, 0.0],
            point![0.0, 2.0, 0.0],
        );
        self.renderer.draw_triangle(triangle, None, Some(RED));

        self.renderer.end_drawing(PRUSSIAN_BLUE, None);
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
