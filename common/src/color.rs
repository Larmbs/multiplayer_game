use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Represents a color in RGB format
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Decode, Encode)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}
impl Color {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
    };
}
impl Color {
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        Self {
            r: rng.random_range(0.0..1.0),
            g: rng.random_range(0.0..1.0),
            b: rng.random_range(0.0..1.0),
        }
    }
}
