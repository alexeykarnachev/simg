use std::ops::AddAssign;

use nalgebra::Vector2;

#[derive(Clone, Copy)]
pub struct Triangle {
    pub a: Vector2<f32>,
    pub b: Vector2<f32>,
    pub c: Vector2<f32>,
}

impl Triangle {
    pub fn new(a: Vector2<f32>, b: Vector2<f32>, c: Vector2<f32>) -> Self {
        Self { a, b, c }
    }

    pub fn to_vertices(&self) -> [Vector2<f32>; 3] {
        [self.a, self.b, self.c]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    bot_left: Vector2<f32>,
    top_right: Vector2<f32>,
}

impl Rectangle {
    pub fn translate(&self, translation: &Vector2<f32>) -> Rectangle {
        Rectangle {
            bot_left: self.bot_left + translation,
            top_right: self.top_right + translation,
        }
    }

    pub fn translate_assign(&mut self, translation: &Vector2<f32>) {
        self.bot_left.add_assign(translation);
        self.top_right.add_assign(translation);
    }

    pub fn get_size(&self) -> Vector2<f32> {
        self.top_right - self.bot_left
    }

    pub fn get_bot_left(&self) -> Vector2<f32> {
        self.bot_left
    }

    pub fn get_top_right(&self) -> Vector2<f32> {
        self.top_right
    }

    pub fn get_center(&self) -> Vector2<f32> {
        let mut center = self.bot_left;
        center += (self.top_right - self.bot_left) / 2.0;

        center
    }

    pub fn get_bot_center(&self) -> Vector2<f32> {
        let mut bot_center = self.bot_left;
        bot_center.x += (self.top_right.x - self.bot_left.x) / 2.0;

        bot_center
    }

    pub fn get_bot_right(&self) -> Vector2<f32> {
        let mut bot_right = self.bot_left;
        bot_right.x = self.top_right.x;

        bot_right
    }

    pub fn get_top_left(&self) -> Vector2<f32> {
        let mut top_left = self.bot_left;
        top_left.y = self.top_right.y;

        top_left
    }

    pub fn from_center(center: Vector2<f32>, size: Vector2<f32>) -> Self {
        let mut bot_left = center;
        bot_left -= size * 0.5;
        let top_right = bot_left + size;

        Self {
            bot_left,
            top_right,
        }
    }

    pub fn from_top_left(
        top_left: Vector2<f32>,
        size: Vector2<f32>,
    ) -> Self {
        let mut bot_left = top_left;
        bot_left.y -= size.y;
        let top_right = bot_left + size;

        Self {
            bot_left,
            top_right,
        }
    }

    pub fn from_bot_left(
        bot_left: Vector2<f32>,
        size: Vector2<f32>,
    ) -> Self {
        let top_right = bot_left + size;

        Self {
            bot_left,
            top_right,
        }
    }

    pub fn from_bot_center(
        bot_center: Vector2<f32>,
        size: Vector2<f32>,
    ) -> Self {
        let mut bot_left = bot_center;
        bot_left.x -= 0.5 * size.x;
        let top_right = bot_left + size;

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
}
