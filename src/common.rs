use nalgebra::Point2;

use crate::color::Color;
use std::collections::HashMap;

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

pub enum ProgramArg {
    FloatArg(f32),
    ColorArg(Color),
}

pub struct Program {
    pub idx: u32,
    pub args: HashMap<String, ProgramArg>,
}

impl Program {
    pub fn new(idx: u32) -> Self {
        Self { idx, args: HashMap::with_capacity(16) }
    }

    pub fn set_arg(&mut self, name: &str, arg: ProgramArg) {
        self.args.insert(name.to_string(), arg);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Texture {
    pub idx: u32,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(idx: u32, width: u32, height: u32) -> Self {
        Self { idx, width, height }
    }
}
