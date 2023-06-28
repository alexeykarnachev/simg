use crate::color::*;
use nalgebra::{Point2, Point3};
use obj::raw::object::Polygon;
use obj::{load_obj, raw::parse_obj, Obj, Vertex};

const INIT_VERT_CAP: usize = 1 << 15;

pub struct VertexBufferCPU {
    positions: Vec<f32>,
    colors: Vec<f32>,
    texcoords: Vec<f32>,
    has_tex: Vec<u8>,
    indices: Option<Vec<u32>>,
}

impl VertexBufferCPU {
    pub fn new(
        positions: Vec<f32>,
        colors: Vec<f32>,
        texcoords: Vec<f32>,
        has_tex: Vec<u8>,
        indices: Option<Vec<u32>>,
    ) -> Self {
        Self {
            positions,
            colors,
            texcoords,
            has_tex,
            indices,
        }
    }

    pub fn new_empty() -> Self {
        Self::new(
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
        let mut indices = vec![];
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
                    let v0 = ptn[0];
                    let v1 = ptn[1];
                    let v2 = ptn[2];

                    indices.push(indices.len() as u32);
                    indices.push(indices.len() as u32);
                    indices.push(indices.len() as u32);

                    positions.push(obj.positions[v0.0].0);
                    positions.push(obj.positions[v0.0].1);
                    positions.push(obj.positions[v0.0].2);
                    positions.push(obj.positions[v1.0].0);
                    positions.push(obj.positions[v1.0].1);
                    positions.push(obj.positions[v1.0].2);
                    positions.push(obj.positions[v2.0].0);
                    positions.push(obj.positions[v2.0].1);
                    positions.push(obj.positions[v2.0].2);

                    texcoords.push(obj.tex_coords[v0.1].0);
                    texcoords.push(obj.tex_coords[v0.1].1);
                    texcoords.push(obj.tex_coords[v1.1].0);
                    texcoords.push(obj.tex_coords[v1.1].1);
                    texcoords.push(obj.tex_coords[v2.1].0);
                    texcoords.push(obj.tex_coords[v2.1].1);
                }
            }
        }

        let n_vertices = positions.len() / 3;
        let colors = vec![1.0; n_vertices * 4];
        let has_tex = vec![1; n_vertices];

        Self::new(positions, colors, texcoords, has_tex, Some(indices))
    }

    pub fn push_vertex(
        &mut self,
        position: Point3<f32>,
        color: Option<Color>,
        texcoord: Option<Point2<f32>>,
    ) {
        if self.indices.is_some() {
            panic!("Can't push vertex to the indexed vertex buffer");
        }

        self.positions.extend_from_slice(position.coords.as_ref());
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

    pub fn clear(&mut self) {
        self.positions.clear();
        self.colors.clear();
        self.texcoords.clear();
        self.has_tex.clear();
        self.indices.as_mut().map(|data| data.clear());
    }
}
