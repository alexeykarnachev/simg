use nalgebra::Vector2;

pub enum Pivot {
    BotLeft(Vector2<f32>),
    TopLeft(Vector2<f32>),
    BotRight(Vector2<f32>),
    TopRight(Vector2<f32>),
    Center(Vector2<f32>),
    TopCenter(Vector2<f32>),
    BotCenter(Vector2<f32>),
}
