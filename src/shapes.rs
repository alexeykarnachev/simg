use crate::common::*;
use std::ops::AddAssign;

use nalgebra::{Point2, Point3, Vector2};

pub const CIRCLE_N_TRIANGLES: usize = 16;
const UNIT_CIRCLE_POINTS: [Point2<f32>; CIRCLE_N_TRIANGLES] = [
    Point2::new(1.0, 0.0),
    Point2::new(0.9238795325112867, 0.3826834323650898),
    Point2::new(0.7071067811865475, 0.7071067811865476),
    Point2::new(0.38268343236508967, 0.9238795325112867),
    Point2::new(0.0, 1.0),
    Point2::new(-0.3826834323650898, 0.9238795325112867),
    Point2::new(-0.7071067811865476, 0.7071067811865475),
    Point2::new(-0.9238795325112867, 0.38268343236508967),
    Point2::new(-1.0, 0.0),
    Point2::new(-0.9238795325112867, -0.3826834323650898),
    Point2::new(-0.7071067811865475, -0.7071067811865476),
    Point2::new(-0.38268343236508967, -0.9238795325112867),
    Point2::new(0.0, -1.0),
    Point2::new(0.3826834323650898, -0.9238795325112867),
    Point2::new(0.7071067811865476, -0.7071067811865475),
    Point2::new(0.9238795325112867, -0.38268343236508967),
];

#[derive(Clone, Copy)]
pub struct Line {
    pub s: Point2<f32>,
    pub e: Point2<f32>,
}

impl Line {
    pub fn new(s: Point2<f32>, e: Point2<f32>) -> Self {
        Self { s, e }
    }
}

#[derive(Clone, Copy)]
pub struct Triangle {
    pub a: Point3<f32>,
    pub b: Point3<f32>,
    pub c: Point3<f32>,
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            a: Point3::origin(),
            b: Point3::origin(),
            c: Point3::origin(),
        }
    }
}

impl Triangle {
    pub fn new(a: Point3<f32>, b: Point3<f32>, c: Point3<f32>) -> Self {
        Self { a, b, c }
    }

    pub fn new_2d(a: Point2<f32>, b: Point2<f32>, c: Point2<f32>) -> Self {
        let a = Point3::new(a.x, a.y, 0.0);
        let b = Point3::new(b.x, b.y, 0.0);
        let c = Point3::new(c.x, c.y, 0.0);
        Self { a, b, c }
    }

    pub fn to_vertices(&self) -> [Point3<f32>; 3] {
        [self.a, self.b, self.c]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    bot_left: Point2<f32>,
    top_right: Point2<f32>,
}

impl Rectangle {
    pub fn new(bot_left: Point2<f32>, top_right: Point2<f32>) -> Self {
        Self { bot_left, top_right }
    }

    pub fn zeros() -> Self {
        Self {
            bot_left: Point2::origin(),
            top_right: Point2::origin(),
        }
    }

    pub fn translate(&self, translation: &Vector2<f32>) -> Rectangle {
        let mut rect = *self;
        rect.translate_assign(translation);

        rect
    }

    pub fn translate_assign(&mut self, translation: &Vector2<f32>) {
        self.bot_left.add_assign(translation);
        self.top_right.add_assign(translation);
    }

    pub fn translate_x_assign(&mut self, translation: f32) {
        self.bot_left.x += translation;
        self.top_right.x += translation;
    }

    pub fn get_size(&self) -> Vector2<f32> {
        self.top_right - self.bot_left
    }

    pub fn get_width(&self) -> f32 {
        self.top_right.x - self.bot_left.x
    }

    pub fn get_height(&self) -> f32 {
        self.top_right.y - self.bot_left.y
    }

    pub fn get_center_x(&self) -> f32 {
        (self.bot_left.x + self.top_right.x) / 2.0
    }

    pub fn get_min_x(&self) -> f32 {
        self.bot_left.x
    }

    pub fn get_max_x(&self) -> f32 {
        self.top_right.x
    }

    pub fn get_max_y(&self) -> f32 {
        self.top_right.y
    }

    pub fn get_min_y(&self) -> f32 {
        self.bot_left.y
    }

    pub fn get_bot_left(&self) -> Point2<f32> {
        self.bot_left
    }

    pub fn get_top_right(&self) -> Point2<f32> {
        self.top_right
    }

    pub fn get_center(&self) -> Point2<f32> {
        let mut center = self.bot_left;
        center += (self.top_right - self.bot_left) / 2.0;

        center
    }

    pub fn get_top_center(&self) -> Point2<f32> {
        let mut top_center = self.top_right;
        top_center.x -= (self.top_right.x - self.bot_left.x) / 2.0;

        top_center
    }

    pub fn get_bot_center(&self) -> Point2<f32> {
        let mut bot_center = self.bot_left;
        bot_center.x += (self.top_right.x - self.bot_left.x) / 2.0;

        bot_center
    }

    pub fn get_bot_right(&self) -> Point2<f32> {
        let mut bot_right = self.bot_left;
        bot_right.x = self.top_right.x;

        bot_right
    }

    pub fn get_top_left(&self) -> Point2<f32> {
        let mut top_left = self.bot_left;
        top_left.y = self.top_right.y;

        top_left
    }

    pub fn get_left_center(&self) -> Point2<f32> {
        let mut left_center = self.bot_left;
        left_center.y += self.get_height() * 0.5;

        left_center
    }

    pub fn from_pivot(pivot: Pivot, size: Vector2<f32>) -> Self {
        use PivotType::*;

        match pivot.ty {
            BotLeft => Self::from_bot_left(pivot.p, size),
            TopLeft => Self::from_top_left(pivot.p, size),
            BotRight => Self::from_bot_right(pivot.p, size),
            TopRight => Self::from_top_right(pivot.p, size),
            Center => Self::from_center(pivot.p, size),
            TopCenter => Self::from_top_center(pivot.p, size),
            BotCenter => Self::from_bot_center(pivot.p, size),
            LeftCenter => Self::from_left_center(pivot.p, size),
            RightCenter => Self::from_right_center(pivot.p, size),
        }
    }

    pub fn from_center(center: Point2<f32>, size: Vector2<f32>) -> Self {
        let mut bot_left = center;
        bot_left -= size * 0.5;

        Self::from_bot_left(bot_left, size)
    }

    pub fn from_left_center(
        left_center: Point2<f32>,
        size: Vector2<f32>,
    ) -> Self {
        let mut bot_left = left_center;
        bot_left.y -= size.y * 0.5;

        Self::from_bot_left(bot_left, size)
    }

    pub fn from_right_center(
        right_center: Point2<f32>,
        size: Vector2<f32>,
    ) -> Self {
        let mut bot_left = right_center;
        bot_left.y -= size.y * 0.5;
        bot_left.x -= size.x;

        Self::from_bot_left(bot_left, size)
    }

    pub fn from_top_left(
        top_left: Point2<f32>,
        size: Vector2<f32>,
    ) -> Self {
        let mut bot_left = top_left;
        bot_left.y -= size.y;

        Self::from_bot_left(bot_left, size)
    }

    pub fn from_top_right(
        top_right: Point2<f32>,
        size: Vector2<f32>,
    ) -> Self {
        let mut bot_left = top_right;
        bot_left -= size;

        Self::from_bot_left(bot_left, size)
    }

    pub fn from_top_center(
        top_center: Point2<f32>,
        size: Vector2<f32>,
    ) -> Self {
        let mut bot_left = top_center;
        bot_left.y -= size.y;
        bot_left.x -= size.x * 0.5;

        Self::from_bot_left(bot_left, size)
    }

    pub fn from_bot_left(
        bot_left: Point2<f32>,
        size: Vector2<f32>,
    ) -> Self {
        let top_right = bot_left + size;

        Self { bot_left, top_right }
    }

    pub fn from_bot_right(
        bot_right: Point2<f32>,
        size: Vector2<f32>,
    ) -> Self {
        let mut bot_left = bot_right;
        bot_left.x -= size.x;

        Self::from_bot_left(bot_left, size)
    }

    pub fn from_bot_center(
        bot_center: Point2<f32>,
        size: Vector2<f32>,
    ) -> Self {
        let mut bot_left = bot_center;
        bot_left.x -= 0.5 * size.x;

        Self::from_bot_left(bot_left, size)
    }

    pub fn get_triangles(&self) -> [Triangle; 2] {
        [
            Triangle::new_2d(
                self.get_top_left(),
                self.get_bot_left(),
                self.get_top_right(),
            ),
            Triangle::new_2d(
                self.get_top_right(),
                self.get_bot_left(),
                self.get_bot_right(),
            ),
        ]
    }

    pub fn get_vertices(&self) -> [Point2<f32>; 4] {
        [
            self.get_bot_left(),
            self.get_bot_right(),
            self.get_top_right(),
            self.get_top_left(),
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Circle {
    pub center: Point2<f32>,
    pub radius: f32,
}

impl Circle {
    pub fn zeros() -> Self {
        Self { center: Point2::origin(), radius: 0.0 }
    }

    pub fn new(center: Point2<f32>, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn get_left(&self) -> Point2<f32> {
        let mut left = self.center;
        left.x -= self.radius;

        left
    }

    pub fn get_right(&self) -> Point2<f32> {
        let mut right = self.center;
        right.x += self.radius;

        right
    }

    pub fn get_top(&self) -> Point2<f32> {
        let mut top = self.center;
        top.y += self.radius;

        top
    }

    pub fn get_bot(&self) -> Point2<f32> {
        let mut bot = self.center;
        bot.y -= self.radius;

        bot
    }

    pub fn get_min_x(&self) -> f32 {
        self.center.x - self.radius
    }

    pub fn get_max_x(&self) -> f32 {
        self.center.x + self.radius
    }

    pub fn get_max_y(&self) -> f32 {
        self.center.y + self.radius
    }

    pub fn get_min_y(&self) -> f32 {
        self.center.y - self.radius
    }

    pub fn from_bot(bot: Point2<f32>, radius: f32) -> Self {
        let mut center = bot;
        center.y += radius;

        Self { center, radius }
    }

    pub fn to_triangles(&self) -> [Triangle; CIRCLE_N_TRIANGLES] {
        let mut triangles =
            [(); CIRCLE_N_TRIANGLES].map(|_| Triangle::default());

        let a = self.center;
        for i in 0..CIRCLE_N_TRIANGLES {
            let j = (i + 1) % (CIRCLE_N_TRIANGLES);
            let b = UNIT_CIRCLE_POINTS[j] * self.radius + a.coords;
            let c = UNIT_CIRCLE_POINTS[i] * self.radius + a.coords;
            triangles[i] = Triangle::new_2d(a, b, c);
        }

        triangles
    }
}
