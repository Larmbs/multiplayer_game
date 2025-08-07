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
    players: HashMap<u64, Player>,
    projectiles: Vec<Projectile>,
}

impl World {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            projectiles: Vec::new(),
        }
    }
    pub fn update(&mut self, dt: f32) {
        for (_, player) in &mut self.players {
            player.update(dt);
        }
        for i in 0..self.projectiles.len() {
            self.projectiles[i].update(dt);
        }
    }
}

impl World {
    pub fn get_all_players(&self) -> &HashMap<u64, Player> {
        &self.players
    }
    pub fn update_player(&mut self, id: u64, player: Player) {
        if let Some(existing) = self.players.get_mut(&id) {
            *existing = player;
        } else {
            self.players.insert(id, player);
        }
    }

    pub fn remove_player(&mut self, player_id: u64) {
        self.players.remove(&player_id);
    }

    pub fn set_players(&mut self, players: HashMap<u64, Player>) {
        self.players = players;
    }

    pub fn set_projectiles(&mut self, projectiles: Vec<Projectile>) {
        self.projectiles = projectiles;
    }

    pub fn create_projectile(&self) -> Projectile {
        Projectile {
            x: 0.0, // Fill in with logic for where projectile should spawn
            y: 0.0,
            vx: 0.0,
            vy: -1.0,
        }
    }
}
