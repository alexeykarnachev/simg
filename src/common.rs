use nalgebra::{vector, Matrix4, Point2, Point3, Vector3};

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

#[derive(Debug, Clone)]
pub struct Transformation {
    translation: Vector3<f32>,
    scale: Vector3<f32>,
    rotation: Vector3<f32>,
}

impl Default for Transformation {
    fn default() -> Self {
        Self::new(
            vector![0.0, 0.0, 0.0],
            vector![1.0, 1.0, 1.0],
            vector![0.0, 0.0, 0.0],
        )
    }
}

impl Transformation {
    pub fn new(
        translation: Vector3<f32>,
        scale: Vector3<f32>,
        rotation: Vector3<f32>,
    ) -> Self {
        Self { translation, scale, rotation }
    }

    pub fn get_mat(&self) -> Matrix4<f32> {
        let t = Matrix4::new_translation(&self.translation);
        let s = Matrix4::new_nonuniform_scaling(&self.scale);
        let r = Matrix4::new_rotation(self.rotation);

        t * r * s
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Projection {
    ProjScreen,
    Proj2D {
        eye: Point2<f32>,
        zoom: f32,
        rotation: f32,
    },
    Proj3D {
        eye: Point3<f32>,
        target: Point3<f32>,
        fovy: f32,
    },
}

impl Projection {
    pub fn get_mat(&self, window_size: (u32, u32)) -> Matrix4<f32> {
        use Projection::*;

        let mat = match self {
            ProjScreen => Matrix4::new_orthographic(
                0.0,
                window_size.0 as f32,
                0.0,
                window_size.1 as f32,
                0.0,
                1.0,
            ),
            Proj2D { eye, zoom, rotation } => {
                let mut scale = Matrix4::identity();
                scale[(0, 0)] = *zoom;
                scale[(1, 1)] = *zoom;

                let mut translation = Matrix4::identity();
                translation[(0, 3)] = -eye.x;
                translation[(1, 3)] = -eye.y;

                let rotation = Matrix4::new_rotation(Vector3::new(
                    0.0, 0.0, -rotation,
                ));

                let view = rotation * scale * translation;

                let projection = Matrix4::new_orthographic(
                    window_size.0 as f32 / -2.0,
                    window_size.0 as f32 / 2.0,
                    window_size.1 as f32 / -2.0,
                    window_size.1 as f32 / 2.0,
                    0.0,
                    1.0,
                );

                projection * view
            }
            Proj3D { eye, target, fovy } => {
                let fovy = fovy.to_radians();
                let up = Vector3::new(0.0, 1.0, 0.0);
                let view = Matrix4::look_at_rh(eye, target, &up);
                let aspect = window_size.0 as f32 / window_size.1 as f32;
                let projection =
                    Matrix4::new_perspective(aspect, fovy, 0.1, 1000.0);

                projection * view
            }
        };

        mat
    }
}
