use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Represents a 2D point
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Decode, Encode)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
}
impl Vec2 {
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        Self {
            x: rng.random_range(0.0..1.0),
            y: rng.random_range(0.0..1.0),
        }
    }
}
impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}
impl Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self::Output {
        if scalar != 0.0 {
            Self {
                x: self.x / scalar,
                y: self.y / scalar,
            }
        } else {
            panic!("Division by zero in Vec2");
        }
    }
}
impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}
impl SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}
impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
    }
}
impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, scalar: f32) {
        if scalar != 0.0 {
            self.x /= scalar;
            self.y /= scalar;
        } else {
            panic!("Division by zero in Vec2");
        }
    }
}
