use crate::vec2::Vec2;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    #[allow(non_snake_case)]
    pub const fn RGB(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[allow(non_snake_case)]
    pub const fn U32(c: u32) -> Self {
        unsafe { std::mem::transmute(c) }
    }

    #[allow(non_snake_case)]
    pub const fn RGBA(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn as_u32(self) -> u32 {
        unsafe { std::mem::transmute(self) }
    }

    pub const WHITE: Self = Self::RGB(255, 255, 255);
    pub const BLACK: Self = Self::RGB(0, 0, 0);
    pub const GRAY: Self = Self::RGB(128, 128, 128);
    pub const GREY: Self = Self::GRAY;
    pub const RED: Self = Self::RGB(255, 0, 0);
    pub const GREEN: Self = Self::RGB(0, 255, 0);
    pub const BLUE: Self = Self::RGB(0, 0, 255);
    pub const MAGENTA: Self = Self::RGB(255, 0, 255);
    pub const YELLOW: Self = Self::RGB(255, 255, 0);
    pub const CYAN: Self = Self::RGB(0, 255, 255);
}

pub trait Renderer {
    fn filled_circle(&mut self, center: Vec2, radius: f64) -> &mut Self;
    fn line(&mut self, a: Vec2, b: Vec2) -> &mut Self;
    fn thick_line(&mut self, a: Vec2, b: Vec2, thickness: f64) -> &mut Self;
    fn rectangle(&mut self, a: Vec2, b: Vec2) -> &mut Self;
    fn filled_rectangle(&mut self, a: Vec2, b: Vec2) -> &mut Self;
    fn filled_rounded_rectangle(&mut self, a: Vec2, b: Vec2, radius: f64) -> &mut Self;
    fn polygon(&mut self, vertices: impl Iterator<Item = Vec2>) -> &mut Self;
    fn text(&mut self, pos: Vec2, text: &str) -> &mut Self;

    fn size(&self) -> (usize, usize);

    fn width(&self) -> usize {
        self.size().0
    }

    fn height(&self) -> usize {
        self.size().1
    }

    fn set_color(&mut self, color: Color) -> &mut Self;
    fn clear(&mut self) -> &mut Self;

    fn finish(&mut self);
}
