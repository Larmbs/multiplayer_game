//! This module defines entities, a movable object in this world
use std::collections::HashMap;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub struct Player {
    pub username: String,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}
impl Player {
    fn update(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
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