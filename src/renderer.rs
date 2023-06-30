use crate::color::*;
use crate::common::*;
use crate::glyph_atlas::*;
use crate::shapes::*;
use crate::vertex_buffer::*;
use core::fmt::Debug;
use enum_iterator::{all, Sequence};
use image::{load_from_memory_with_format, EncodableLayout, ImageFormat};
use nalgebra::{point, Matrix3, Matrix4, Point2, Point3, Vector3};
use std::num::NonZeroU32;

use glow::HasContext;

const PRIMITIVE_VERT_SRC: &str = include_str!("../shaders/primitive.vert");
const PRIMITIVE_FRAG_SRC: &str = include_str!("../shaders/primitive.frag");
const SCREEN_RECT_VERT_SRC: &str =
    include_str!("../shaders/screen_rect.vert");
const MAX_N_VERTICES: usize = 1 << 15;

#[derive(Copy, Clone, PartialEq)]
struct VertexBufferGL {
    vao: glow::NativeVertexArray,
    positions_vbo: glow::NativeBuffer,
    normals_vbo: glow::NativeBuffer,
    colors_vbo: glow::NativeBuffer,
    texcoords_vbo: glow::NativeBuffer,
    flags_vbo: glow::NativeBuffer,
    indices_vbo: Option<glow::NativeBuffer>,

    n_vertices: usize,
    n_indices: usize,
}

impl VertexBufferGL {
    pub fn new(
        gl: &glow::Context,
        positions: &[f32],
        normals: &[f32],
        colors: &[f32],
        texcoords: &[f32],
        flags: &[u8],
        indices: Option<&[u32]>,
    ) -> Self {
        let n_vertices = positions.len() / 3;
        let mut n_indices = 0;

        let vao;
        let positions_vbo;
        let normals_vbo;
        let texcoords_vbo;
        let colors_vbo;
        let flags_vbo;
        let mut indices_vbo = None;
        unsafe {
            vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
        }

        positions_vbo = create_attrib_vbo(gl, 0, 3, positions);
        normals_vbo = create_attrib_vbo(gl, 1, 3, normals);
        texcoords_vbo = create_attrib_vbo(gl, 2, 2, texcoords);
        colors_vbo = create_attrib_vbo(gl, 3, 4, colors);
        flags_vbo = create_attrib_vbo(gl, 4, 1, flags);

        if let Some(indices) = indices {
            indices_vbo = Some(create_indices_vbo(gl, indices));
            n_indices = indices.len();
        }

        Self {
            vao,
            positions_vbo,
            normals_vbo,
            colors_vbo,
            texcoords_vbo,
            flags_vbo,
            indices_vbo,

            n_vertices,
            n_indices,
        }
    }

    pub fn new_empty(gl: &glow::Context, n_vertices: usize) -> Self {
        Self::new(
            gl,
            &vec![0.0; n_vertices * 3],
            &vec![0.0; n_vertices * 3],
            &vec![0.0; n_vertices * 4],
            &vec![0.0; n_vertices * 2],
            &vec![0; n_vertices],
            None,
        )
    }

    pub fn new_from_cpu(gl: &glow::Context, vb: &VertexBufferCPU) -> Self {
        Self::new(
            gl,
            vb.get_positions(),
            vb.get_normals(),
            vb.get_colors(),
            vb.get_texcoords(),
            vb.get_flags(),
            vb.get_indices(),
        )
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, self.indices_vbo);
            gl.bind_vertex_array(Some(self.vao));
        }
    }

    fn set_from_cpu(&mut self, gl: &glow::Context, vb: &VertexBufferCPU) {
        self.set_data(
            gl,
            vb.get_positions(),
            vb.get_normals(),
            vb.get_texcoords(),
            vb.get_colors(),
            vb.get_flags(),
            vb.get_indices(),
        );
    }

    fn set_from_cpu_slice(
        &mut self,
        gl: &glow::Context,
        vb: &VertexBufferCPU,
        from_vertex: usize,
        n_vertices: usize,
    ) {
        self.set_data(
            gl,
            vb.get_positions_slice(from_vertex, n_vertices),
            vb.get_normals_slice(from_vertex, n_vertices),
            vb.get_texcoords_slice(from_vertex, n_vertices),
            vb.get_colors_slice(from_vertex, n_vertices),
            vb.get_flags_slice(from_vertex, n_vertices),
            None,
        );
    }

    fn set_data(
        &mut self,
        gl: &glow::Context,
        positions: &[f32],
        normals: &[f32],
        texcoords: &[f32],
        colors: &[f32],
        flags: &[u8],
        indices: Option<&[u32]>,
    ) {
        self.n_vertices = positions.len() / 3;
        if normals.len() / 3 != self.n_vertices
            || texcoords.len() / 2 != self.n_vertices
            || colors.len() / 4 != self.n_vertices
            || flags.len() != self.n_vertices
        {
            panic!("Can't set vertex buffer data with inconsistent number of components in the arrays");
        }

        self.n_indices = indices.map_or(0, |data| data.len());

        unsafe {
            if let (Some(vbo), Some(data)) = (self.indices_vbo, indices) {
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(vbo));
                gl.buffer_sub_data_u8_slice(
                    glow::ELEMENT_ARRAY_BUFFER,
                    0,
                    cast_slice_to_u8(data),
                );
            } else if self.indices_vbo.is_none() && indices.is_some() {
                panic!(
                    "Can't set indices for the unindexed vertex buffer"
                );
            } else if self.indices_vbo.is_some() && indices.is_none() {
                panic!("Expecting indexes for the indexed vertex buffer");
            }

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.positions_vbo));
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                cast_slice_to_u8(positions),
            );

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.normals_vbo));
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                cast_slice_to_u8(normals),
            );

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.texcoords_vbo));
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                cast_slice_to_u8(texcoords),
            );

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.colors_vbo));
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                cast_slice_to_u8(colors),
            );

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.flags_vbo));
            gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                0,
                cast_slice_to_u8(flags),
            );
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

        let header =
            header.to_owned() + &enum_to_shader_source::<VertexFlag>();

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

    fn set_uniform_matrix_3_f32(
        &self,
        gl: &glow::Context,
        name: &str,
        val: &[f32],
    ) {
        unsafe {
            gl.uniform_matrix_3_f32_slice(
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
                glow::REPEAT as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::REPEAT as i32,
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

#[derive(Debug, Clone, Default)]
struct DrawCall {
    vb_idx: usize,
    start: usize,
    count: usize,
    tex: Option<Texture>,
    transform: Option<Transformation>,
    camera: Option<Camera>,
    proj: Option<Projection>,
    is_font: bool,
    depth_test: bool,
}

impl DrawCall {
    pub fn new(
        vb_idx: usize,
        start: usize,
        tex: Option<Texture>,
        transform: Option<Transformation>,
        camera: Option<Camera>,
        proj: Option<Projection>,
        is_font: bool,
        depth_test: bool,
    ) -> Self {
        Self {
            vb_idx,
            start,
            count: 0,
            tex,
            transform,
            camera,
            proj,
            is_font,
            depth_test,
        }
    }
}

pub struct Renderer {
    window: sdl2::video::Window,
    gl: glow::Context,
    program: Program,

    ms_fbo: Option<glow::NativeFramebuffer>,

    postfx_fbo: glow::NativeFramebuffer,
    postfx_tex: Texture,

    vb_cpu: VertexBufferCPU,
    vertex_buffers: Vec<VertexBufferGL>,
    draw_calls: Vec<DrawCall>,
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

        let mut ms_fbo = None;
        let postfx_tex;
        let postfx_fbo;
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.enable(glow::CULL_FACE);
            gl.cull_face(glow::BACK);
            gl.front_face(glow::CCW);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            // -----------------------------------------------------------
            // Multisample buffer
            let n_samples = get_msaa_max_n_samples(&gl, msaa);
            if n_samples > 0 {
                ms_fbo = Some(gl.create_framebuffer().unwrap());
                gl.bind_framebuffer(glow::FRAMEBUFFER, ms_fbo);

                let ms_color_rbo = Some(gl.create_renderbuffer().unwrap());
                gl.bind_renderbuffer(glow::RENDERBUFFER, ms_color_rbo);
                gl.renderbuffer_storage_multisample(
                    glow::RENDERBUFFER,
                    n_samples as i32,
                    glow::RGBA32F,
                    window_size.0 as i32,
                    window_size.1 as i32,
                );
                gl.framebuffer_renderbuffer(
                    glow::FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0,
                    glow::RENDERBUFFER,
                    ms_color_rbo,
                );

                let ms_depth_rbo = Some(gl.create_renderbuffer().unwrap());
                gl.bind_renderbuffer(glow::RENDERBUFFER, ms_depth_rbo);
                gl.renderbuffer_storage_multisample(
                    glow::RENDERBUFFER,
                    n_samples as i32,
                    glow::DEPTH_COMPONENT16,
                    window_size.0 as i32,
                    window_size.1 as i32,
                );
                gl.framebuffer_renderbuffer(
                    glow::FRAMEBUFFER,
                    glow::DEPTH_ATTACHMENT,
                    glow::RENDERBUFFER,
                    ms_depth_rbo,
                );
            }

            // -----------------------------------------------------------
            // Postfx buffer
            postfx_fbo = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(postfx_fbo));

            let rbo = Some(gl.create_renderbuffer().unwrap());
            gl.bind_renderbuffer(glow::RENDERBUFFER, rbo);
            gl.renderbuffer_storage(
                glow::RENDERBUFFER,
                glow::DEPTH_COMPONENT16,
                window_size.0 as i32,
                window_size.1 as i32,
            );
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                glow::RENDERBUFFER,
                rbo,
            );

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

        let vertex_buffers =
            vec![VertexBufferGL::new_empty(&gl, MAX_N_VERTICES)];

        Self {
            window,
            gl,
            program,

            ms_fbo,

            postfx_fbo,
            postfx_tex,

            vb_cpu: VertexBufferCPU::new_empty(),
            vertex_buffers,
            draw_calls: Vec::with_capacity(128),
        }
    }

    pub fn get_window_size(&self) -> (u32, u32) {
        self.window.size()
    }

    pub fn get_window_aspect(&self) -> f32 {
        let (w, h) = self.get_window_size();

        w as f32 / h as f32
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
            .flipv()
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

    pub fn load_vertex_buffer_from_cpu(
        &mut self,
        vb: &VertexBufferCPU,
    ) -> usize {
        let vb = VertexBufferGL::new_from_cpu(&self.gl, vb);
        self.vertex_buffers.push(vb);

        self.vertex_buffers.len() - 1
    }

    fn draw_vertex(
        &mut self,
        position: Point3<f32>,
        normal: Option<Vector3<f32>>,
        color: Option<Color>,
        texcoord: Option<Point2<f32>>,
    ) {
        let draw_call = self.get_default_draw_call();
        draw_call.count += 1;

        self.vb_cpu.push_vertex(position, normal, color, texcoord);
    }

    pub fn draw_triangle(
        &mut self,
        triangle: Triangle,
        normals: Option<Triangle>,
        texcoords: Option<Triangle>,
        color: Option<Color>,
    ) {
        let normals = if let Some(normals) = normals {
            [
                Some(normals.a.coords),
                Some(normals.b.coords),
                Some(normals.c.coords),
            ]
        } else {
            [None; 3]
        };

        let texcoords = if let Some(texcoords) = texcoords {
            [
                Some(texcoords.a.xy()),
                Some(texcoords.b.xy()),
                Some(texcoords.c.xy()),
            ]
        } else {
            [None; 3]
        };

        self.draw_vertex(triangle.a, normals[0], color, texcoords[0]);
        self.draw_vertex(triangle.b, normals[1], color, texcoords[1]);
        self.draw_vertex(triangle.c, normals[2], color, texcoords[2]);
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

        self.draw_triangle(positions[0], None, texcoords[0], color);
        self.draw_triangle(positions[1], None, texcoords[1], color);
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
            self.draw_triangle(positions[i], None, texcoords[i], color);
        }
    }

    pub fn draw_glyph(&mut self, glyph: Glyph, color: Option<Color>) {
        self.draw_rect(glyph.rect, Some(glyph.texcoords), color);
    }

    pub fn draw_vertex_buffer(
        &mut self,
        vb_idx: usize,
        transform: Option<Transformation>,
    ) {
        let vb = self.vertex_buffers[vb_idx];

        let mut draw_call = self.get_new_draw_call();
        draw_call.vb_idx = vb_idx;
        draw_call.count = vb.n_vertices;
        draw_call.transform = transform;
    }

    pub fn set_proj(&mut self, proj: Projection) {
        let curr_proj = self.get_curr_draw_call().proj;
        if curr_proj.is_none() || curr_proj.is_some_and(|p| p != proj) {
            let draw_call = self.get_new_draw_call();
            draw_call.proj = Some(proj);
        }
    }

    pub fn set_screen_proj(&mut self) {
        self.set_proj(Projection::new_screen(self.get_window_size()));
    }

    pub fn set_camera(&mut self, camera: Camera) {
        let curr_camera = self.get_curr_draw_call().camera;
        if curr_camera.is_none()
            || curr_camera.is_some_and(|p| p != camera)
        {
            let draw_call = self.get_new_draw_call();
            draw_call.camera = Some(camera);
        }
    }

    pub fn set_screen_camera(&mut self) {
        self.set_camera(Camera::new_screen(self.get_window_size()));
    }

    pub fn set_origin_2d_camera(&mut self) {
        self.set_camera(Camera::new_origin_2d());
    }

    pub fn set_depth_test(&mut self, is_set: bool) {
        if self.get_curr_draw_call().depth_test != is_set {
            self.get_new_draw_call().depth_test = is_set;
        }
    }

    pub fn set_tex(&mut self, tex: Texture, is_font: bool) {
        let curr_tex = self.get_curr_draw_call().tex;
        if curr_tex.is_none() || curr_tex.is_some_and(|t| t != tex) {
            let draw_call = self.get_new_draw_call();
            draw_call.tex = Some(tex);
            draw_call.is_font = is_font;
        }
    }

    fn get_curr_draw_call(&mut self) -> &mut DrawCall {
        if self.draw_calls.len() == 0 {
            return self.get_new_draw_call();
        }

        let idx = self.draw_calls.len() - 1;
        &mut self.draw_calls[idx]
    }

    fn get_new_draw_call(&mut self) -> &mut DrawCall {
        if self.draw_calls.len() == 0 {
            self.draw_calls.push(DrawCall::default());
        } else if self.get_curr_draw_call().count != 0 {
            let curr = self.get_curr_draw_call().clone();
            let new = DrawCall {
                vb_idx: 0,
                start: self.vb_cpu.get_n_vertcies(),
                count: 0,
                tex: curr.tex,
                transform: None,
                camera: curr.camera,
                proj: curr.proj,
                is_font: curr.is_font,
                depth_test: curr.depth_test,
            };
            self.draw_calls.push(new);
        }

        self.get_curr_draw_call()
    }

    fn get_default_draw_call(&mut self) -> &mut DrawCall {
        if self.get_curr_draw_call().vb_idx != 0 {
            return self.get_new_draw_call();
        }

        self.get_curr_draw_call()
    }

    pub fn end_drawing(
        &mut self,
        clear_color: Color,
        postfx_program: Option<&Program>,
    ) {
        let window_size = self.get_window_size();
        let screen_rect = Rectangle::new(
            point![0.0, 0.0],
            point![window_size.0 as f32, window_size.1 as f32],
        );
        unsafe {
            self.gl.bind_texture(glow::TEXTURE_2D, None);

            // -----------------------------------------------------------
            // Draw scene to the multisample buffer
            let out_fbo = if let Some(ms_fbo) = self.ms_fbo {
                Some(ms_fbo)
            } else {
                Some(self.postfx_fbo)
            };

            bind_framebuffer(
                &self.gl,
                out_fbo,
                &screen_rect,
                Some(clear_color),
                true,
            );

            self.program.bind(&self.gl);

            let mut curr_vb_idx = None;
            let mut curr_tex = None;

            for draw_call in self.draw_calls.iter() {
                if draw_call.depth_test {
                    self.gl.enable(glow::DEPTH_TEST);
                } else {
                    self.gl.disable(glow::DEPTH_TEST);
                }

                let vb_idx = draw_call.vb_idx;
                let model = draw_call
                    .transform
                    .as_ref()
                    .map_or(Matrix4::identity(), |t| t.get_mat());
                let view = if let Some(camera) = draw_call.camera.as_ref()
                {
                    camera.get_mat()
                } else {
                    panic!("The draw call doesn't have camera. Call `renderer.set_camera` before drawing");
                };
                let proj = if let Some(proj) = draw_call.proj.as_ref() {
                    proj.get_mat()
                } else {
                    panic!("The draw call doesn't have projection. Call `renderer.set_proj` before drawing");
                };
                let position_mat = proj * view * model;
                let normal_mat = Matrix3::from_fn(|i, j| model[(i, j)])
                    .try_inverse()
                    .unwrap()
                    .transpose();

                self.program.set_uniform_matrix_4_f32(
                    &self.gl,
                    "u_position_mat",
                    position_mat.as_slice(),
                );

                self.program.set_uniform_matrix_3_f32(
                    &self.gl,
                    "u_normal_mat",
                    normal_mat.as_slice(),
                );

                if curr_vb_idx.is_none()
                    || curr_vb_idx.is_some_and(|idx| idx != vb_idx)
                {
                    curr_vb_idx = Some(vb_idx);
                    self.vertex_buffers[vb_idx].bind(&self.gl);
                }

                // Update default vertex buffer
                if vb_idx == 0 {
                    self.vertex_buffers[0].set_from_cpu_slice(
                        &self.gl,
                        &self.vb_cpu,
                        draw_call.start,
                        draw_call.count,
                    );
                }

                if let Some(tex) = draw_call.tex {
                    if curr_tex.is_none()
                        || curr_tex.is_some_and(|t| t != tex)
                    {
                        curr_tex = Some(tex);
                        tex.bind(&self.gl);
                        self.program
                            .set_uniform_1_i32(&self.gl, "u_tex", 0);
                        self.program.set_uniform_1_u32(
                            &self.gl,
                            "u_is_font",
                            draw_call.is_font as u32,
                        );
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
                        glow::UNSIGNED_INT,
                        0,
                    );
                } else {
                    self.gl.draw_arrays(
                        glow::TRIANGLES,
                        0,
                        draw_call.count as i32,
                    );
                }
            }

            // -----------------------------------------------------------
            // Render the final image

            // Blit ms to postfx
            if let Some(ms_fbo) = self.ms_fbo {
                blit_framebuffer(
                    &self.gl,
                    ms_fbo,
                    Some(self.postfx_fbo),
                    &screen_rect,
                    &screen_rect,
                );
            }

            // Render postfx program
            if let Some(program) = postfx_program {
                program.bind(&self.gl);
                program.set_arg_uniforms(&self.gl);
                program.set_uniform_1_i32(&self.gl, "u_tex", 0);

                self.gl.active_texture(glow::TEXTURE0 + 0);
                self.postfx_tex.bind(&self.gl);

                bind_framebuffer(
                    &self.gl,
                    None,
                    &screen_rect,
                    Some(clear_color),
                    true,
                );
                self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            // Or just blit the postfx to the screen
            } else {
                blit_framebuffer(
                    &self.gl,
                    self.postfx_fbo,
                    None,
                    &screen_rect,
                    &screen_rect,
                );
            }
        }

        self.draw_calls.clear();
        self.vb_cpu.clear();
    }

    pub fn swap_window(&self) {
        self.window.gl_swap_window();
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

fn create_attrib_vbo<T>(
    gl: &glow::Context,
    index: u32,
    size: i32,
    data: &[T],
) -> glow::NativeBuffer {
    unsafe {
        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            cast_slice_to_u8(data),
            glow::DYNAMIC_DRAW,
        );
        gl.enable_vertex_attrib_array(index);

        let type_name = std::any::type_name::<T>();
        if type_name == std::any::type_name::<f32>() {
            gl.vertex_attrib_pointer_f32(
                index,
                size,
                glow::FLOAT,
                false,
                0,
                0,
            );
        } else if type_name == std::any::type_name::<u8>() {
            gl.vertex_attrib_pointer_i32(
                index,
                size,
                glow::UNSIGNED_BYTE,
                0,
                0,
            );
        } else {
            panic!(
                "Can't create attribute vbo with data of type: {}",
                type_name
            );
        }

        vbo
    }
}

fn create_indices_vbo(
    gl: &glow::Context,
    data: &[u32],
) -> glow::NativeBuffer {
    unsafe {
        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            cast_slice_to_u8(data),
            glow::STATIC_DRAW,
        );

        vbo
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

fn bind_framebuffer(
    gl: &glow::Context,
    fbo: Option<glow::NativeFramebuffer>,
    viewport: &Rectangle,
    color: Option<Color>,
    depth: bool,
) {
    unsafe {
        gl.bind_framebuffer(glow::FRAMEBUFFER, fbo);
        gl.viewport(
            viewport.get_min_x() as i32,
            viewport.get_min_y() as i32,
            viewport.get_max_x() as i32,
            viewport.get_max_y() as i32,
        );
        clear_framebuffer(gl, color, depth);
    }
}

fn clear_framebuffer(
    gl: &glow::Context,
    color: Option<Color>,
    depth: bool,
) {
    unsafe {
        let mut flags = 0;
        if let Some(color) = color {
            gl.clear_color(color.r, color.g, color.b, color.a);
            flags |= glow::COLOR_BUFFER_BIT;
        }
        if depth {
            flags |= glow::DEPTH_BUFFER_BIT;
        }
        if flags != 0 {
            gl.clear(flags);
        }
    }
}

fn blit_framebuffer(
    gl: &glow::Context,
    src: glow::NativeFramebuffer,
    dst: Option<glow::NativeFramebuffer>,
    src_rect: &Rectangle,
    dst_rect: &Rectangle,
) {
    unsafe {
        gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, dst);
        clear_framebuffer(gl, Some(BLACK), true);

        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(src));

        gl.blit_framebuffer(
            src_rect.get_min_x() as i32,
            src_rect.get_min_y() as i32,
            src_rect.get_max_x() as i32,
            src_rect.get_max_y() as i32,
            dst_rect.get_min_x() as i32,
            dst_rect.get_min_y() as i32,
            dst_rect.get_max_x() as i32,
            dst_rect.get_max_y() as i32,
            glow::COLOR_BUFFER_BIT,
            glow::NEAREST,
        );
    }
}

fn enum_to_shader_source<T: Sequence + Debug + Copy + Into<u32>>() -> String
{
    let mut source = String::new();

    for variant in all::<T>().collect::<Vec<_>>() {
        let definition =
            format!("const uint {:?} = uint({:?});\n", variant, variant.into());
        source.push_str(&definition);
    }

    source
}
