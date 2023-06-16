#![allow(dead_code)]
#![allow(unused_imports)]

use crate::color::Color;
use crate::glyph_atlas::*;
use crate::program::*;
use crate::shapes::*;
use image::{load_from_memory_with_format, ImageFormat};
use image::{DynamicImage, EncodableLayout};
use nalgebra::{Matrix4, Vector2};
use std::collections::HashMap;
use std::{mem::size_of, num::NonZeroU32};

use camera::*;
use glow::{HasContext, NativeTexture};

const PRIMITIVE_VERT_SRC: &str = include_str!("../shaders/primitive.vert");
const PRIMITIVE_FRAG_SRC: &str = include_str!("../shaders/primitive.frag");
const SCREEN_RECT_VERT_SRC: &str =
    include_str!("../shaders/screen_rect.vert");
const MAX_N_VERTICES: usize = 1 << 15;
const MAX_N_PROGRAMS: usize = 16;
const MAX_N_TEXTURES: usize = 16;
const MAX_N_BATCHES: usize = MAX_N_TEXTURES;

impl Program {
    fn new_gl(gl: &glow::Context, vert_src: &str, frag_src: &str) -> Self {
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

        Self::new(program.0.get())
    }

    fn to_glow(&self) -> glow::NativeProgram {
        glow::NativeProgram(NonZeroU32::new(self.idx).unwrap())
    }

    fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.to_glow()));
        }
    }

    fn set_arg_uniforms(&self, gl: &glow::Context) {
        use ProgramArg::*;
        let program = self.to_glow();

        unsafe {
            for (name, arg) in self.args.iter() {
                let loc = gl.get_uniform_location(program, name).unwrap();
                match arg {
                    FloatArg(val) => {
                        gl.uniform_1_f32(Some(&loc), *val);
                    }
                    ColorArg(val) => {
                        gl.uniform_4_f32(
                            Some(&loc),
                            val.r,
                            val.g,
                            val.b,
                            val.a,
                        );
                    }
                }
            }
        }
    }

    fn set_uniform_1_i32(&self, gl: &glow::Context, name: &str, val: i32) {
        unsafe {
            gl.uniform_1_i32(
                gl.get_uniform_location(self.to_glow(), name).as_ref(),
                val,
            );
        }
    }

    fn set_uniform_1_u32(&self, gl: &glow::Context, name: &str, val: u32) {
        unsafe {
            gl.uniform_1_u32(
                gl.get_uniform_location(self.to_glow(), name).as_ref(),
                val,
            );
        }
    }

    fn set_uniform_matrix_4_f32(
        &self,
        gl: &glow::Context,
        name: &str,
        val: &[f32],
    ) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(self.to_glow(), name).as_ref(),
                false,
                val,
            );
        }
    }
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
    tex_idx: Option<usize>,
    transform: Matrix4<f32>,
}

pub enum Projection {
    ProjScreen,
    Proj2D(Camera2D),
}

impl BatchInfo {
    pub fn new(
        start: usize,
        tex_idx: Option<usize>,
        transform: Matrix4<f32>,
    ) -> Self {
        Self {
            start,
            count: 0,
            tex_idx,
            transform,
        }
    }
}

impl Default for BatchInfo {
    fn default() -> Self {
        Self {
            start: 0,
            count: 0,
            tex_idx: None,
            transform: Matrix4::identity(),
        }
    }
}

pub struct Renderer {
    window: sdl2::video::Window,
    window_size: (u32, u32),
    gl: glow::Context,
    program: Program,

    vao: glow::NativeVertexArray,
    positions_vbo: glow::NativeBuffer,
    texcoords_vbo: glow::NativeBuffer,
    colors_vbo: glow::NativeBuffer,

    postfx_buffer_size: (u32, u32),
    postfx_fbo: glow::NativeFramebuffer,
    postfx_tex: glow::Texture,

    n_textures: usize,
    textures: [u32; MAX_N_TEXTURES],

    positions: [f32; MAX_N_VERTICES * 3],
    texcoords: [f32; MAX_N_VERTICES * 2],
    colors: [f32; MAX_N_VERTICES * 4],

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

        video.gl_set_swap_interval(1).unwrap();

        // ---------------------------------------------------------------
        let program =
            Program::new_gl(&gl, PRIMITIVE_VERT_SRC, PRIMITIVE_FRAG_SRC);

        let vao;
        let positions_vbo;
        let texcoords_vbo;
        let colors_vbo;
        let postfx_tex;
        let postfx_fbo;
        let postfx_buffer_size = window_size;
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

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

            postfx_fbo = gl.create_framebuffer().unwrap();
            postfx_tex = create_texture(
                &gl,
                glow::RGBA32F as i32,
                postfx_buffer_size.0 as i32,
                postfx_buffer_size.1 as i32,
                glow::RGBA,
                glow::FLOAT,
                None,
                glow::NEAREST,
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(postfx_fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(postfx_tex),
                0,
            );

            #[cfg(not(target_os = "emscripten"))]
            gl.draw_buffer(glow::COLOR_ATTACHMENT0);

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
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

            postfx_buffer_size,
            postfx_fbo,
            postfx_tex,

            n_textures: 0,
            textures: [0; MAX_N_TEXTURES],

            positions: [0.0; MAX_N_VERTICES * 3],
            texcoords: [0.0; MAX_N_VERTICES * 2],
            colors: [0.0; MAX_N_VERTICES * 4],

            n_batches: 0,
            batch_infos: [BatchInfo::default(); MAX_N_BATCHES],
        }
    }

    pub fn load_program(
        &mut self,
        vert_src: &str,
        frag_src: &str,
    ) -> Program {
        Program::new_gl(&self.gl, vert_src, frag_src)
    }

    pub fn load_screen_rect_program(&mut self, frag_src: &str) -> Program {
        self.load_program(SCREEN_RECT_VERT_SRC, frag_src)
    }

    pub fn load_texture_from_pixel_bytes(
        &mut self,
        bytes: &[u8],
        width: u32,
        height: u32,
    ) -> usize {
        if self.n_textures == MAX_N_TEXTURES {
            panic!("Can't create more than {} texture", MAX_N_TEXTURES);
        }

        let n_components = bytes.len() as u32 / (width * height);
        let (format, internal_format, alignment) = match n_components {
            1 => {
                #[cfg(target_os = "emscripten")]
                {
                    (glow::ALPHA, glow::ALPHA, 1)
                }

                #[cfg(not(target_os = "emscripten"))]
                {
                    (glow::ALPHA, glow::RGBA, 1)
                }
            }
            4 => (glow::RGBA, glow::RGBA, 4),
            _ => {
                panic!(
                    "Can't load texture with {}-components pixel",
                    n_components
                )
            }
        };

        unsafe {
            self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, alignment);
        }

        let tex = create_texture(
            &self.gl,
            internal_format as i32,
            width as i32,
            height as i32,
            format,
            glow::UNSIGNED_BYTE,
            Some(bytes),
            glow::LINEAR,
        );

        let idx = self.n_textures;
        self.textures[idx] = tex.0.get();
        self.n_textures += 1;

        idx
    }

    pub fn load_texture_from_image_bytes(
        &mut self,
        bytes: &[u8],
    ) -> usize {
        let image = load_from_memory_with_format(bytes, ImageFormat::Png)
            .expect("Can't decode image bytes")
            .into_rgba8();

        self.load_texture_from_pixel_bytes(
            image.as_bytes(),
            image.width(),
            image.height(),
        )
    }

    pub fn load_texture_from_glyph_atlas(
        &mut self,
        atlas: &GlyphAtlas,
    ) -> usize {
        self.load_texture_from_pixel_bytes(
            &atlas.pixels,
            atlas.image_width,
            atlas.image_height,
        )
    }

    fn draw_vertex(
        &mut self,
        position: Vector2<f32>,
        texcoord: Option<Vector2<f32>>,
        color: Option<Color>,
    ) {
        let batch_info = &mut self
            .batch_infos
            .get_mut(self.n_batches - 1)
            .expect("You should start a new batch before drawing");
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
        rect: Rectangle,
        texcoords: Option<Rectangle>,
        color: Option<Color>,
    ) {
        let positions = rect.to_triangles();
        let texcoords = if let Some(texcoords) = texcoords {
            texcoords.to_triangles().map(|t| Some(t))
        } else {
            [None, None]
        };

        self.draw_triangle(positions[0], texcoords[0], color);
        self.draw_triangle(positions[1], texcoords[1], color);
    }

    pub fn draw_glyph(&mut self, glyph: Glyph, color: Option<Color>) {
        self.draw_rect(glyph.rect, Some(glyph.texcoords), color);
    }

    pub fn start_new_batch(
        &mut self,
        proj: Projection,
        texture: Option<usize>,
    ) {
        use Projection::*;

        if self.n_batches == MAX_N_BATCHES {
            panic!("Maximum number of batches ({}) has been reached, can't start the new one. Call `end_drawing` to render all batches before starting the new one", self.n_batches);
        }

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

    pub fn end_drawing(
        &mut self,
        clear_color: Color,
        postfx_program: Option<&Program>,
    ) {
        unsafe {
            if postfx_program.is_some() {
                self.gl.bind_framebuffer(
                    glow::FRAMEBUFFER,
                    Some(self.postfx_fbo),
                );
                self.gl.viewport(
                    0,
                    0,
                    self.postfx_buffer_size.0 as i32,
                    self.postfx_buffer_size.1 as i32,
                );
            } else {
                self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                self.gl.viewport(
                    0,
                    0,
                    self.window_size.0 as i32,
                    self.window_size.1 as i32,
                );
            }

            self.gl.clear_color(
                clear_color.r,
                clear_color.g,
                clear_color.b,
                clear_color.a,
            );
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.program.bind(&self.gl);
            self.gl.bind_vertex_array(Some(self.vao));

            for i_batch in 0..self.n_batches {
                let batch_info = self.batch_infos[i_batch];
                let start = batch_info.start;
                let count = batch_info.count;
                let transform = batch_info.transform;
                let tex_idx = batch_info.tex_idx;

                buffer_sub_data(
                    &self.gl,
                    self.positions_vbo,
                    &self.positions[start * 3..(start + count) * 3],
                );
                buffer_sub_data(
                    &self.gl,
                    self.texcoords_vbo,
                    &self.texcoords[start * 2..(start + count) * 2],
                );
                buffer_sub_data(
                    &self.gl,
                    self.colors_vbo,
                    &self.colors[start * 4..(start + count) * 4],
                );

                self.program.set_uniform_matrix_4_f32(
                    &self.gl,
                    "u_transform",
                    transform.as_slice(),
                );

                let mut use_tex = 0;
                if let Some(tex) = tex_idx {
                    let tex = self.textures[tex];
                    self.gl.bind_texture(
                        glow::TEXTURE_2D,
                        Some(glow::NativeTexture(
                            NonZeroU32::new(tex).unwrap(),
                        )),
                    );
                    self.program.set_uniform_1_i32(&self.gl, "u_tex", 0);

                    use_tex = 1;
                }
                self.program.set_uniform_1_u32(
                    &self.gl,
                    "u_use_tex",
                    use_tex,
                );

                self.gl.draw_arrays(glow::TRIANGLES, 0, count as i32);
            }

            if let Some(program) = postfx_program {
                program.bind(&self.gl);
                program.set_arg_uniforms(&self.gl);
                program.set_uniform_1_i32(&self.gl, "u_tex", 0);

                self.gl.active_texture(glow::TEXTURE0 + 0);
                self.gl
                    .bind_texture(glow::TEXTURE_2D, Some(self.postfx_tex));

                self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                self.gl.viewport(
                    0,
                    0,
                    self.window_size.0 as i32,
                    self.window_size.1 as i32,
                );

                self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            }
        }

        self.n_batches = 0;
    }

    pub fn swap_window(&self) {
        self.window.gl_swap_window();
    }
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

fn buffer_sub_data<T>(
    gl: &glow::Context,
    vbo: glow::NativeBuffer,
    data: &[T],
) {
    unsafe {
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_sub_data_u8_slice(
            glow::ARRAY_BUFFER,
            0,
            cast_slice_to_u8(data),
        );
    }
}

fn cast_slice_to_u8<T>(slice: &[T]) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * core::mem::size_of::<T>(),
        )
    }
}
