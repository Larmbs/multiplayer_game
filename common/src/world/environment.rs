use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::vec::Vec2;

/// Describes the entire game environment.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub struct Environment {
    pub objects: Vec<Object>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub struct Object {
    pub pos: Vec2,
    pub size: Vec2,
}