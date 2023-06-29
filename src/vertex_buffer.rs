use crate::color::*;
use nalgebra::{Point2, Point3, Vector3};
use obj::raw::object::Polygon;
use obj::raw::parse_obj;

const INIT_VERT_CAP: usize = 1 << 15;

pub struct VertexBufferCPU {
    positions: Vec<f32>,
    normals: Vec<f32>,
    colors: Vec<f32>,
    texcoords: Vec<f32>,
    has_tex: Vec<u8>,
    indices: Option<Vec<u32>>,
}

impl VertexBufferCPU {
    pub fn new(
        positions: Vec<f32>,
        normals: Vec<f32>,
        colors: Vec<f32>,
        texcoords: Vec<f32>,
        has_tex: Vec<u8>,
        indices: Option<Vec<u32>>,
    ) -> Self {
        Self {
            positions,
            normals,
            colors,
            texcoords,
            has_tex,
            indices,
        }
    }

    pub fn new_empty() -> Self {
        Self::new(
            Vec::with_capacity(INIT_VERT_CAP * 3),
            Vec::with_capacity(INIT_VERT_CAP * 3),
            Vec::with_capacity(INIT_VERT_CAP * 4),
            Vec::with_capacity(INIT_VERT_CAP * 2),
            Vec::with_capacity(INIT_VERT_CAP * 1),
            None,
        )
    }

    pub fn from_obj_bytes(bytes: &[u8]) -> Self {
        let obj = parse_obj(bytes).unwrap();

        let mut positions = vec![];
        let mut normals = vec![];
        let mut texcoords = vec![];

        use Polygon::*;
        for polygon in obj.polygons {
            match polygon {
                P(p) => {
                    println!("a")
                }
                PT(pt) => {
                    println!("b")
                }
                PN(pn) => {
                    println!("c")
                }
                PTN(ptn) => {
                    for v in ptn.iter() {
                        positions.push(obj.positions[v.0].0);
                        positions.push(obj.positions[v.0].1);
                        positions.push(obj.positions[v.0].2);

                        normals.push(obj.normals[v.2].0);
                        normals.push(obj.normals[v.2].1);
                        normals.push(obj.normals[v.2].2);

                        texcoords.push(obj.tex_coords[v.1].0);
                        texcoords.push(obj.tex_coords[v.1].1);
                    }
                }
            }
        }

        let n_vertices = positions.len() / 3;
        let colors = vec![1.0; n_vertices * 4];
        let has_tex = vec![1; n_vertices];

        Self::new(positions, normals, colors, texcoords, has_tex, None)
    }

    pub fn push_vertex(
        &mut self,
        position: Point3<f32>,
        normal: Option<Vector3<f32>>,
        color: Option<Color>,
        texcoord: Option<Point2<f32>>,
    ) {
        if self.indices.is_some() {
            panic!("Can't push vertex to the indexed vertex buffer");
        }

        self.positions.extend_from_slice(position.coords.as_ref());
        if let Some(normal) = normal {
            self.normals.extend_from_slice(normal.as_ref());
        } else {
            self.normals.extend_from_slice(&[0.0; 3]);
        }
        self.colors
            .extend_from_slice(&color.unwrap_or(ZEROS).as_arr());

        if let Some(texcoord) = texcoord {
            self.texcoords.extend_from_slice(texcoord.coords.as_ref());
            self.has_tex.push(1);
        } else {
            self.texcoords.extend_from_slice(&[0.0, 0.0]);
            self.has_tex.push(0);
        }
    }

    pub fn get_positions(&self) -> &[f32] {
        &self.positions
    }

    pub fn get_normals(&self) -> &[f32] {
        &self.normals
    }

    pub fn get_colors(&self) -> &[f32] {
        &self.colors
    }

    pub fn get_texcoords(&self) -> &[f32] {
        &self.texcoords
    }

    pub fn get_has_tex(&self) -> &[u8] {
        &self.has_tex
    }

    pub fn get_indices(&self) -> Option<&[u32]> {
        self.indices.as_deref()
    }

    pub fn get_positions_slice(
        &self,
        from_vertex: usize,
        n_vertices: usize,
    ) -> &[f32] {
        &self.positions[from_vertex * 3..(from_vertex + n_vertices) * 3]
    }

    pub fn get_normals_slice(
        &self,
        from_vertex: usize,
        n_vertices: usize,
    ) -> &[f32] {
        &self.normals[from_vertex * 3..(from_vertex + n_vertices) * 3]
    }

    pub fn get_colors_slice(
        &self,
        from_vertex: usize,
        n_vertices: usize,
    ) -> &[f32] {
        &self.colors[from_vertex * 4..(from_vertex + n_vertices) * 4]
    }

    pub fn get_texcoords_slice(
        &self,
        from_vertex: usize,
        n_vertices: usize,
    ) -> &[f32] {
        &self.texcoords[from_vertex * 2..(from_vertex + n_vertices) * 2]
    }

    pub fn get_has_tex_slice(
        &self,
        from_vertex: usize,
        n_vertices: usize,
    ) -> &[u8] {
        &self.has_tex[from_vertex..(from_vertex + n_vertices)]
    }

    pub fn get_n_vertcies(&self) -> usize {
        self.positions.len() / 3
    }

    pub fn get_n_indices(&self) -> usize {
        self.indices.as_ref().map_or(0, |data| data.len())
    }

    pub fn set_has_tex(&mut self, has_tex: bool) {
        self.has_tex.fill(has_tex as u8);
    }

    pub fn set_color(&mut self, color: Color) {
        let color = color.as_arr();
        for i in 0..self.colors.len() {
            self.colors[i] = color[i % 4];
        }
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.colors.clear();
        self.texcoords.clear();
        self.has_tex.clear();
        self.indices.as_mut().map(|data| data.clear());
    }
}
