mod vec2;

use vec2::Vec2;

fn main() {
    let x = Vec2::new(0.5, 0.5);
    let y = Vec2::from_angle_deg(32.0);

    let w = Vec2::null().angle_deg(y.rotate_deg(32.0));

    println!("{:?}", w);
}
