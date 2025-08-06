use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub struct Player {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}
impl Player {
    fn update(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub struct Projectile {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}
impl Projectile {
    fn update(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub struct World {
    players: Vec<Player>,
    projectiles: Vec<Projectile>,
}

impl World {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            projectiles: Vec::new(),
        }
    }
    pub fn update(&mut self, dt: f32) {
        for i in 0..self.players.len() {
            self.players[i].update(dt);
        }
        for i in 0..self.projectiles.len() {
            self.projectiles[i].update(dt);
        }
    }
}
