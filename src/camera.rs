use nalgebra::Point2;

#[derive(Clone, Copy)]
pub struct Camera2D {
    pub position: Point2<f32>,
    pub rotation: f32,
    pub zoom: f32,
}

impl Default for Camera2D {
    fn default() -> Self {
        Self::new(Point2::origin())
    }
}

impl Camera2D {
    pub fn new(position: Point2<f32>) -> Self {
        Self { position, rotation: 0.0, zoom: 1.0 }
    }
}
