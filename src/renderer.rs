#![allow(dead_code)]
#![allow(unused_imports)]

use crate::shapes::*;
use image::{DynamicImage, EncodableLayout};
use nalgebra::{Matrix4, Vector2};
use std::{mem::size_of, num::NonZeroU32};

use camera::*;
use color::*;
use glow::{HasContext, NativeTexture};

const PRIMITIVE_VERT_SRC: &str = include_str!("../shaders/primitive.vert");
const PRIMITIVE_FRAG_SRC: &str = include_str!("../shaders/primitive.frag");
const MAX_N_VERTICES: usize = 1 << 10;
const MAX_N_TEXTURES: usize = 16;
const MAX_N_BATCHES: usize = MAX_N_TEXTURES;

pub mod color {
    #[derive(Clone, Copy)]
    pub struct Color {
        pub r: f32,
        pub g: f32,
        pub b: f32,
        pub a: f32,
    }

    impl Color {
        pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
            Self { r, g, b, a }
        }

        pub fn as_arr(&self) -> [f32; 4] {
            [self.r, self.g, self.b, self.a]
        }
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

#[derive(Clone, Copy)]
struct BatchInfo {
    start: usize,
    count: usize,
    texture: Option<usize>,
    transform: Matrix4<f32>,
}

pub enum Projection {
    ProjScreen,
    Proj2D(Camera2D),
}

impl BatchInfo {
    pub fn new(
        start: usize,
        texture: Option<usize>,
        transform: Matrix4<f32>,
    ) -> Self {
        Self {
            start,
            count: 0,
            texture,
            transform,
        }
    }
}

impl Default for BatchInfo {
    fn default() -> Self {
        Self {
            start: 0,
            count: 0,
            texture: None,
            transform: Matrix4::identity(),
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
    texcoords_vbo: glow::NativeBuffer,
    colors_vbo: glow::NativeBuffer,
    use_tex_vbo: glow::NativeBuffer,

    n_textures: usize,
    textures: [u32; MAX_N_TEXTURES],

    positions: [f32; MAX_N_VERTICES * 3],
    texcoords: [f32; MAX_N_TEXTURES * 2],
    colors: [f32; MAX_N_VERTICES * 4],
    use_tex: [u32; MAX_N_VERTICES],

    n_batches: usize,
    batch_infos: [BatchInfo; MAX_N_BATCHES],
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

        let gl_profile;
        let gl_major_version;
        let gl_minor_version;
        #[cfg(target_os = "emscripten")]
        {
            gl_profile = sdl2::video::GLProfile::GLES;
            gl_major_version = 3;
            gl_minor_version = 0;
        }

        #[cfg(not(target_os = "emscripten"))]
        {
            gl_profile = sdl2::video::GLProfile::Core;
            gl_major_version = 4;
            gl_minor_version = 6;
        }

        gl_attr.set_context_profile(gl_profile);
        gl_attr.set_context_major_version(gl_major_version);
        gl_attr.set_context_minor_version(gl_minor_version);

        let gl_context = window.gl_create_context().unwrap();
        window.gl_make_current(&gl_context).unwrap();
        Box::leak(Box::new(gl_context));

        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                video.gl_get_proc_address(s) as *const _
            })
        };

        // video.gl_set_swap_interval(1).unwrap();

        // ---------------------------------------------------------------
        let program =
            create_program(&gl, PRIMITIVE_VERT_SRC, PRIMITIVE_FRAG_SRC);

        let vao;
        let positions_vbo;
        let texcoords_vbo;
        let colors_vbo;
        let use_tex_vbo;
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

            texcoords_vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(texcoords_vbo));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size_of::<f32>() * 2 * MAX_N_VERTICES) as i32,
                glow::DYNAMIC_DRAW,
            );
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);

            colors_vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(colors_vbo));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size_of::<f32>() * 4 * MAX_N_VERTICES) as i32,
                glow::DYNAMIC_DRAW,
            );
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 4, glow::FLOAT, false, 0, 0);

            use_tex_vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(use_tex_vbo));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size_of::<u32>() * 1 * MAX_N_VERTICES) as i32,
                glow::DYNAMIC_DRAW,
            );
            gl.enable_vertex_attrib_array(3);
            gl.vertex_attrib_pointer_i32(3, 1, glow::UNSIGNED_INT, 0, 0);
        }

        Self {
            window,
            window_size,
            gl,
            program,

            vao,
            positions_vbo,
            texcoords_vbo,
            colors_vbo,
            use_tex_vbo,

            n_textures: 0,
            textures: [0; MAX_N_TEXTURES],

            positions: [0.0; MAX_N_VERTICES * 3],
            texcoords: [0.0; MAX_N_TEXTURES * 2],
            colors: [0.0; MAX_N_VERTICES * 4],
            use_tex: [0; MAX_N_VERTICES],

            n_batches: 0,
            batch_infos: [BatchInfo::default(); MAX_N_BATCHES],
        }
    }

    pub fn load_texture_from_image(
        &mut self,
        image: DynamicImage,
    ) -> usize {
        let image = image.into_rgba8();
        let tex = create_texture(
            &self.gl,
            glow::RGBA as i32,
            image.width() as i32,
            image.height() as i32,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(&image.as_bytes()),
            glow::LINEAR,
        );

        let idx = self.n_textures;
        self.textures[idx] = tex.0.get();
        self.n_textures += 1;

        return idx;
    }

    pub fn clear_color(&self, color: Color) {
        unsafe {
            self.gl.clear_color(color.r, color.g, color.b, color.a);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    fn draw_vertex(
        &mut self,
        position: Vector2<f32>,
        texcoord: Option<Vector2<f32>>,
        color: Option<Color>,
    ) {
        if self.n_batches == 0 {
            panic!("You should start a new batch before drawing!")
        }
        let batch_info = &mut self.batch_infos[self.n_batches - 1];
        let idx = batch_info.start + batch_info.count;
        self.positions[idx * 3 + 0] = position.x;
        self.positions[idx * 3 + 1] = position.y;
        self.positions[idx * 3 + 2] = 0.0;

        if let Some(color) = color {
            self.colors[idx * 4 + 0] = color.r;
            self.colors[idx * 4 + 1] = color.g;
            self.colors[idx * 4 + 2] = color.b;
            self.colors[idx * 4 + 3] = color.a;
        } else {
            self.colors[idx * 4 + 0] = 0.0;
            self.colors[idx * 4 + 1] = 0.0;
            self.colors[idx * 4 + 2] = 0.0;
            self.colors[idx * 4 + 3] = 0.0;
        }

        if let Some(texcoord) = texcoord {
            self.texcoords[idx * 2 + 0] = texcoord.x;
            self.texcoords[idx * 2 + 1] = texcoord.y;
            self.use_tex[idx] = 1;
        } else {
            self.use_tex[idx] = 0;
        }

        batch_info.count += 1;
    }

    pub fn draw_triangle(
        &mut self,
        triangle: Triangle,
        texcoords: Option<Triangle>,
        color: Option<Color>,
    ) {
        if let Some(texcoords) = texcoords {
            self.draw_vertex(triangle.a, Some(texcoords.a), color);
            self.draw_vertex(triangle.b, Some(texcoords.b), color);
            self.draw_vertex(triangle.c, Some(texcoords.c), color);
        } else {
            self.draw_vertex(triangle.a, None, color);
            self.draw_vertex(triangle.b, None, color);
            self.draw_vertex(triangle.c, None, color);
        }
    }

    pub fn draw_rect(
        &mut self,
        rect: Rect,
        texcoords: Option<Rect>,
        color: Option<Color>,
    ) {
        let positions = rect.to_triangles();
        let texcoords = if let Some(texcoords) = texcoords {
            texcoords.to_some_triangles()
        } else {
            [None, None]
        };

        self.draw_triangle(positions[0], texcoords[0], color);
        self.draw_triangle(positions[1], texcoords[1], color);
    }

    pub fn start_new_batch(
        &mut self,
        proj: Projection,
        texture: Option<usize>,
    ) {
        use Projection::*;

        self.window_size = self.window.size();
        let transform = match proj {
            ProjScreen => Matrix4::new_orthographic(
                0.0,
                self.window_size.0 as f32,
                0.0,
                self.window_size.1 as f32,
                0.0,
                1.0,
            ),
            Proj2D(camera) => {
                let view = camera.get_view();
                let projection = Matrix4::new_orthographic(
                    self.window_size.0 as f32 / -2.0,
                    self.window_size.0 as f32 / 2.0,
                    self.window_size.1 as f32 / -2.0,
                    self.window_size.1 as f32 / 2.0,
                    0.0,
                    1.0,
                );
                projection * view
            }
        };

        let start = if self.n_batches == 0 {
            0
        } else {
            let prev_batch_info = self.batch_infos[self.n_batches - 1];

            prev_batch_info.start + prev_batch_info.count
        };

        let batch_info = BatchInfo::new(start, texture, transform);
        self.batch_infos[self.n_batches] = batch_info;
        self.n_batches += 1;
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

            for i_batch in 0..self.n_batches {
                let batch_info = self.batch_infos[i_batch];
                let start = batch_info.start;
                let count = batch_info.count;
                let transform = batch_info.transform;
                let texture = batch_info.texture;

                self.gl.bind_buffer(
                    glow::ARRAY_BUFFER,
                    Some(self.positions_vbo),
                );
                self.gl.buffer_sub_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    0,
                    cast_slice_to_u8(
                        &self.positions[start * 3..(start + count) * 3],
                    ),
                );

                self.gl.bind_buffer(
                    glow::ARRAY_BUFFER,
                    Some(self.texcoords_vbo),
                );
                self.gl.buffer_sub_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    0,
                    cast_slice_to_u8(
                        &self.texcoords[start * 2..(start + count) * 2],
                    ),
                );

                self.gl.bind_buffer(
                    glow::ARRAY_BUFFER,
                    Some(self.colors_vbo),
                );
                self.gl.buffer_sub_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    0,
                    cast_slice_to_u8(
                        &self.colors[start * 4..(start + count) * 4],
                    ),
                );

                self.gl.bind_buffer(
                    glow::ARRAY_BUFFER,
                    Some(self.use_tex_vbo),
                );
                self.gl.buffer_sub_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    0,
                    cast_slice_to_u8(
                        &self.use_tex[start * 1..(start + count) * 1],
                    ),
                );

                self.gl.uniform_matrix_4_f32_slice(
                    self.gl
                        .get_uniform_location(self.program, "u_transform")
                        .as_ref(),
                    false,
                    transform.as_slice(),
                );

                if let Some(tex) = texture {
                    let tex = self.textures[tex];
                    self.gl.bind_texture(
                        0,
                        Some(glow::NativeTexture(
                            NonZeroU32::new(tex).unwrap(),
                        )),
                    );
                    self.gl.uniform_1_i32(
                        self.gl
                            .get_uniform_location(self.program, "u_tex")
                            .as_ref(),
                        0,
                    );
                }

                self.gl.use_program(Some(self.program));
                self.gl.draw_arrays(glow::TRIANGLES, 0, count as i32);
            }
        }

        self.n_batches = 0;
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

    #[cfg(target_os = "emscripten")]
    let header = r#"#version 300 es
        #ifdef GL_ES
        precision highp float;
        #endif
    "#;

    #[cfg(not(target_os = "emscripten"))]
    let header = r#"#version 460 core
    "#;

    unsafe {
        program = gl.create_program().expect("Cannot create program");

        let shaders_src = [
            (glow::VERTEX_SHADER, header.to_owned() + vert_src),
            (glow::FRAGMENT_SHADER, header.to_owned() + frag_src),
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

fn create_texture(
    gl: &glow::Context,
    internal_format: i32,
    width: i32,
    height: i32,
    format: u32,
    ty: u32,
    pixels: Option<&[u8]>,
    filter: u32,
) -> glow::Texture {
    let tex;

    unsafe {
        tex = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            internal_format,
            width,
            height,
            0,
            format,
            ty,
            pixels,
        );

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            filter as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            filter as i32,
        );
    }

    tex
}

fn cast_slice_to_u8<T>(slice: &[T]) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * core::mem::size_of::<T>(),
        )
    }
}
