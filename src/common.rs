use nalgebra::{point, vector, Matrix4, Point2, Point3, Vector3};

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

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Camera {
    Cam2D {
        position: Point2<f32>,
        rotation: f32,
    },
    Cam3D {
        position: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
    },
}

impl Camera {
    pub fn new_2d(position: Point2<f32>, rotation: f32) -> Self {
        Self::Cam2D { position, rotation }
    }

    pub fn new_3d(
        position: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
    ) -> Self {
        Self::Cam3D { position, target, up }
    }

    pub fn new_screen(window_size: (u32, u32)) -> Self {
        Camera::Cam2D {
            position: point![
                window_size.0 as f32 / 2.0,
                window_size.1 as f32 / 2.0
            ],
            rotation: 0.0,
        }
    }

    pub fn new_origin_2d() -> Self {
        Camera::Cam2D {
            position: point![0.0, 0.0],
            rotation: 0.0,
        }
    }

    pub fn get_mat(&self) -> Matrix4<f32> {
        use Camera::*;

        match self {
            Cam2D { position, rotation } => {
                let mut translation = Matrix4::identity();
                translation[(0, 3)] = -position.x;
                translation[(1, 3)] = -position.y;

                let rotation = Matrix4::new_rotation(Vector3::new(
                    0.0, 0.0, -rotation,
                ));

                rotation * translation
            }
            Cam3D { position, target, up } => {
                Matrix4::look_at_rh(position, target, up)
            }
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Projection {
    Orthographic {
        view_width: f32,
        view_height: f32,
        znear: f32,
        zfar: f32,
    },
    Perspective {
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    },
}

impl Projection {
    pub fn new_orthographic(
        view_width: f32,
        view_height: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self::Orthographic { view_width, view_height, znear, zfar }
    }

    pub fn new_perspective(
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self::Perspective { aspect, fovy, znear, zfar }
    }

    pub fn new_screen(window_size: (u32, u32)) -> Self {
        Projection::Orthographic {
            view_width: window_size.0 as f32,
            view_height: window_size.1 as f32,
            znear: 0.0,
            zfar: 1.0,
        }
    }

    pub fn new_2d(view_width: f32, view_height: f32) -> Self {
        Projection::Orthographic {
            view_width,
            view_height,
            znear: 0.0,
            zfar: 1.0,
        }
    }

    pub fn get_mat(&self) -> Matrix4<f32> {
        use Projection::*;
        match self {
            Orthographic { view_width, view_height, znear, zfar } => {
                Matrix4::new_orthographic(
                    view_width / -2.0,
                    view_width / 2.0,
                    view_height / -2.0,
                    view_height / 2.0,
                    *znear,
                    *zfar,
                )
            }
            Perspective { aspect, fovy, znear, zfar } => {
                Matrix4::new_perspective(*aspect, *fovy, *znear, *zfar)
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum Material {
    #[default]
    VertexColor,
    BlinnPhong {
        shininess: f32,
    },
}
