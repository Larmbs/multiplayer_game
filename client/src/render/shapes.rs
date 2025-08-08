use std::f32::consts::PI;

#[repr(C)]
#[derive(Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
}

pub trait Mesh {
    fn mesh_vertices(self) -> Vec<Vertex>;
}

#[derive(Clone)]
pub struct Tri {
    v1: Vertex,
    v2: Vertex,
    v3: Vertex,
}
impl Tri {
    pub fn new(v1: Vertex, v2: Vertex, v3: Vertex) -> Self {
        Self { v1, v2, v3 }
    }
    pub fn point(center: Vertex, size: f32) -> Self {
        // 120 degrees between each vertex (in radians)
        let angle_offset = -PI / 2.0;

        let v1 = Vertex {
            x: center.x + size * (angle_offset + 0.0).cos(),
            y: center.y + size * (angle_offset + 0.0).sin(),
        };

        let v2 = Vertex {
            x: center.x + size * (angle_offset + 2.0 * PI / 3.0).cos(),
            y: center.y + size * (angle_offset + 2.0 * PI / 3.0).sin(),
        };

        let v3 = Vertex {
            x: center.x + size * (angle_offset + 4.0 * PI / 3.0).cos(),
            y: center.y + size * (angle_offset + 4.0 * PI / 3.0).sin(),
        };

        Self::new(v1, v2, v3)
    }
}
impl Mesh for Tri {
    fn mesh_vertices(self) -> Vec<Vertex> {
        vec![self.v1, self.v2, self.v3]
    }
}

#[derive(Clone)]
pub struct Quad {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}
impl Mesh for Quad {
    fn mesh_vertices(self) -> Vec<Vertex> {
        vec![
            Vertex {
                x: self.x,
                y: self.y,
            },
            Vertex {
                x: self.x + self.w,
                y: self.y,
            },
            Vertex {
                x: self.x,
                y: self.y + self.h,
            },
            Vertex {
                x: self.x,
                y: self.y + self.h,
            },
            Vertex {
                x: self.x + self.w,
                y: self.y,
            },
            Vertex {
                x: self.x + self.w,
                y: self.y + self.h,
            },
        ]
    }
}
