use obj::{load_obj, raw::parse_obj, Obj, Vertex};

pub struct Mesh {
    pub n_vertices: usize,
    pub positions: Vec<f32>,
    pub texcoords: Option<Vec<f32>>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn from_obj_bytes(bytes: &[u8]) -> Self {
        let obj: Obj = load_obj(bytes).unwrap();
        let positions: Vec<f32> =
            obj.vertices.iter().flat_map(|v| v.position).collect();
        let n_vertices = positions.len() / 3;
        let texcoords = None;
        let indices = obj.indices;

        Self {
            n_vertices,
            positions,
            texcoords,
            indices,
        }
        // let obj = parse_obj(bytes).unwrap();

        // let positions: Vec<f32> = obj.positions.iter().flat_map(|&(x, y, z, _)| [x, y, z]).collect();
        // let n_vertices = positions.len() / 3;

        // let texcoords = if obj.tex_coords.len() != 0 {
        //     Some(obj.tex_coords.iter().flat_map(|&(x, y, _)| [x, y]).collect())
        // } else {
        //     None
        // };

        // let indices = obj.points.iter().map(|p| *p as u16).collect();
        // println!("{:?}", obj.polygons);

        // Self { n_vertices, positions, texcoords, indices }
    }
}
