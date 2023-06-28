use image::ImageFormat;
use nalgebra::Matrix4;
use nalgebra::{point, vector, Point3};
use sdl2::mouse::MouseButton;
use simg::color::*;
use simg::common::*;
use simg::input::Input;
use simg::renderer::Renderer;
use simg::shapes::*;
use simg::vertex_buffer::*;

const MSAA: i32 = 16;

const GAME_DT: f32 = 0.005;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;

const DOG_OBJ: &[u8] = include_bytes!("./assets/basic_3d/dog/dog.obj");
const DOG_TEX: &[u8] = include_bytes!("./assets/basic_3d/dog/color.png");

struct Camera {
    pub target: Point3<f32>,
    pub pitch: f32,
    pub yaw: f32,
    pub distance: f32,
    pub fovy: f32,
}

impl Camera {
    pub fn new(
        target: Point3<f32>,
        pitch: f32,
        yaw: f32,
        distance: f32,
        fovy: f32,
    ) -> Self {
        Self { target, pitch, yaw, distance, fovy }
    }

    pub fn update(&mut self, input: &Input) {
        if input.mouse_buttons.is_pressed(MouseButton::Left) {
            let dx = input.mouse_xrel as f32;
            let dy = input.mouse_yrel as f32;
            self.pitch += dy;
            self.yaw += dx;

            self.pitch = self.pitch.signum() * self.pitch.abs().min(85.0);
        }

        self.distance -= input.mouse_wheel as f32;
    }

    pub fn get_proj(&self) -> Projection {
        let q = nalgebra::UnitQuaternion::from_euler_angles(
            -self.pitch.to_radians(),
            self.yaw.to_radians(),
            0.0,
        );
        let eye = point![0.0, 0.0, self.distance];
        let eye = (q * (eye - self.target)) + self.target.coords;

        Projection::Proj3D {
            eye: eye.into(),
            target: self.target,
            fovy: self.fovy,
        }
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

    camera: Camera,

    vb_gpu: usize,
    tex: Texture,
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

        let camera =
            Camera::new(point![0.0, 0.0, 0.0], 45.0, -45.0, 10.0, 60.0);

        let vb_gpu = renderer.load_vertex_buffer_from_cpu(
            &VertexBufferCPU::from_obj_bytes(DOG_OBJ),
        );
        let tex = renderer
            .load_texture_from_image_bytes(DOG_TEX, ImageFormat::Png);

        Self {
            dt: 0.0,
            time: 0.0,
            prev_ticks: timer.ticks(),
            timer,
            should_quit: false,
            sdl2,
            input,
            renderer,
            camera,
            vb_gpu,
            tex,
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
        self.camera.update(&self.input);
    }

    fn update_renderer(&mut self) {
        self.renderer.set_proj(self.camera.get_proj());
        self.renderer.set_tex(self.tex, false);

        let triangle = Triangle::new(
            point![0.0, 0.0, 0.0],
            point![0.5, 0.0, 0.0],
            point![0.0, 2.0, 0.0],
        );
        self.renderer.draw_triangle(triangle, None, Some(RED));

        let triangle = Triangle::new(
            point![10.0, 0.0, 0.0],
            point![10.5, 0.0, 0.0],
            point![10.0, 2.0, 0.0],
        );
        self.renderer.draw_triangle(triangle, None, Some(GREEN));

        let transform = Transformation::new(
            vector![10.0, self.time.sin() * 8.0, self.time.cos() * 8.0],
            vector![1.0, 1.0, 1.0],
            vector![0.0, 0.0, self.time / 2.0],
        );
        self.renderer
            .draw_vertex_buffer(self.vb_gpu, Some(transform));

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
