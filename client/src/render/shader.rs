use miniquad::*;

pub const VERTEX: &str = r#"
#version 100
precision mediump float;

attribute vec2 in_pos;
attribute vec3 in_color;

uniform float time;
uniform vec2 offset;

varying vec3 color;

void main() {
    gl_Position = vec4(in_pos - offset, 0.0, 1.0);
    gl_PointSize = 400.0; // Size in screen pixels
    color = in_color;
}
"#;

pub const FRAGMENT: &str = r#"#version 100
precision mediump float;

varying vec3 color;

void main() {
    gl_FragColor = vec4(color, 1.0);
}
"#;

pub const METAL: &str = r#""#;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("time", UniformType::Float1),
                UniformDesc::new("offset", UniformType::Float2),
            ],
        },
    }
}

#[repr(C)]
pub struct Uniforms {
    pub time: f32,
    pub offset: (f32, f32),
}
