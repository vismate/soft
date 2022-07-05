mod vec2;

use sdl2::{
    event::Event,
    gfx::{framerate::FPSManager, primitives::DrawRenderer},
    keyboard::Keycode,
    mouse::MouseButton,
    pixels::Color,
};
use vec2::Vec2;

#[derive(Clone)]
struct Particle {
    pos: Vec2,
    vel: Vec2,
    acc: Vec2,
}

impl Particle {
    pub const R: f64 = 8.5;
    pub const SPACING: f64 = 25.0;
    pub const DIAG_SQR: f64 = 2.0 * Particle::SPACING * Particle::SPACING;

    pub fn new(x: f64, y: f64) -> Self {
        Self {
            pos: Vec2::new(x, y),
            vel: Vec2::null(),
            acc: Vec2::null(),
        }
    }
}

struct Spring {
    a: usize,
    b: usize,
    l0: f64,
}

impl Spring {
    pub const KS: f64 = 5000.0;
    pub const KD: f64 = 100.0;

    pub fn new(a: usize, b: usize, l0: f64) -> Self {
        Self { a, b, l0 }
    }
}

struct Edge {
    start: Vec2,
    end: Vec2,
}

impl Edge {
    const R: f64 = 5.0;

    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self { start, end }
    }
}

fn spawn_rect(
    w: usize,
    h: usize,
    x: f64,
    y: f64,
    particles: &mut Vec<Particle>,
    springs: &mut Vec<Spring>,
    boundaries: &mut Vec<usize>,
) {
    particles.reserve(w * h);
    springs.reserve(w * h * 4);
    boundaries.reserve(2 * w + 2 * h);

    for i in 0..w {
        for j in 0..h {
            particles.push(Particle::new(
                i as f64 * Particle::SPACING + x,
                j as f64 * Particle::SPACING + y,
            ));

            let ind = particles.len() - 1;
            if i < w - 1 {
                springs.push(Spring::new(ind, ind + h, Particle::SPACING))
            }
            if j < h - 1 {
                springs.push(Spring::new(ind, ind + 1, Particle::SPACING))
            }
            if i < w - 1 && j < h - 1 {
                springs.push(Spring::new(ind, ind + h + 1, Particle::DIAG_SQR.sqrt()))
            }
            if i > 0 && j < h - 1 {
                springs.push(Spring::new(ind, ind - h + 1, Particle::DIAG_SQR.sqrt()))
            }

            if i == 0 || i == w - 1 || j == 0 || j == h - 1 {
                boundaries.push(ind)
            }
        }
    }
}

fn update_spring(spring: &Spring, particles: &mut [Particle]) {
    let p1 = &particles[spring.a];
    let p2 = &particles[spring.b];

    let diff = p2.pos - p1.pos;
    let diff_norm = diff.normalize();

    let fs = (diff.len() - spring.l0) * Spring::KS;
    let fd = diff_norm.dot(p2.vel - p1.vel) * Spring::KD;

    let f = (fs + fd) * diff_norm;

    particles[spring.a].acc += f;
    particles[spring.b].acc -= f;
}

fn integrate(particle: &mut Particle, dt: f64) {
    particle.pos += particle.vel * dt + 0.5 * particle.acc * dt * dt;
    particle.vel += particle.acc * dt;

    particle.acc = Vec2::null();
}

fn constrain(particle: &mut Particle, w: f64, h: f64) {
    if particle.pos.x + Particle::R > w {
        particle.pos.x = w - Particle::R;
        particle.vel.x *= -0.5;
    } else if particle.pos.x - Particle::R < 0.0 {
        particle.pos.x = Particle::R;
        particle.vel.x *= -0.5;
    }

    if particle.pos.y + Particle::R > h {
        particle.pos.y = h - Particle::R;
        particle.vel.y *= -0.5;
    } else if particle.pos.y - Particle::R < 0.0 {
        particle.pos.y = Particle::R;
        particle.vel.y *= -0.5;
    }
}

fn collide_edge(particle: &mut Particle, edge: &Edge) {
    let line1 = edge.end - edge.start;
    let line2 = particle.pos - edge.start;

    let edge_len = line1.len_sqr();

    let t = line1.dot(line2).clamp(0.0, edge_len) / edge_len;

    let closest_point = edge.start + t * line1;

    let d = Particle::R + Edge::R - particle.pos.dist(closest_point);


    if d >= 0.0 {
        particle.vel = particle.vel.reflect(line1.normal()) * 0.5;
        particle.pos += particle.vel.normalize() * d;
    }
}

fn collide_particle(this: &mut Particle, that: &mut Particle) {
    let diff = that.pos - this.pos;

    const D_SQR: f64 = Particle::R * Particle::R * 4.0;

    if diff.len_sqr() < D_SQR {
        // Static resolution
        let midp = (this.pos + that.pos) * 0.5;

        let offset = Particle::R * diff.normalize();

        this.pos = midp - offset;
        that.pos = midp + offset;

        // Dynamic resolution
        let diff = that.pos - this.pos;
        let diff_norm = diff.normalize();

        let vel_offset = (that.vel.dot(diff_norm) - this.vel.dot(diff_norm)) * diff_norm;

        this.vel += vel_offset;
        that.vel -= vel_offset;
    }
}

const GRID: usize = Particle::R as usize * 8;
const WIDTH: f64 = 1920.0;
const HEIGHT: f64 = 1080.0;
#[inline(always)]
fn bucket<'a>(particle: &Particle, buckets: &'a mut [Vec<usize>]) -> &'a mut Vec<usize> {
    &mut buckets[(particle.pos.x as usize / GRID) % buckets.len()]
}

fn main() {
    let mut particles = vec![];
    let mut springs = vec![];
    let mut boundaries = vec![];
    let mut edges = vec![];
    let mut buckets: Vec<Vec<usize>> = vec![];

    buckets.resize(WIDTH as usize / GRID + 1, vec![]);

    edges.push(Edge::new(Vec2::new(0.0, 400.0), Vec2::new(280.0, 400.0)));
    edges.push(Edge::new(Vec2::new(400.0, 700.0), Vec2::new(680.0, 700.0)));

    edges.push(Edge::new(
        Vec2::new(750.0, 1200.0),
        Vec2::new(1920.0, 800.0),
    ));

    let ctx = sdl2::init().unwrap();
    let video = ctx.video().unwrap();
    let timer = ctx.timer().unwrap();

    let window = video
        .window("soft", WIDTH as u32, HEIGHT as u32)
        .fullscreen()
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(11, 14, 20));
    canvas.clear();
    canvas.present();

    let mut events = ctx.event_pump().unwrap();

    let mut fps_manager = FPSManager::new();
    fps_manager.set_framerate(60).unwrap();

    let dt = 0.00125f64;
    let mut dt_acc = 0.0f64;

    let mut fps = 0u8;

    let mut simulate = false;
    let mut rect_start: Option<Vec2> = None;

    'running: loop {
        let begin = timer.ticks();

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    simulate = !simulate;
                }
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Right,
                    x,
                    y,
                    ..
                } => {
                    rect_start = Some(Vec2::new(x as f64, y as f64));
                }
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Right,
                    x,
                    y,
                    ..
                } => {
                    spawn_rect(
                        ((rect_start.unwrap().x - x as f64).abs() / Particle::SPACING) as usize + 1,
                        ((rect_start.unwrap().y - y as f64).abs() / Particle::SPACING) as usize + 1,
                        f64::min(rect_start.unwrap().x, x as f64),
                        f64::min(rect_start.unwrap().y, y as f64),
                        &mut particles,
                        &mut springs,
                        &mut boundaries,
                    );

                    rect_start = None;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Delete),
                    ..
                } => {
                    particles.clear();
                    springs.clear();
                    boundaries.clear();
                }

                _ => {}
            }
        }

        if simulate {
            while dt_acc >= dt {
                for (i, particle) in particles.iter().enumerate() {
                    bucket(&particle, &mut buckets).push(i);
                }

                for spring in &springs {
                    update_spring(spring, &mut particles);
                }

                for i in 0..particles.len() {
                    let mut particle = particles[i].clone();

                    for j in bucket(&particle, &mut buckets) {
                        if i != *j {
                            collide_particle(&mut particle, &mut particles[*j]);
                        }
                    }

                    //Gravity
                    particle.acc += Vec2::new(0.0, 350.0);

                    integrate(&mut particle, dt);

                    particles[i] = particle;
                }

                for i in &boundaries {
                    let mut particle = &mut particles[*i];
                    let size = canvas.output_size().unwrap();
                    constrain(&mut particle, size.0 as f64, size.1 as f64);

                    for edge in &edges {
                        collide_edge(&mut particle, edge);
                    }
                }

                for b in &mut buckets {
                    b.clear();
                }

                dt_acc -= dt;
            }
        }

        canvas.set_draw_color(Color::RGB(11, 14, 20));
        canvas.clear();

        for spring in &springs {
            let p1 = &particles[spring.a];
            let p2 = &particles[spring.b];
            canvas
                .line(
                    p1.pos.x as i16,
                    p1.pos.y as i16,
                    p2.pos.x as i16,
                    p2.pos.y as i16,
                    Color::CYAN,
                )
                .unwrap();
        }

        for particle in &particles {
            canvas
                .filled_circle(
                    particle.pos.x as i16,
                    particle.pos.y as i16,
                    Particle::R as i16,
                    Color::YELLOW,
                )
                .unwrap();
        }

        for edge in &edges {
            canvas
                .thick_line(
                    edge.start.x as i16,
                    edge.start.y as i16,
                    edge.end.x as i16,
                    edge.end.y as i16,
                    Edge::R as u8 * 2,
                    Color::RGB(44, 56, 80),
                )
                .unwrap();
        }

        if let Some(Vec2 { x, y }) = rect_start {
            let mouse_state = events.mouse_state();
            canvas
                .rectangle(
                    x as i16,
                    y as i16,
                    mouse_state.x() as i16,
                    mouse_state.y() as i16,
                    Color::RGB(44, 56, 80),
                )
                .unwrap();

            let w = ((x - mouse_state.x() as f64).abs() / Particle::SPACING) as i16 + 1;
            let h = ((y - mouse_state.y() as f64).abs() / Particle::SPACING) as i16 + 1;
            canvas
                .string(
                    x as i16 + 10,
                    y as i16 - 10,
                    format!("{w} x {h}").as_str(),
                    Color::RGB(44, 56, 80),
                )
                .unwrap();
        }

        canvas
            .string(15, 15, format!("{fps} FPS").as_str(), Color::CYAN)
            .unwrap();

        canvas
            .string(
                10,
                35,
                format!("{} particles", particles.len()).as_str(),
                Color::RGB(44, 56, 80),
            )
            .unwrap();

        canvas
            .string(
                10,
                45,
                format!("{} springs", springs.len()).as_str(),
                Color::RGB(44, 56, 80),
            )
            .unwrap();

        canvas.present();
        fps_manager.delay();

        let dur = (timer.ticks() - begin) as f64;
        let new_fps = (1000.0 / dur) as u8;
        if new_fps.abs_diff(fps) > 10 {
            fps = new_fps;
        }

        if simulate {
            dt_acc += dur / 1000.0;
        }
    }
}
