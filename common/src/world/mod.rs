//! This module defines the world structure and its components, including players and the environment.
//! It provides the [`World`] struct, which contains the game state, including entities and their
//! properties. The world can be updated with player movements and other game logic.
use std::collections::HashMap;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub mod entities;
pub mod environment;

use entities::Entities;
use environment::Environment;

/// The main game world that contains the environment and entities (players).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub struct World {
    pub environment: Environment,
    pub entities: Entities,
}
impl World {
    pub fn new() -> Self {
        Self {
            environment: Environment {
                objects: Vec::new(),
            },
            entities: Entities {
                players: HashMap::new(),
            },
        }
    }
}
