#![allow(dead_code)]
#![allow(unused_imports)]

use crate::camera::*;
use crate::color::*;
use crate::glyph_atlas::*;
use crate::mesh::Mesh;
use crate::program::*;
use crate::shapes::*;
use image::{
    load_from_memory_with_format, DynamicImage, EncodableLayout,
    ImageFormat,
};
use nalgebra::{Matrix4, Point2, Point3, Vector2, Vector3};
use std::{collections::HashMap, mem::size_of, num::NonZeroU32};

use glow::{HasContext, NativeTexture};

const PRIMITIVE_VERT_SRC: &str = include_str!("../shaders/primitive.vert");
const PRIMITIVE_FRAG_SRC: &str = include_str!("../shaders/primitive.frag");
const SCREEN_RECT_VERT_SRC: &str =
    include_str!("../shaders/screen_rect.vert");
const MAX_N_VERTICES: usize = 1 << 15;
const MAX_N_PROGRAMS: usize = 16;
const MAX_N_TEXTURES: usize = 16;
const MAX_N_VERTEX_BUFFERS: usize = 128;
const MAX_N_BATCHES: usize = MAX_N_TEXTURES;

#[derive(Copy, Clone, PartialEq)]
struct VertexBufferGL {
    vao: glow::NativeVertexArray,
    positions_vbo: glow::NativeBuffer,
    colors_vbo: glow::NativeBuffer,
    texcoords_vbo: glow::NativeBuffer,
    has_tex_vbo: glow::NativeBuffer,

    indices_vbo: Option<glow::NativeBuffer>,

    n_vertices: usize,
    n_indices: usize,
}

impl Default for VertexBufferGL {
    fn default() -> Self {
        Self {
            vao: glow::NativeVertexArray(NonZeroU32::new(1).unwrap()),
            positions_vbo: glow::NativeBuffer(NonZeroU32::new(1).unwrap()),
            colors_vbo: glow::NativeBuffer(NonZeroU32::new(1).unwrap()),
            texcoords_vbo: glow::NativeBuffer(NonZeroU32::new(1).unwrap()),
            has_tex_vbo: glow::NativeBuffer(NonZeroU32::new(1).unwrap()),
            indices_vbo: None,
            n_vertices: 0,
            n_indices: 0,
        }
    }
}

impl VertexBufferGL {
    pub fn new(gl: &glow::Context, n_vertices: usize) -> Self {
        let vao;
        let positions_vbo;
        let texcoords_vbo;
        let colors_vbo;
        let has_tex_vbo;
        let indices_vbo = None;
        unsafe {
            vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            positions_vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(positions_vbo));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size_of::<f32>() * 3 * n_vertices) as i32,
                glow::DYNAMIC_DRAW,
            );
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

            texcoords_vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(texcoords_vbo));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size_of::<f32>() * 2 * n_vertices) as i32,
                glow::DYNAMIC_DRAW,
            );
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 0, 0);

            colors_vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(colors_vbo));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size_of::<f32>() * 4 * n_vertices) as i32,
                glow::DYNAMIC_DRAW,
            );
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 4, glow::FLOAT, false, 0, 0);

            has_tex_vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(has_tex_vbo));
            gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                (size_of::<u8>() * 1 * n_vertices) as i32,
                glow::DYNAMIC_DRAW,
            );
            gl.enable_vertex_attrib_array(3);
            gl.vertex_attrib_pointer_i32(3, 1, glow::UNSIGNED_BYTE, 0, 0);
        };

        Self {
            vao,
            positions_vbo,
            texcoords_vbo,
            colors_vbo,
            has_tex_vbo,
            indices_vbo,
            n_vertices,
            n_indices: 0,
        }
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
        }
    }

    pub fn set_positions(&mut self, gl: &glow::Context, data: &[f32]) {
        buffer_sub_data(gl, glow::ARRAY_BUFFER, self.positions_vbo, data);
    }

    pub fn set_texcoords(&mut self, gl: &glow::Context, data: &[f32]) {
        buffer_sub_data(gl, glow::ARRAY_BUFFER, self.texcoords_vbo, data);
    }

    pub fn set_colors(&mut self, gl: &glow::Context, data: &[f32]) {
        buffer_sub_data(gl, glow::ARRAY_BUFFER, self.colors_vbo, data);
    }

    pub fn set_has_tex(&mut self, gl: &glow::Context, data: &[u8]) {
        buffer_sub_data(gl, glow::ARRAY_BUFFER, self.has_tex_vbo, data);
    }

    pub fn set_indices(&mut self, gl: &glow::Context, data: &[u16]) {
        if self.indices_vbo.is_none() {
            unsafe {
                let vbo = gl.create_buffer().unwrap();
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(vbo));
                gl.buffer_data_size(
                    glow::ELEMENT_ARRAY_BUFFER,
                    (size_of::<u16>() * data.len()) as i32,
                    glow::DYNAMIC_DRAW,
                );
                self.indices_vbo = Some(vbo);
                self.n_indices = data.len();
            }
        }

        if let Some(vbo) = self.indices_vbo {
            buffer_sub_data(gl, glow::ELEMENT_ARRAY_BUFFER, vbo, data);
        }
    }
}

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
                let loc = gl.get_uniform_location(program, name).expect(
                    &format!("Program should have {} uniform", name),
                );
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

impl Texture {
    fn new_gl(
        gl: &glow::Context,
        data: Option<&[u8]>,
        internal_format: u32,
        width: u32,
        height: u32,
        format: u32,
        data_type: u32,
        filter: u32,
    ) -> Self {
        let tex;

        unsafe {
            tex = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                internal_format as i32,
                width as i32,
                height as i32,
                0,
                format,
                data_type,
                data,
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

        Self::new(tex.0.get(), width, height)
    }

    fn to_glow(&self) -> glow::Texture {
        glow::NativeTexture(NonZeroU32::new(self.idx).unwrap())
    }

    fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.to_glow()));
        }
    }
}

#[derive(Clone, Copy)]
struct BatchInfo {
    vertex_buffer_idx: usize,
    start: usize,
    count: usize,
    tex: Option<Texture>,
    proj: Projection,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Projection {
    ProjScreen,
    Proj2D {
        eye: Point2<f32>,
        zoom: f32,
        rotation: f32,
    },
    Proj3D {
        eye: Point3<f32>,
        target: Point3<f32>,
        fovy: f32,
    },
}

impl BatchInfo {
    pub fn new(
        vertex_buffer_idx: usize,
        start: usize,
        tex: Option<Texture>,
        proj: Projection,
    ) -> Self {
        Self {
            vertex_buffer_idx,
            start,
            count: 0,
            tex,
            proj,
        }
    }

    pub fn get_next(&self) -> Self {
        let mut batch = *self;
        batch.count = 0;
        batch.start = self.start + self.count;

        batch
    }
}

impl Default for BatchInfo {
    fn default() -> Self {
        Self {
            vertex_buffer_idx: 0,
            start: 0,
            count: 0,
            tex: None,
            proj: Projection::ProjScreen,
        }
    }
}

pub struct Renderer {
    window: sdl2::video::Window,
    window_size: (u32, u32),
    gl: glow::Context,
    program: Program,

    n_vertex_buffers: usize,
    vertex_buffers: [VertexBufferGL; MAX_N_VERTEX_BUFFERS],

    ms_fbo: Option<glow::NativeFramebuffer>,

    postfx_fbo: glow::NativeFramebuffer,
    postfx_tex: Texture,

    positions: [f32; MAX_N_VERTICES * 3],
    texcoords: [f32; MAX_N_VERTICES * 2],
    colors: [f32; MAX_N_VERTICES * 4],
    has_tex: [u8; MAX_N_VERTICES],

    curr_batch_idx: usize,
    batch_infos: [BatchInfo; MAX_N_BATCHES],
}

impl Renderer {
    pub fn new(
        sdl2: &sdl2::Sdl,
        window_name: &str,
        window_width: u32,
        window_height: u32,
        msaa: i32,
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
        // gl_attr.set_multisample_samples(N_MULTISAMPLES);

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

        let mut vertex_buffers =
            [VertexBufferGL::default(); MAX_N_VERTEX_BUFFERS];
        let mut ms_fbo = None;
        let postfx_tex;
        let postfx_fbo;
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            // -----------------------------------------------------------
            // Multisample buffer
            let n_samples = get_msaa_max_n_samples(&gl, msaa);
            if n_samples > 0 {
                ms_fbo = Some(gl.create_framebuffer().unwrap());
                let ms_rbo = gl.create_renderbuffer().unwrap();
                gl.bind_renderbuffer(glow::RENDERBUFFER, Some(ms_rbo));
                gl.renderbuffer_storage_multisample(
                    glow::RENDERBUFFER,
                    n_samples as i32,
                    glow::RGBA32F,
                    window_size.0 as i32,
                    window_size.1 as i32,
                );
                gl.bind_framebuffer(glow::FRAMEBUFFER, ms_fbo);
                gl.framebuffer_renderbuffer(
                    glow::FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0,
                    glow::RENDERBUFFER,
                    Some(ms_rbo),
                );
            }

            // -----------------------------------------------------------
            // Default vertex buffer (for batch rendering)
            vertex_buffers[0] = VertexBufferGL::new(&gl, MAX_N_VERTICES);

            // -----------------------------------------------------------
            // Postfx buffer
            postfx_fbo = gl.create_framebuffer().unwrap();
            postfx_tex = Texture::new_gl(
                &gl,
                None,
                glow::RGBA32F,
                window_size.0,
                window_size.1,
                glow::RGBA,
                glow::FLOAT,
                glow::NEAREST,
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(postfx_fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(postfx_tex.to_glow()),
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

            n_vertex_buffers: 1,
            vertex_buffers,

            ms_fbo,

            postfx_fbo,
            postfx_tex,

            positions: [0.0; MAX_N_VERTICES * 3],
            texcoords: [0.0; MAX_N_VERTICES * 2],
            colors: [0.0; MAX_N_VERTICES * 4],
            has_tex: [0; MAX_N_VERTICES],

            curr_batch_idx: 0,
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
    ) -> Texture {
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

        Texture::new_gl(
            &self.gl,
            Some(bytes),
            internal_format,
            width,
            height,
            format,
            glow::UNSIGNED_BYTE,
            glow::LINEAR,
        )
    }

    pub fn load_texture_from_image_bytes(
        &mut self,
        bytes: &[u8],
        format: ImageFormat,
    ) -> Texture {
        let image = load_from_memory_with_format(bytes, format)
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
    ) -> Texture {
        self.load_texture_from_pixel_bytes(
            &atlas.pixels,
            atlas.image_width,
            atlas.image_height,
        )
    }

    pub fn load_mesh(&mut self, mesh: &Mesh) -> usize {
        let mut vb = VertexBufferGL::new(&self.gl, mesh.n_vertices);

        if let Some(texcoords) = mesh.texcoords.as_ref() {
            vb.set_has_tex(&self.gl, &vec![1u8; mesh.n_vertices]);
            vb.set_texcoords(&self.gl, &texcoords);
        } else {
            vb.set_has_tex(&self.gl, &vec![0u8; mesh.n_vertices]);
            vb.set_texcoords(&self.gl, &vec![0.0; mesh.n_vertices * 2]);
        };

        vb.set_colors(&self.gl, &vec![0.0; mesh.n_vertices * 4]);
        vb.set_positions(&self.gl, &mesh.positions);
        vb.set_indices(&self.gl, &mesh.indices);

        let idx = self.n_vertex_buffers;
        self.vertex_buffers[idx] = vb;
        self.n_vertex_buffers += 1;

        idx
    }

    fn draw_vertex(
        &mut self,
        position: Point3<f32>,
        texcoord: Option<Point2<f32>>,
        color: Option<Color>,
    ) {
        let batch_info = &mut self.batch_infos[self.curr_batch_idx];
        let idx = batch_info.start + batch_info.count;
        self.positions[idx * 3 + 0] = position.x;
        self.positions[idx * 3 + 1] = position.y;
        self.positions[idx * 3 + 2] = position.z;

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
            self.has_tex[idx] = 1;
        } else {
            self.has_tex[idx] = 0;
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
            self.draw_vertex(triangle.a, Some(texcoords.a.xy()), color);
            self.draw_vertex(triangle.b, Some(texcoords.b.xy()), color);
            self.draw_vertex(triangle.c, Some(texcoords.c.xy()), color);
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
        let positions = rect.get_triangles();
        let texcoords = if let Some(texcoords) = texcoords {
            texcoords.get_triangles().map(|t| Some(t))
        } else {
            [None, None]
        };

        self.draw_triangle(positions[0], texcoords[0], color);
        self.draw_triangle(positions[1], texcoords[1], color);
    }

    pub fn draw_circle(
        &mut self,
        circle: Circle,
        texcoords: Option<Circle>,
        color: Option<Color>,
    ) {
        let positions = circle.to_triangles();
        let texcoords = if let Some(texcoords) = texcoords {
            texcoords.to_triangles().map(|t| Some(t))
        } else {
            [None; CIRCLE_N_TRIANGLES]
        };

        for i in 0..CIRCLE_N_TRIANGLES {
            self.draw_triangle(positions[i], texcoords[i], color);
        }
    }

    pub fn draw_glyph(&mut self, glyph: Glyph, color: Option<Color>) {
        self.draw_rect(glyph.rect, Some(glyph.texcoords), color);
    }

    pub fn draw_mesh(&mut self, mesh_idx: usize) {
        let mut batch_info =
            &mut self.batch_infos[self.get_new_batch_idx()];
        let vb = self.vertex_buffers[mesh_idx];

        batch_info.vertex_buffer_idx = mesh_idx;
        batch_info.count = vb.n_vertices;
    }

    pub fn set_proj(&mut self, proj: Projection) {
        if self.batch_infos[self.curr_batch_idx].proj != proj {
            self.batch_infos[self.get_new_batch_idx()].proj = proj;
        }
    }

    pub fn set_tex(&mut self, tex: Texture) {
        let curr_tex = self.batch_infos[self.curr_batch_idx].tex;
        if curr_tex.is_none() || curr_tex.is_some_and(|t| t != tex) {
            self.batch_infos[self.get_new_batch_idx()].tex = Some(tex);
        }
    }

    fn get_new_batch_idx(&mut self) -> usize {
        if self.batch_infos[self.curr_batch_idx].count == 0 {
            self.curr_batch_idx
        } else if self.curr_batch_idx + 1 < MAX_N_BATCHES {
            let curr_batch = self.batch_infos[self.curr_batch_idx];
            self.batch_infos[self.curr_batch_idx + 1] =
                curr_batch.get_next();
            self.curr_batch_idx += 1;
            self.curr_batch_idx
        } else {
            panic!("Maximum number of batches ({}) has been reached, can't start the new one. Call `end_drawing` to render all batches before starting the new one", self.curr_batch_idx + 1);
        }
    }

    pub fn reset_batches(&mut self) {
        self.curr_batch_idx = 0;
        self.batch_infos[0] = BatchInfo::default();
    }

    pub fn end_drawing(
        &mut self,
        clear_color: Color,
        postfx_program: Option<&Program>,
    ) {
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, None);

            // -----------------------------------------------------------
            // Draw scene to the multisample buffer
            let out_fbo = if let Some(ms_fbo) = self.ms_fbo {
                Some(ms_fbo)
            } else {
                Some(self.postfx_fbo)
            };

            self.gl.bind_framebuffer(glow::FRAMEBUFFER, out_fbo);
            self.gl.viewport(
                0,
                0,
                self.window_size.0 as i32,
                self.window_size.1 as i32,
            );

            self.gl.clear_color(
                clear_color.r,
                clear_color.g,
                clear_color.b,
                clear_color.a,
            );
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            self.program.bind(&self.gl);

            let mut curr_vb_idx = None;
            let mut curr_tex = None;

            for i_batch in 0..=self.curr_batch_idx {
                let batch_info = self.batch_infos[i_batch];
                let vb_idx = batch_info.vertex_buffer_idx;
                let proj = batch_info.proj;
                let transform =
                    get_transform_from_proj(proj, self.window_size);

                self.program.set_uniform_matrix_4_f32(
                    &self.gl,
                    "u_transform",
                    transform.as_slice(),
                );

                if curr_vb_idx.is_none()
                    || curr_vb_idx.is_some_and(|idx| idx != vb_idx)
                {
                    curr_vb_idx = Some(vb_idx);
                    self.vertex_buffers[vb_idx].bind(&self.gl);
                }

                // Update default vertex buffer
                if vb_idx == 0 {
                    let start = batch_info.start;
                    let count = batch_info.count;
                    let vb = &mut self.vertex_buffers[vb_idx];
                    vb.set_positions(
                        &self.gl,
                        &self.positions[start * 3..(start + count) * 3],
                    );
                    vb.set_texcoords(
                        &self.gl,
                        &self.texcoords[start * 2..(start + count) * 2],
                    );
                    vb.set_colors(
                        &self.gl,
                        &self.colors[start * 4..(start + count) * 4],
                    );
                    vb.set_has_tex(
                        &self.gl,
                        &self.has_tex[start * 1..(start + count) * 1],
                    );
                }

                if let Some(tex) = batch_info.tex {
                    if curr_tex.is_none()
                        || curr_tex.is_some_and(|t| t != tex)
                    {
                        curr_tex = Some(tex);
                        tex.bind(&self.gl);
                        self.program
                            .set_uniform_1_i32(&self.gl, "u_tex", 0);
                    }
                }

                let vb = &mut self.vertex_buffers[vb_idx];
                if let Some(indices_vbo) = vb.indices_vbo {
                    self.gl.bind_buffer(
                        glow::ELEMENT_ARRAY_BUFFER,
                        Some(indices_vbo),
                    );
                    self.gl.draw_elements(
                        glow::TRIANGLES,
                        vb.n_indices as i32,
                        glow::UNSIGNED_SHORT,
                        0,
                    );
                } else {
                    self.gl.draw_arrays(
                        glow::TRIANGLES,
                        0,
                        batch_info.count as i32,
                    );
                }
            }

            // -----------------------------------------------------------
            // Render the final image

            // Blit ms to postfx
            if self.ms_fbo.is_some() {
                self.gl
                    .bind_framebuffer(glow::READ_FRAMEBUFFER, self.ms_fbo);
                self.gl.bind_framebuffer(
                    glow::DRAW_FRAMEBUFFER,
                    Some(self.postfx_fbo),
                );
                self.gl.blit_framebuffer(
                    0,
                    0,
                    self.window_size.0 as i32,
                    self.window_size.1 as i32,
                    0,
                    0,
                    self.window_size.0 as i32,
                    self.window_size.1 as i32,
                    glow::COLOR_BUFFER_BIT,
                    glow::NEAREST,
                );
            }

            // Render postfx program
            if let Some(program) = postfx_program {
                program.bind(&self.gl);
                program.set_arg_uniforms(&self.gl);
                program.set_uniform_1_i32(&self.gl, "u_tex", 0);

                self.gl.active_texture(glow::TEXTURE0 + 0);
                self.postfx_tex.bind(&self.gl);

                self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                self.gl.viewport(
                    0,
                    0,
                    self.window_size.0 as i32,
                    self.window_size.1 as i32,
                );

                self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            // Or just blit the postfx to the screen
            } else {
                self.gl.bind_framebuffer(
                    glow::READ_FRAMEBUFFER,
                    Some(self.postfx_fbo),
                );
                self.gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
                self.gl.blit_framebuffer(
                    0,
                    0,
                    self.window_size.0 as i32,
                    self.window_size.1 as i32,
                    0,
                    0,
                    self.window_size.0 as i32,
                    self.window_size.1 as i32,
                    glow::COLOR_BUFFER_BIT,
                    glow::NEAREST,
                );
            }
        }

        self.reset_batches();
    }

    pub fn swap_window(&self) {
        self.window.gl_swap_window();
    }
}

fn buffer_sub_data<T>(
    gl: &glow::Context,
    target: u32,
    vbo: glow::NativeBuffer,
    data: &[T],
) {
    unsafe {
        gl.bind_buffer(target, Some(vbo));
        gl.buffer_sub_data_u8_slice(target, 0, cast_slice_to_u8(data));
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

fn get_msaa_max_n_samples(
    gl: &glow::Context,
    desired_n_samples: i32,
) -> i32 {
    let mut n_samples = unsafe { gl.get_parameter_i32(glow::MAX_SAMPLES) };

    n_samples = if desired_n_samples == 0 {
        0
    } else if desired_n_samples > 0 {
        n_samples.min(desired_n_samples)
    } else {
        n_samples
    };

    n_samples
}

fn get_transform_from_proj(
    proj: Projection,
    window_size: (u32, u32),
) -> Matrix4<f32> {
    use Projection::*;

    let transform = match proj {
        ProjScreen => Matrix4::new_orthographic(
            0.0,
            window_size.0 as f32,
            0.0,
            window_size.1 as f32,
            0.0,
            1.0,
        ),
        Proj2D { eye, zoom, rotation } => {
            let mut scale = Matrix4::identity();
            scale[(0, 0)] = zoom;
            scale[(1, 1)] = zoom;

            let mut translation = Matrix4::identity();
            translation[(0, 3)] = -eye.x;
            translation[(1, 3)] = -eye.y;

            let rotation =
                Matrix4::new_rotation(Vector3::new(0.0, 0.0, -rotation));

            let view = rotation * scale * translation;

            let projection = Matrix4::new_orthographic(
                window_size.0 as f32 / -2.0,
                window_size.0 as f32 / 2.0,
                window_size.1 as f32 / -2.0,
                window_size.1 as f32 / 2.0,
                0.0,
                1.0,
            );

            projection * view
        }
        Proj3D { eye, target, fovy } => {
            let fovy = fovy.to_radians();
            let up = Vector3::new(0.0, 1.0, 0.0);
            let view = Matrix4::look_at_rh(&eye, &target, &up);
            let aspect = window_size.0 as f32 / window_size.1 as f32;
            let projection =
                Matrix4::new_perspective(aspect, fovy, 0.1, 1000.0);

            projection * view
        }
    };

    transform
}
