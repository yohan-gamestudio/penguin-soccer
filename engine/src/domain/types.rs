use std::ops::{Add, Sub, Mul, Neg};

pub type PlayerId = u8;
pub type TeamId = u8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn normalized(&self) -> Self {
        let len = self.length();
        if len < 1e-10 {
            Vec2::ZERO
        } else {
            Vec2::new(self.x / len, self.y / len)
        }
    }

    pub fn dot(&self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Vec2::new(-self.x, -self.y)
    }
}
