//! This file is part of the multiplayer game project.
//! It defines the shapes used in the game rendering, including vertices and meshes for triangles and quads.
//! The shapes are used to represent players and other entities in the game world.
use std::f32::consts::PI;
use common::{color::Color, vec::Vec2};

/// Represents a vertex in the game world with position and color.
/// This is used for rendering shapes like triangles and quads.
#[repr(C)]
#[derive(Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
}
impl Vertex {
    pub fn new(pos: Vec2, color: Color) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }
}

/// A trait for shapes that can be converted into a mesh of vertices.
/// This allows different shapes to be rendered in the game world.
/// Implementations of this trait should provide a method to convert the shape into a vector of vertices
pub trait Mesh {
    fn mesh_vertices(self) -> Vec<Vertex>;
}

/// Represents a triangle in the game world.
/// It consists of three vertices and a color.
/// The triangle can be used to represent players or other entities in the game.
#[derive(Clone)]
pub struct Tri {
    v1: Vec2,
    v2: Vec2,
    v3: Vec2,
    color: Color,
}
impl Tri {
    pub fn new(v1: Vec2, v2: Vec2, v3: Vec2, color: Color) -> Self {
        Self { v1, v2, v3, color }
    }
    pub fn point(center: Vec2, size: f32, color: Color) -> Self {
        // 120 degrees between each vertex (in radians)
        let angle_offset = -PI / 2.0;

        let v1 = Vec2 {
            x: center.x + size * (angle_offset + 0.0).cos(),
            y: center.y + size * (angle_offset + 0.0).sin(),
        };

        let v2 = Vec2 {
            x: center.x + size * (angle_offset + 2.0 * PI / 3.0).cos(),
            y: center.y + size * (angle_offset + 2.0 * PI / 3.0).sin(),
        };

        let v3 = Vec2 {
            x: center.x + size * (angle_offset + 4.0 * PI / 3.0).cos(),
            y: center.y + size * (angle_offset + 4.0 * PI / 3.0).sin(),
        };

        Self::new(v1, v2, v3, color)
    }
}
impl Mesh for Tri {
    fn mesh_vertices(self) -> Vec<Vertex> {
        vec![
            Vertex::new(self.v1, self.color.clone()),
            Vertex::new(self.v2, self.color.clone()),
            Vertex::new(self.v3, self.color),
        ]
    }
}

/// Represents a quad in the game world.
/// It consists of four vertices and a color.
/// The quad can be used for rendering larger areas or backgrounds.
#[derive(Clone)]
pub struct Quad {
    pos: Vec2,
    size: Vec2,
    color: Color,
}
impl Mesh for Quad {
    fn mesh_vertices(self) -> Vec<Vertex> {
        vec![
            Vertex::new(
                Vec2 {
                    x: self.pos.x,
                    y: self.pos.y,
                },
                self.color.clone(),
            ),
            Vertex::new(
                Vec2 {
                    x: self.pos.x + self.size.x,
                    y: self.pos.y,
                },
                self.color.clone(),
            ),
            Vertex::new(
                Vec2 {
                    x: self.pos.x,
                    y: self.pos.y + self.size.y,
                },
                self.color.clone(),
            ),
            Vertex::new(
                Vec2 {
                    x: self.pos.x + self.size.y,
                    y: self.pos.y,
                },
                self.color.clone(),
            ),
            Vertex::new(
                Vec2 {
                    x: self.pos.x + self.size.x,
                    y: self.pos.y,
                },
                self.color.clone(),
            ),
            Vertex::new(
                Vec2 {
                    x: self.pos.x + self.size.x,
                    y: self.pos.y + self.size.y,
                },
                self.color.clone(),
            ),
        ]
    }
}
