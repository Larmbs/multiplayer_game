//! This module defines entities, a movable object in this world
use std::collections::HashMap;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use crate::{color::Color, vec::Vec2};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub struct Player {
    pub username: String,
    pub color: Color,
    pub pos: Vec2,
    pub vel: Vec2,
}
impl Player {
    fn update(&mut self, dt: f32) {
        self.pos.x += self.vel.x * dt;
        self.pos.y += self.vel.y * dt;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub struct Entities {
    pub players: HashMap<u64, Player>,
}
impl Entities {
    pub fn update(&mut self, dt: f32) {
        for (_, player) in &mut self.players {
            player.update(dt);
        }
    }
}