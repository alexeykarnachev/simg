#![allow(dead_code)]
#![allow(unused_imports)]

use nalgebra::{Matrix4, Vector2};
use std::mem::size_of;

use camera::*;
use color::*;
use glow::HasContext;

const PRIMITIVE_VERT_SRC: &str = include_str!("../shaders/primitive.vert");
const PRIMITIVE_FRAG_SRC: &str = include_str!("../shaders/primitive.frag");
const MAX_N_VERTICES: usize = 1 << 14;

pub mod color {
    #[derive(Clone, Copy)]
    pub struct Color {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
    }

    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
}

pub mod camera {
    use nalgebra::{Matrix4, Vector2, Vector3};

    #[derive(Clone, Copy)]
    pub struct Camera2D {
        pub position: Vector2<f32>,
        pub rotation: f32,
        pub zoom: f32,
    }

    impl Camera2D {
        pub fn new(position: Vector2<f32>) -> Self {
            Self {
                position,
                rotation: 0.0,
                zoom: 1.0,
            }
        }

        pub fn get_view(&self) -> Matrix4<f32> {
            let mut scale = Matrix4::identity();
            scale[(0, 0)] = self.zoom;
            scale[(1, 1)] = self.zoom;

            let mut translation = Matrix4::identity();
            translation[(0, 3)] = -self.position.x;
            translation[(1, 3)] = -self.position.y;

            let rotation = Matrix4::new_rotation(Vector3::new(
                0.0,
                0.0,
                -self.rotation,
            ));

            rotation * scale * translation
        }
    }
}

pub struct Renderer {
    window: sdl2::video::Window,
    window_size: (u32, u32),
    gl: glow::Context,
    program: glow::NativeProgram,

    vao: glow::NativeVertexArray,
    positions_vbo: glow::NativeBuffer,
    colors_vbo: glow::NativeBuffer,

    n_vertices: usize,
    positions: [f32; MAX_N_VERTICES * 3],
    colors: [f32; MAX_N_VERTICES * 4],

    transform: Matrix4<f32>,
}

impl Renderer {
    pub fn new(
        sdl2: &sdl2::Sdl,
        window_name: &str,
        window_width: u32,
        window_height: u32,
    ) -> Self {
        // ---------------------------------------------------------------
        // Initialize sdl2 window with OpenGL context
        let video = sdl2.video().unwrap();
        let window = video
            .window(window_name, window_width, window_height)
            .opengl()
            .resizable()
            .build()
            .unwrap();
        let window_size = window.size();

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 6);

        Box::leak(Box::new(window.gl_create_context().unwrap()));
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                video.gl_get_proc_address(s) as *const _
            })
        };

        video.gl_set_swap_interval(1).unwrap();

        // ---------------------------------------------------------------
        let program =
            create_program(&gl, PRIMITIVE_VERT_SRC, PRIMITIVE_FRAG_SRC);

        let vao;
        let positions_vbo;
        let colors_vbo;
        unsafe {
            vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            positions_vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(positions_vbo));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size_of::<f32>() * 3 * MAX_N_VERTICES) as i32,
                glow::DYNAMIC_DRAW,
            );
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

            colors_vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(colors_vbo));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size_of::<f32>() * 4 * MAX_N_VERTICES) as i32,
                glow::DYNAMIC_DRAW,
            );
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, 0, 0);
        }

        Self {
            window,
            window_size,
            gl,
            program,

            vao,
            positions_vbo,
            colors_vbo,

            n_vertices: 0,
            positions: [0.0; MAX_N_VERTICES * 3],
            colors: [0.0; MAX_N_VERTICES * 4],

            transform: Matrix4::identity(),
        }
    }

    pub fn clear_color(&self, color: Color) {
        unsafe {
            self.gl.clear_color(color.r, color.g, color.b, color.a);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    fn draw_vertex_2d(&mut self, position: Vector2<f32>, color: Color) {
        let idx = self.n_vertices;
        self.positions[idx * 3 + 0] = position.x;
        self.positions[idx * 3 + 1] = position.y;
        self.positions[idx * 3 + 2] = 0.0;

        self.colors[idx * 4 + 0] = color.r;
        self.colors[idx * 4 + 1] = color.g;
        self.colors[idx * 4 + 2] = color.b;
        self.colors[idx * 4 + 3] = color.a;

        self.n_vertices += 1;
    }

    pub fn draw_triangle(
        &mut self,
        a: Vector2<f32>,
        b: Vector2<f32>,
        c: Vector2<f32>,
        color: Color,
    ) {
        self.draw_vertex_2d(a, color);
        self.draw_vertex_2d(b, color);
        self.draw_vertex_2d(c, color);
    }

    pub fn begin_screen_drawing(&mut self) {
        self.window_size = self.window.size();
        self.transform = Matrix4::new_orthographic(
            0.0,
            self.window_size.0 as f32,
            0.0,
            self.window_size.1 as f32,
            0.0,
            1.0,
        );
    }

    pub fn begin_2d_drawing(&mut self, camera: Camera2D) {
        self.window_size = self.window.size();
        let view = camera.get_view();
        let projection = Matrix4::new_orthographic(
            self.window_size.0 as f32 / -2.0,
            self.window_size.0 as f32 / 2.0,
            self.window_size.1 as f32 / -2.0,
            self.window_size.1 as f32 / 2.0,
            0.0,
            1.0,
        );
        self.transform = projection * view;
    }

    pub fn end_drawing(&mut self) {
        unsafe {
            self.gl.viewport(
                0,
                0,
                self.window_size.0 as i32,
                self.window_size.1 as i32,
            );
            self.gl.bind_vertex_array(Some(self.vao));

            self.gl
                .bind_buffer(glow::ARRAY_BUFFER, Some(self.positions_vbo));
            self.gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                cast_slice_to_u8(&self.positions[0..self.n_vertices * 3]),
            );

            self.gl
                .bind_buffer(glow::ARRAY_BUFFER, Some(self.colors_vbo));
            self.gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                cast_slice_to_u8(&self.colors[0..self.n_vertices * 4]),
            );

            self.gl.uniform_matrix_4_f32_slice(
                self.gl
                    .get_uniform_location(self.program, "u_transform")
                    .as_ref(),
                false,
                self.transform.as_slice(),
            );

            self.gl.use_program(Some(self.program));
            self.gl.draw_arrays(
                glow::TRIANGLES,
                0,
                self.n_vertices as i32,
            );
        }

        self.n_vertices = 0;
    }

    pub fn swap_window(&self) {
        self.window.gl_swap_window();
    }
}

fn create_program(
    gl: &glow::Context,
    vert_src: &str,
    frag_src: &str,
) -> glow::NativeProgram {
    let program;

    unsafe {
        program = gl.create_program().expect("Cannot create program");

        let shaders_src = [
            (glow::VERTEX_SHADER, vert_src),
            (glow::FRAGMENT_SHADER, frag_src),
        ];

        let mut shaders = Vec::with_capacity(shaders_src.len());
        for (shader_type, shader_src) in shaders_src.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, shader_src);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
    }

    program
}

fn cast_slice_to_u8<T>(slice: &[T]) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * core::mem::size_of::<T>(),
        )
    }
}
