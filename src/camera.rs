use nalgebra::{Matrix4, Vector2, Vector3};

#[derive(Clone, Copy)]
pub struct Camera2D {
    pub position: Vector2<f32>,
    pub rotation: f32,
    pub zoom: f32,
}

impl Camera2D {
    pub fn new(position: Vector2<f32>) -> Self {
        Self { position, rotation: 0.0, zoom: 1.0 }
    }

    pub fn get_view(&self) -> Matrix4<f32> {
        let mut scale = Matrix4::identity();
        scale[(0, 0)] = self.zoom;
        scale[(1, 1)] = self.zoom;

        let mut translation = Matrix4::identity();
        translation[(0, 3)] = -self.position.x;
        translation[(1, 3)] = -self.position.y;

        let rotation =
            Matrix4::new_rotation(Vector3::new(0.0, 0.0, -self.rotation));

        rotation * scale * translation
    }
}
