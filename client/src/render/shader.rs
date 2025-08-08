use miniquad::*;

pub const VERTEX: &str = r#"
#version 100
precision mediump float;

attribute vec2 in_pos;
uniform float time;

void main() {
    gl_Position = vec4(in_pos, 0.0, 1.0);
    gl_PointSize = 400.0; // Size in screen pixels
}
"#;

pub const FRAGMENT: &str = r#"#version 100
precision mediump float;

void main() {
    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0); // White dot
}
"#;

pub const METAL: &str = r#""#;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("time", UniformType::Float1),
            ],
        },
    }
}

#[repr(C)]
pub struct Uniforms {
    pub time: f32,
}
