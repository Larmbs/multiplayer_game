use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub struct Player {
    pub id: u64,
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
pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
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

impl World {
    pub fn get_all_players(&self) -> &Vec<Player> {
        &self.players
    }
    pub fn update_player(&mut self, player: Player) {
        if let Some(existing) = self.players.iter_mut().find(|p| p.id == player.id) {
            *existing = player;
        } else {
            self.players.push(player);
        }
    }

    pub fn remove_player(&mut self, player_id: u64) {
        self.players.retain(|p| p.id != player_id);
    }
}