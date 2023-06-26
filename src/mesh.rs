use obj::{load_obj, raw::parse_obj, Obj, Vertex};

pub struct Mesh {
    pub positions: Vec<f32>,
    pub colors: Vec<f32>,
    pub texcoords: Vec<f32>,
    pub has_tex: Vec<u8>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn from_obj_bytes(bytes: &[u8]) -> Self {
        let obj: Obj = load_obj(bytes).unwrap();
        let positions: Vec<f32> =
            obj.vertices.iter().flat_map(|v| v.position).collect();
        let indices = obj.indices;

        let n_vertices = positions.len() / 3;
        let colors = vec![1.0; n_vertices * 4];
        let texcoords = vec![0.0; n_vertices * 2];
        let has_tex = vec![0; n_vertices];

        Self {
            positions,
            colors,
            texcoords,
            has_tex,
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

    pub fn get_n_vertcies(&self) -> usize {
        self.positions.len() / 3
    }

    pub fn get_n_indices(&self) -> usize {
        self.indices.len()
    }
}
