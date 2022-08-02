use crate::{
    renderer::{Color, Renderer},
    vec2::Vec2,
};
use sdl2::{
    gfx::primitives::DrawRenderer,
    pixels::Color as Sdl2Color,
    render::{Canvas, RenderTarget},
};

pub struct SDL2CanvasWrapper<T: RenderTarget>(Canvas<T>, Sdl2Color);

impl<T: RenderTarget> From<sdl2::render::Canvas<T>> for SDL2CanvasWrapper<T> {
    fn from(canvas: sdl2::render::Canvas<T>) -> Self {
        Self(canvas, Sdl2Color::RGBA(0, 0, 0, 0))
    }
}

impl From<Color> for Sdl2Color {
    fn from(Color { r, g, b, a }: Color) -> Self {
        Self { r, g, b, a }
    }
}

impl<T: RenderTarget> Renderer for SDL2CanvasWrapper<T> {
    fn filled_circle(&mut self, center: Vec2, radius: f64) -> &mut Self {
        self.0
            .filled_circle(center.x as i16, center.y as i16, radius as i16, self.1)
            .expect("could not draw filled circle");

        self
    }

    fn line(&mut self, a: Vec2, b: Vec2) -> &mut Self {
        self.0
            .line(a.x as i16, a.y as i16, b.x as i16, b.y as i16, self.1)
            .expect("could not draw line");

        self
    }

    fn thick_line(&mut self, a: Vec2, b: Vec2, thickness: f64) -> &mut Self {
        self.0
            .thick_line(
                a.x as i16,
                a.y as i16,
                b.x as i16,
                b.y as i16,
                thickness as u8,
                self.1,
            )
            .expect("could not draw thick line");

        self
    }

    fn rectangle(&mut self, a: Vec2, b: Vec2) -> &mut Self {
        self.0
            .rectangle(a.x as i16, a.y as i16, b.x as i16, b.y as i16, self.1)
            .expect("could not draw rectangle");

        self
    }

    fn filled_rectangle(&mut self, a: Vec2, b: Vec2) -> &mut Self {
        self.0
            .box_(a.x as i16, a.y as i16, b.x as i16, b.y as i16, self.1)
            .expect("could not draw rectangle");

        self
    }

    fn filled_rounded_rectangle(&mut self, a: Vec2, b: Vec2, radius: f64) -> &mut Self {
        self.0
            .rounded_box(
                a.x as i16,
                a.y as i16,
                b.x as i16,
                b.y as i16,
                radius as i16,
                self.1,
            )
            .expect("Could not draw rectangle");

        self
    }

    fn polygon(&mut self, vertices: impl Iterator<Item = Vec2>) -> &mut Self {
        let n = vertices.size_hint().1.unwrap_or_default();
        let mut vx = Vec::<i16>::with_capacity(n);
        let mut vy = Vec::<i16>::with_capacity(n);

        for v in vertices {
            vx.push(v.x as i16);
            vy.push(v.y as i16);
        }

        self.0
            .polygon(&vx, &vy, self.1)
            .expect("could not draw polygon");

        self
    }

    fn text(&mut self, pos: Vec2, text: &str) -> &mut Self {
        self.0
            .string(pos.x as i16, pos.y as i16, text, self.1)
            .expect("could not draw text");

        self
    }

    fn size(&self) -> (usize, usize) {
        let (w, h) = self.0.logical_size();
        (w as usize, h as usize)
    }

    fn set_color(&mut self, color: Color) -> &mut Self {
        self.0.set_draw_color(color);
        self.1 = color.into();

        self
    }

    fn clear(&mut self) -> &mut Self {
        self.0.clear();
        self
    }

    fn finish(&mut self) {
        self.0.present();
    }
}
