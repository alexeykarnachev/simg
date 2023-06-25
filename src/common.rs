use nalgebra::Point2;

pub enum PivotType {
    BotLeft,
    TopLeft,
    BotRight,
    TopRight,
    Center,
    TopCenter,
    BotCenter,
    LeftCenter,
    RightCenter,
}

pub struct Pivot {
    pub ty: PivotType,
    pub p: Point2<f32>,
}

use PivotType::*;

impl Pivot {
    pub fn new(ty: PivotType, p: Point2<f32>) -> Self {
        Self { ty, p }
    }

    pub fn bot_left(p: Point2<f32>) -> Self {
        Self::new(BotLeft, p)
    }

    pub fn top_left(p: Point2<f32>) -> Self {
        Self::new(TopLeft, p)
    }

    pub fn bot_right(p: Point2<f32>) -> Self {
        Self::new(BotRight, p)
    }

    pub fn top_right(p: Point2<f32>) -> Self {
        Self::new(TopRight, p)
    }

    pub fn center(p: Point2<f32>) -> Self {
        Self::new(Center, p)
    }

    pub fn top_center(p: Point2<f32>) -> Self {
        Self::new(TopCenter, p)
    }

    pub fn bot_center(p: Point2<f32>) -> Self {
        Self::new(BotCenter, p)
    }

    pub fn left_center(p: Point2<f32>) -> Self {
        Self::new(LeftCenter, p)
    }

    pub fn right_center(p: Point2<f32>) -> Self {
        Self::new(RightCenter, p)
    }
}
