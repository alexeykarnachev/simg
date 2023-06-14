use nalgebra::Vector2;

type Vertex = Vector2<f32>;

#[derive(Clone, Copy)]
pub struct Triangle {
    pub a: Vertex,
    pub b: Vertex,
    pub c: Vertex,
}

impl Triangle {
    pub fn new(a: Vertex, b: Vertex, c: Vertex) -> Self {
        Self { a, b, c }
    }
}

#[derive(Clone, Copy)]
pub struct Rect {
    bot_left: Vertex,
    top_right: Vertex,
}

impl Rect {
    pub fn new(bot_left: Vertex, top_right: Vertex) -> Self {
        Self {
            bot_left,
            top_right,
        }
    }

    pub fn get_bot_left(&self) -> Vertex {
        self.bot_left
    }

    pub fn get_top_right(&self) -> Vertex {
        self.top_right
    }

    pub fn get_bot_right(&self) -> Vertex {
        let mut bot_right = self.bot_left;
        bot_right.x = self.top_right.x;

        bot_right
    }

    pub fn get_top_left(&self) -> Vertex {
        let mut top_left = self.bot_left;
        top_left.y = self.top_right.y;

        top_left
    }

    pub fn from_top_left(top_left: Vertex, size: Vertex) -> Self {
        let mut bot_left = top_left;
        bot_left.y -= size.y;

        let mut top_right = top_left;
        top_right.x += size.x;

        Self {
            bot_left,
            top_right,
        }
    }

    pub fn to_triangles(&self) -> [Triangle; 2] {
        [
            Triangle::new(
                self.get_top_left(),
                self.get_bot_left(),
                self.get_top_right(),
            ),
            Triangle::new(
                self.get_top_right(),
                self.get_bot_left(),
                self.get_bot_right(),
            ),
        ]
    }

    pub fn to_some_triangles(&self) -> [Option<Triangle>; 2] {
        let triangles = self.to_triangles();

        [Some(triangles[0]), Some(triangles[1])]
    }
}
