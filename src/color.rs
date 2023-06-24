#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn as_arr(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn gray(c: f32, a: f32) -> Self {
        Self { r: c, g: c, b: c, a }
    }

    pub fn with_alpha(&self, a: f32) -> Self {
        let mut color = self.clone();
        color.a = a;

        color
    }

    pub fn add_white(&self, c: f32) -> Self {
        Self::new(self.r + c, self.g + c, self.b + c, self.a)
    }

    pub fn lerp(&self, k: f32, other: Color) -> Self {
        Self {
            r: k * self.r + (1.0 - k) * other.r,
            g: k * self.g + (1.0 - k) * other.g,
            b: k * self.b + (1.0 - k) * other.b,
            a: k * self.a + (1.0 - k) * other.a,
        }
    }
}

pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
pub const GRAY: Color = Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
pub const YELLOW: Color = Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
pub const ORANGE: Color = Color { r: 0.93, g: 0.44, b: 0.08, a: 1.0 };
pub const PRUSSIAN_BLUE: Color =
    Color { r: 0.0, g: 0.19, b: 0.36, a: 1.0 };
