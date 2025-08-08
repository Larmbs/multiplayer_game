//! This module defines the world environment, an array of immovable objects that are more reliable and have less of a need of being passed around over the network.
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub struct Environment {
    pub objects: Vec<Object>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Decode, Encode)]
pub struct Object {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
