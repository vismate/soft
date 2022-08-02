use crate::{
    consts::{HEIGHT, WIDTH},
    renderer::{Color, Renderer},
    vec2::Vec2,
};

use serde::{Deserialize, Serialize};

macro_rules! SQR {
    ($e:expr) => {
        $e * $e
    };
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Particle {
    pos: Vec2,
    vel: Vec2,
    acc: Vec2,
}

impl Particle {
    pub const R: f64 = 7.25;
    pub const SPACING: f64 = 21.0;
    pub const DIAG_SQR: f64 = 2.0 * SQR!(Particle::SPACING);

    pub fn new(x: f64, y: f64) -> Self {
        Self {
            pos: Vec2::new(x, y),
            vel: Vec2::null(),
            acc: Vec2::null(),
        }
    }

    pub fn collide(&mut self, other: &mut Self) {
        let diff = other.pos - self.pos;
        let diff_len_sqr = diff.len_sqr();

        if SQR!(2.0 * Particle::R) >= diff_len_sqr {
            // Static resolution
            let diff_len = diff_len_sqr.sqrt();
            let offset = 0.5 * (2.0 * Particle::R - diff_len) * (diff / diff_len);
            self.pos -= offset;
            other.pos += offset;

            // Dynamic resolution
            let diff_norm = (other.pos - self.pos) / (2.0 * Particle::R);
            let vel_offset = (self.vel.dot(diff_norm) - other.vel.dot(diff_norm)) * diff_norm;

            self.vel -= vel_offset;
            other.vel += vel_offset;
        }
    }

    pub fn integrate(&mut self, dt: f64) {
        self.pos += self.vel * dt + 0.5 * self.acc * dt * dt;
        self.vel += self.acc * dt;

        self.acc = Vec2::null();
    }
}

#[derive(Serialize, Deserialize)]
struct Spring {
    a: usize,
    b: usize,
    l0: f64,
}

impl Spring {
    pub const KS: f64 = 6000.0;
    pub const KD: f64 = 100.0;

    pub fn new(a: usize, b: usize, l0: f64) -> Self {
        Self { a, b, l0 }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Edge {
    start: Vec2,
    line: Vec2,
    len_sqr: f64,
}

impl Edge {
    pub const R: f64 = 1.5 * Particle::R;
    const FRICTION: f64 = 0.990;

    pub fn new(start: Vec2, end: Vec2) -> Self {
        let line = end - start;
        Self {
            start,
            line,
            len_sqr: line.len_sqr(),
        }
    }

    pub fn get_start(&self) -> Vec2 {
        self.start
    }

    pub fn get_end(&self) -> Vec2 {
        self.start + self.line
    }

    pub fn set_start(&mut self, start: Vec2) {
        self.line += self.start - start;
        self.len_sqr = self.line.len_sqr();
        self.start = start;
    }

    pub fn set_end(&mut self, end: Vec2) {
        self.line = end - self.start;
        self.len_sqr = self.line.len_sqr();
    }

    pub fn collide(&self, particle: &mut Particle) {
        let line2 = particle.pos - self.start;
        let t = self.line.dot(line2).clamp(0.0, self.len_sqr) / self.len_sqr;

        let closest_point = self.start + t * self.line;

        let diff = particle.pos - closest_point;
        let diff_len_sqr = diff.len_sqr();

        if diff_len_sqr <= SQR!(Particle::R + Edge::R) {
            let diff_len = diff_len_sqr.sqrt();
            particle.pos += ((Particle::R + Edge::R) - diff_len) * (diff / diff_len);

            let tangent = (particle.pos - closest_point) / (Edge::R + Particle::R);
            let dp = particle.vel.dot(tangent);

            particle.vel = (particle.vel - (dp * tangent) * 1.50) * Self::FRICTION;
        }
    }
}
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
struct ObjectDescriptor {
    particle_start: usize,
    particle_end: usize,

    spring_start: usize,
    spring_end: usize,

    boundary_start: usize,
    boundary_end: usize,
}
#[allow(dead_code)]
impl ObjectDescriptor {
    pub fn new(
        particle_start: usize,
        particle_end: usize,
        spring_start: usize,
        spring_end: usize,
        boundary_start: usize,
        boundary_end: usize,
    ) -> Self {
        Self {
            particle_start,
            particle_end,
            spring_start,
            spring_end,
            boundary_start,
            boundary_end,
        }
    }

    pub fn particles_len(&self) -> usize {
        self.particle_end - self.particle_start
    }

    pub fn particles_range(&self) -> std::ops::Range<usize> {
        self.particle_start..self.particle_end
    }

    pub fn springs_len(&self) -> usize {
        self.spring_end - self.spring_start
    }

    pub fn springs_range(&self) -> std::ops::Range<usize> {
        self.spring_start..self.spring_end
    }

    pub fn boundaries_len(&self) -> usize {
        self.boundary_end - self.boundary_start
    }

    pub fn boundaries_range(&self) -> std::ops::Range<usize> {
        self.boundary_start..self.boundary_end
    }
}

#[derive(Serialize, Deserialize)]
pub struct World {
    particles: Vec<Particle>,
    springs: Vec<Spring>,
    boundaries: Vec<usize>,
    objects: Vec<ObjectDescriptor>,
    edges: Vec<Edge>,
    buckets: Vec<Vec<usize>>,
    dt_acc: f64,
}

impl World {
    const DT: f64 = 0.00125;
    const GRID: f64 = HEIGHT / (Particle::R * 2.0);
    const GRAVITY: Vec2 = Vec2::new(0.0, 350.0);

    pub fn new() -> Self {
        let mut world = World {
            particles: vec![],
            springs: vec![],
            boundaries: vec![],
            objects: vec![],
            edges: vec![],
            buckets: vec![],
            dt_acc: 0.0,
        };

        world.buckets.resize(SQR!(Self::GRID) as usize, vec![]);
        world
    }

    #[allow(clippy::unused_self)]
    pub fn can_add_edge(&self, start: Vec2, end: Vec2) -> bool {
        start != end
    }

    pub fn add_edge(&mut self, start: Vec2, end: Vec2) -> Result<(), &'static str> {
        if !self.can_add_edge(start, end) {
            return Err("cant add edge, length cannot be 0");
        }
        self.edges.push(Edge::new(start, end));
        Ok(())
    }

    #[allow(clippy::unused_self)]
    pub fn can_spawn_rect(&self, w: usize, h: usize) -> bool {
        w >= 2 && h >= 2
    }

    pub fn spawn_rect(&mut self, w: usize, h: usize, x: f64, y: f64) -> Result<(), (usize, usize)> {
        if !self.can_spawn_rect(w, h) {
            return Err((w, h));
        }

        self.particles.reserve(w * h);
        self.springs.reserve(w * h * 4);
        self.boundaries.reserve(2 * w + 2 * h);

        let p_start = self.particles.len();
        let s_start = self.springs.len();

        for i in 0..w {
            for j in 0..h {
                self.particles.push(Particle::new(
                    i as f64 * Particle::SPACING + x,
                    j as f64 * Particle::SPACING + y,
                ));

                let ind = self.particles.len() - 1;
                if i < w - 1 {
                    self.springs
                        .push(Spring::new(ind, ind + h, Particle::SPACING));
                }
                if j < h - 1 {
                    self.springs
                        .push(Spring::new(ind, ind + 1, Particle::SPACING));
                }
                if i < w - 1 && j < h - 1 {
                    self.springs
                        .push(Spring::new(ind, ind + h + 1, Particle::DIAG_SQR.sqrt()));
                }
                if i > 0 && j < h - 1 {
                    self.springs
                        .push(Spring::new(ind, ind - h + 1, Particle::DIAG_SQR.sqrt()));
                }
            }
        }

        let b_start = self.boundaries.len();

        for n in 0..w {
            self.boundaries.push(p_start + n * h);
        }
        for n in (w - 1) * h + 1..w * h {
            self.boundaries.push(p_start + n);
        }
        for n in (1..w - 1).rev() {
            self.boundaries.push(p_start + (n + 1) * h - 1);
        }
        for n in (1..h).rev() {
            self.boundaries.push(p_start + n);
        }

        self.objects.push(ObjectDescriptor::new(
            p_start,
            self.particles.len(),
            s_start,
            self.springs.len(),
            b_start,
            self.boundaries.len(),
        ));

        Ok(())
    }

    pub fn update(&mut self) -> Result<(), f64> {
        while self.dt_acc >= Self::DT {
            for (i, particle) in self.particles.iter().enumerate() {
                let x = ((particle.pos.x / WIDTH) * Self::GRID) as usize;
                let y = ((particle.pos.y / HEIGHT) * Self::GRID) as usize;

                self.buckets[(x + y * Self::GRID as usize)
                    .clamp(0, (Self::GRID * Self::GRID) as usize - 1)]
                .push(i);
            }

            for spring in &self.springs {
                Self::update_spring(spring, &mut self.particles)?;
            }

            for i in 0..self.particles.len() {
                let mut particle = self.particles[i].clone();

                //TODO: CLEAR THIS SHIT UP
                let (x, y) = Self::grid_pos(&particle);

                let mut collide_bucket = |z: usize| {
                    for j in &self.buckets[z] {
                        if i != *j {
                            particle.collide(&mut self.particles[*j]);
                        }
                    }
                };

                collide_bucket(Self::grid_idx(x, y));

                if y > 0 {
                    collide_bucket(Self::grid_idx(x, y - 1));
                }

                if y > 0 && x > 0 {
                    collide_bucket(Self::grid_idx(x - 1, y - 1));
                }

                if x > 0 {
                    collide_bucket(Self::grid_idx(x - 1, y));
                }

                if x > 0 && y < Self::GRID as usize {
                    collide_bucket(Self::grid_idx(x - 1, y + 1));
                }

                //Gravity
                particle.acc += Self::GRAVITY;

                particle.integrate(Self::DT);

                self.particles[i] = particle;
            }

            for i in &self.boundaries {
                for edge in &self.edges {
                    edge.collide(&mut self.particles[*i]);
                }
            }

            self.buckets.iter_mut().for_each(Vec::clear);

            self.dt_acc -= Self::DT;
        }

        Ok(())
    }

    pub fn end_frame(&mut self, dt: f64) {
        self.dt_acc += dt;
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.springs.clear();
        self.boundaries.clear();
        self.objects.clear();
    }

    pub fn info(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.particles.len(),
            self.springs.len(),
            self.boundaries.len(),
            self.edges.len(),
            self.objects.len(),
        )
    }

    pub fn draw_particles(&self, canvas: &mut impl Renderer) {
        canvas.set_color(Color::YELLOW);
        for particle in &self.particles {
            canvas.filled_circle(particle.pos, Particle::R);
        }
    }

    pub fn draw_springs(&self, canvas: &mut impl Renderer) {
        canvas.set_color(Color::CYAN);
        for spring in &self.springs {
            canvas.line(self.particles[spring.a].pos, self.particles[spring.b].pos);
        }
    }

    pub fn draw_polys(&self, canvas: &mut impl Renderer) {
        const COLORS: [Color; 7] = [
            Color::RED,
            Color::YELLOW,
            Color::BLUE,
            Color::MAGENTA,
            Color::CYAN,
            Color::GREEN,
            Color::WHITE,
        ];

        for (obj, &color) in self.objects.iter().zip(COLORS.iter().cycle()) {
            let vertices = obj
                .boundaries_range()
                .map(|i| self.particles[self.boundaries[i]].pos);

            canvas.set_color(color).polygon(vertices);
        }
    }

    pub fn draw_edges(&self, canvas: &mut impl Renderer) {
        for edge in &self.edges {
            canvas
                .set_color(Color::RGB(44, 56, 80))
                .thick_line(edge.start, edge.get_end(), Edge::R * 2.0)
                .set_color(Color::RGB(88, 112, 161))
                .filled_circle(edge.start, Edge::R)
                .filled_circle(edge.get_end(), Edge::R);
        }
    }

    pub fn remove_last(&mut self) {
        if let Some(obj) = self.objects.pop() {
            self.particles.truncate(obj.particle_start);
            self.springs.truncate(obj.spring_start);
            self.boundaries.truncate(obj.boundary_start);
        }
    }

    pub fn edges_iter_mut(&mut self) -> impl Iterator<Item = &'_ mut Edge> {
        self.edges.iter_mut()
    }

    pub fn edges_iter(&mut self) -> impl Iterator<Item = &'_ Edge> {
        self.edges.iter()
    }

    pub fn remove_edge(&mut self, n: usize) {
        self.edges.remove(n);
    }

    fn grid_pos(particle: &Particle) -> (usize, usize) {
        let x = ((particle.pos.x / WIDTH) * Self::GRID) as usize;
        let y = ((particle.pos.y / HEIGHT) * Self::GRID) as usize;

        (x, y)
    }

    fn grid_idx(x: usize, y: usize) -> usize {
        (x + y * Self::GRID as usize).clamp(0, SQR!(Self::GRID) as usize - 1)
    }

    fn update_spring(spring: &Spring, particles: &mut [Particle]) -> Result<(), f64> {
        let p1 = &particles[spring.a];
        let p2 = &particles[spring.b];

        let diff = p2.pos - p1.pos;
        let diff_len = diff.len();

        /*NOTE: If the current length of the spring
        is greater than - lets say - five times the initial length
        we have probably detected an instabil explosion. We need to report this because
        these explosions can bog down the application and make it unresponsive.*/
        if diff_len > spring.l0 * 5.0 {
            return Err(diff_len);
        }

        let diff_norm = diff / diff_len;

        let dl = diff_len - spring.l0;

        let dist_factor = if dl.is_sign_positive() { dl } else { 1.0 };

        let fs = dist_factor * dl * Spring::KS;
        let fd = diff_norm.dot(p2.vel - p1.vel) * Spring::KD;

        let f = (fs + fd) * diff_norm;

        particles[spring.a].acc += f;
        particles[spring.b].acc -= f;

        Ok(())
    }
}
