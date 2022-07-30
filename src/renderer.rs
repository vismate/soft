use sdl2::pixels::Color;
use crate::vec2::Vec2;
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