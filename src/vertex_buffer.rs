use obj::raw::object::Polygon;
use obj::{load_obj, raw::parse_obj, Obj, Vertex};

pub struct VertexBufferCPU {
    pub positions: Vec<f32>,
    pub colors: Vec<f32>,
    pub texcoords: Vec<f32>,
    pub has_tex: Vec<u8>,
    pub indices: Vec<u32>,
}

impl VertexBufferCPU {
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

        Self {
            positions,
            colors,
            texcoords,
            has_tex,
            indices,
        }
    }

    pub fn get_n_vertcies(&self) -> usize {
        self.positions.len() / 3
    }

    pub fn get_n_indices(&self) -> usize {
        self.indices.len()
    }
}
