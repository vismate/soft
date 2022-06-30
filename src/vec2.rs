use auto_ops::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn from_angle(angle: f64) -> Self {
        Self {
            x: angle.cos(),
            y: angle.sin(),
        }
    }

    pub fn from_angle_deg(angle: f64) -> Self {
        Self::from_angle(angle.to_radians())
    }

    pub const fn null() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn len(self) -> f64 {
        self.len_sqr().sqrt()
    }

    pub fn len_sqr(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn dist(self, other: Self) -> f64 {
        self.dist_sqr(other).sqrt()
    }

    pub fn dist_sqr(self, other: Self) -> f64 {
        (self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y)
    }

    pub fn angle(self, other: Self) -> f64 {
        f64::atan2(other.y, other.x) - f64::atan2(self.y, self.x)
    }

    pub fn angle_deg(self, other: Self) -> f64 {
        self.angle(other).to_degrees()
    }

    pub fn normalize(self) -> Self {
        let len = self.len();

        if len > 0.0 {
            self * len.recip()
        } else {
            Self::null()
        }
    }

    pub fn lerp(self, other: Self, factor: f64) -> Self {
        Self {
            x: self.x + factor * (other.x - self.x),
            y: self.y + factor * (other.y - self.y),
        }
    }

    pub fn reflect(self, normal: Self) -> Self {
        let dot = self.dot(normal);

        Self {
            x: self.x - (2.0 * normal.x) * dot,
            y: self.y - (2.0 * normal.y) * dot,
        }
    }

    pub fn rotate(self, angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();

        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    pub fn rotate_deg(self, angle: f64) -> Self {
        self.rotate(angle.to_radians())
    }

    pub fn recip(self) -> Self {
        Self {
            x: self.x.recip(),
            y: self.x.recip(),
        }
    }

    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            x: f64::min(max.x, f64::max(min.x, self.x)),
            y: f64::min(max.y, f64::max(min.y, self.y)),
        }
    }

    pub fn clamp_len(self, min: f64, max: f64) -> Self {
        let len_sqr = self.len_sqr();

        if len_sqr <= 0.0 {
            return Self::null();
        }

        let len = len_sqr.sqrt();

        if len < min {
            self * (min / len)
        } else if len > max {
            self * (max / len)
        } else {
            Self::null()
        }
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Self::null()
    }
}

// Addition
impl_op!(+ |a: Vec2, b: Vec2| -> Vec2 {Vec2 {x: a.x + b.x, y: a.y + b.y}});
impl_op!(+ |a: &Vec2, b: &Vec2| -> Vec2 {Vec2 {x: a.x + b.x, y: a.y + b.y}});
impl_op_commutative!(+ |a: &Vec2, b: Vec2| -> Vec2 {Vec2 {x: a.x + b.x, y: a.y + b.y}});

impl_op!(+= |a: &mut Vec2, b: Vec2| {a.x += b.x; a.y += b.y;});
impl_op!(+= |a: &mut Vec2, b: &Vec2| {a.x += b.x; a.y += b.y;});

impl_op!(+ |a: Vec2, b: f64| -> Vec2 {Vec2 {x: a.x + b, y: a.y + b}});
impl_op!(+ |a: &Vec2, b: f64| -> Vec2 {Vec2 {x: a.x + b, y: a.y + b}});
impl_op!(+= |a: &mut Vec2, b: f64| {a.x += b; a.y += b;});

// Subtraction
impl_op!(- |a: Vec2, b: Vec2| -> Vec2 {Vec2 {x: a.x - b.x, y: a.y - b.y}});
impl_op!(- |a: &Vec2, b: &Vec2| -> Vec2 {Vec2 {x: a.x - b.x, y: a.y - b.y}});
impl_op_commutative!(- |a: &Vec2, b: Vec2| -> Vec2 {Vec2 {x: a.x - b.x, y: a.y - b.y}});

impl_op!(-= |a: &mut Vec2, b: Vec2| {a.x -= b.x; a.y -= b.y;});
impl_op!(-= |a: &mut Vec2, b: &Vec2| {a.x -= b.x; a.y -= b.y;});

impl_op!(- |a: Vec2, b: f64| -> Vec2 {Vec2 {x: a.x - b, y: a.y - b}});
impl_op!(- |a: &Vec2, b: f64| -> Vec2 {Vec2 {x: a.x - b, y: a.y - b}});
impl_op!(-= |a: &mut Vec2, b: f64| {a.x -= b; a.y -= b;});


// Multiplication
impl_op!(* |a: Vec2, b: Vec2| -> Vec2 {Vec2 {x: a.x * b.x, y: a.y * b.y}});
impl_op!(* |a: &Vec2, b: &Vec2| -> Vec2 {Vec2 {x: a.x * b.x, y: a.y * b.y}});
impl_op_commutative!(* |a: &Vec2, b: Vec2| -> Vec2 {Vec2 {x: a.x * b.x, y: a.y * b.y}});

impl_op!(*= |a: &mut Vec2, b: Vec2| {a.x *= b.x; a.y *= b.y;});
impl_op!(*= |a: &mut Vec2, b: &Vec2| {a.x *= b.x; a.y *= b.y;});

impl_op_commutative!(* |a: Vec2, b: f64| -> Vec2 {Vec2 {x: a.x * b, y: a.y * b}});
impl_op_commutative!(* |a: &Vec2, b: f64| -> Vec2 {Vec2 {x: a.x * b, y: a.y * b}});
impl_op!(*= |a: &mut Vec2, b: f64| {a.x *= b; a.y *= b;});


// Division
impl_op!(/ |a: Vec2, b: Vec2| -> Vec2 {Vec2 {x: a.x / b.x, y: a.y / b.y}});
impl_op!(/ |a: &Vec2, b: &Vec2| -> Vec2 {Vec2 {x: a.x / b.x, y: a.y / b.y}});
impl_op_commutative!(/ |a: &Vec2, b: Vec2| -> Vec2 {Vec2 {x: a.x / b.x, y: a.y / b.y}});

impl_op!(/= |a: &mut Vec2, b: Vec2| {a.x /= b.x; a.y /= b.y;});
impl_op!(/= |a: &mut Vec2, b: &Vec2| {a.x /= b.x; a.y /= b.y;});

impl_op!(/ |a: Vec2, b: f64| -> Vec2 {Vec2 {x: a.x / b, y: a.y / b}});
impl_op!(/ |a: &Vec2, b: f64| -> Vec2 {Vec2 {x: a.x / b, y: a.y / b}});
impl_op!(/= |a: &mut Vec2, b: f64| {a.x /= b; a.y /= b;});

// Misc
impl_op!(- |a: Vec2| -> Vec2 {Vec2{x: -a.x, y: -a.y}});
impl_op!(- |a: &Vec2| -> Vec2 {Vec2{x: -a.x, y: -a.y}});