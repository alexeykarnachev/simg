use crate::color::*;
use enum_iterator::Sequence;
use nalgebra::{Point2, Point3, Vector3};
use obj::raw::object::Polygon;
use obj::raw::parse_obj;

const INIT_VERT_CAP: usize = 1 << 15;

#[repr(u8)]
#[derive(Sequence, Copy, Clone, Debug)]
pub enum VertexFlag {
    HasTexture = 1 << 0,
    HasNormal = 1 << 1,
}
impl From<VertexFlag> for u32 {
    fn from(e: VertexFlag) -> u32 {
        e as u32
    }
}

pub struct VertexBufferCPU {
    positions: Vec<f32>,
    normals: Vec<f32>,
    colors: Vec<f32>,
    texcoords: Vec<f32>,
    flags: Vec<u8>,
    indices: Option<Vec<u32>>,
}

impl VertexBufferCPU {
    pub fn new(
        positions: Vec<f32>,
        normals: Vec<f32>,
        colors: Vec<f32>,
        texcoords: Vec<f32>,
        flags: Vec<u8>,
        indices: Option<Vec<u32>>,
    ) -> Self {
        Self {
            positions,
            normals,
            colors,
            texcoords,
            flags,
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
        use VertexFlag::*;

        let obj = parse_obj(bytes).unwrap();

        let mut positions = vec![];
        let mut normals = vec![];
        let mut texcoords = vec![];
        let mut flags = vec![];

        use Polygon::*;
        for polygon in obj.polygons {
            match polygon {
                P(p) => {
                    for v in p.iter() {
                        positions.push(obj.positions[*v].0);
                        positions.push(obj.positions[*v].1);
                        positions.push(obj.positions[*v].2);

                        normals.push(0.0);
                        normals.push(0.0);
                        normals.push(0.0);

                        texcoords.push(0.0);
                        texcoords.push(0.0);
                    }
                }
                PT(pt) => {
                    for v in pt.iter() {
                        positions.push(obj.positions[v.0].0);
                        positions.push(obj.positions[v.0].1);
                        positions.push(obj.positions[v.0].2);

                        normals.push(0.0);
                        normals.push(0.0);
                        normals.push(0.0);

                        texcoords.push(obj.tex_coords[v.1].0);
                        texcoords.push(obj.tex_coords[v.1].1);

                        flags.push(HasTexture as u8);
                    }
                }
                PN(pn) => {
                    for v in pn.iter() {
                        positions.push(obj.positions[v.0].0);
                        positions.push(obj.positions[v.0].1);
                        positions.push(obj.positions[v.0].2);

                        normals.push(obj.normals[v.1].0);
                        normals.push(obj.normals[v.1].1);
                        normals.push(obj.normals[v.1].2);

                        texcoords.push(0.0);
                        texcoords.push(0.0);

                        flags.push(HasNormal as u8);
                    }
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

                        flags.push(HasNormal as u8 | HasTexture as u8);
                    }
                }
            }
        }

        let n_vertices = positions.len() / 3;
        let colors = vec![1.0; n_vertices * 4];

        Self::new(positions, normals, colors, texcoords, flags, None)
    }

    pub fn push_vertex(
        &mut self,
        position: Point3<f32>,
        normal: Option<Vector3<f32>>,
        color: Option<Color>,
        texcoord: Option<Point2<f32>>,
    ) {
        use VertexFlag::*;

        if self.indices.is_some() {
            panic!("Can't push vertex to the indexed vertex buffer");
        }

        let mut flags = 0;
        self.positions.extend_from_slice(position.coords.as_ref());
        if let Some(normal) = normal {
            self.normals.extend_from_slice(normal.as_ref());
            flags |= HasNormal as u8;
        } else {
            self.normals.extend_from_slice(&[0.0; 3]);
        }
        self.colors
            .extend_from_slice(&color.unwrap_or(ZEROS).as_arr());

        if let Some(texcoord) = texcoord {
            self.texcoords.extend_from_slice(texcoord.coords.as_ref());
            flags |= HasTexture as u8;
        } else {
            self.texcoords.extend_from_slice(&[0.0, 0.0]);
        }

        self.flags.push(flags);
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

    pub fn get_flags(&self) -> &[u8] {
        &self.flags
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

    pub fn get_flags_slice(
        &self,
        from_vertex: usize,
        n_vertices: usize,
    ) -> &[u8] {
        &self.flags[from_vertex..(from_vertex + n_vertices)]
    }

    pub fn get_n_vertcies(&self) -> usize {
        self.positions.len() / 3
    }

    pub fn get_n_indices(&self) -> usize {
        self.indices.as_ref().map_or(0, |data| data.len())
    }

    pub fn set_flags(&mut self, flags: u8) {
        self.set_flags_slice(flags, 0, self.get_n_vertcies());
    }

    pub fn unset_flags(&mut self, flags: u8) {
        self.unset_flags_slice(flags, 0, self.get_n_vertcies());
    }

    pub fn set_colors(&mut self, color: Color) {
        self.set_colors_slice(color, 0, self.get_n_vertcies());
    }

    pub fn set_flags_slice(
        &mut self,
        flags: u8,
        from_vertex: usize,
        n_vertices: usize,
    ) {
        self.flags[from_vertex..from_vertex + n_vertices].fill(flags);
    }

    pub fn unset_flags_slice(
        &mut self,
        flags: u8,
        from_vertex: usize,
        n_vertices: usize,
    ) {
        for i in from_vertex..from_vertex + n_vertices {
            self.flags[i] &= !flags;
        }
    }

    pub fn set_colors_slice(
        &mut self,
        color: Color,
        from_vertex: usize,
        n_vertices: usize,
    ) {
        let color = color.as_arr();
        let start = from_vertex * 4;
        let end = (from_vertex + n_vertices) * 4;
        for i in start..end {
            self.colors[i] = color[i % 4];
        }
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.colors.clear();
        self.texcoords.clear();
        self.flags.clear();
        self.indices.as_mut().map(|data| data.clear());
    }
}
