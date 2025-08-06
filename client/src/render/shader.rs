use miniquad::*;

pub const VERTEX: &str = r#"
#version 100
attribute vec2 in_pos;
attribute vec2 in_uv;
varying highp vec2 uv;

void main() {
    gl_Position = vec4(in_pos, 0, 1);
    uv = in_uv;
}
"#;

pub const FRAGMENT: &str = r#"#version 100
precision highp float;

varying vec2 uv;

uniform float time;

uniform int player_count;
uniform vec2 players[16];
uniform vec2 player_size;

uniform int bullet_count;
uniform vec2 bullets[32];
uniform vec2 bullet_size;

bool in_rect(vec2 p, vec2 center, vec2 half_size) {
    return abs(p.x - center.x) <= half_size.x &&
           abs(p.y - center.y) <= half_size.y;
}

void main() {
    vec3 color = vec3(0.0);

    vec2 half_player = player_size * 0.5;
    vec2 half_bullet = bullet_size * 0.5;

    for (int i = 0; i < 16; i++) {
        if (i >= player_count) break;
        if (in_rect(uv, players[i], half_player)) {
            color = vec3(0.0, 0.5, 1.0); // Blue player
        }
    }

    for (int i = 0; i < 32; i++) {
        if (i >= bullet_count) break;
        if (in_rect(uv, bullets[i], half_bullet)) {
            color = vec3(1.0, 0.0, 0.0); // Red bullet
        }
    }

    gl_FragColor = vec4(color, 1.0);
}
"#;

pub const METAL: &str = r#"
#include <metal_stdlib>
using namespace metal;

struct Uniforms {
    float time;

    short player_count;
    float2 players[16];
    float2 player_size;

    short bullet_count;
    float2 bullets[32];
    float2 bullet_size;
};

struct Vertex {
    float2 in_pos   [[attribute(0)]];
    float2 in_uv    [[attribute(1)]];
};

struct RasterizerData {
    float4 position [[position]];
    float2 uv       [[user(locn0)]];
};

vertex RasterizerData vertexShader(Vertex v [[stage_in]]) {
    RasterizerData out;
    out.position = float4(v.in_pos.xy, 0.0, 1.0);
    out.uv = v.in_uv;
    return out;
}

bool in_rect(float2 p, float2 center, float2 half_size) {
    return fabs(p.x - center.x) <= half_size.x &&
           fabs(p.y - center.y) <= half_size.y;
}

fragment float4 fragmentShader(RasterizerData in [[stage_in]],
                               constant Uniforms& u [[buffer(0)]]) {
    float3 color = float3(0.0);

    float2 half_player = u.player_size * 0.5;
    float2 half_bullet = u.bullet_size * 0.5;

    for (int i = 0; i < 16; i++) {
        if (i >= u.player_count) break;
        if (in_rect(in.uv, u.players[i], half_player)) {
            color = float3(0.0, 0.5, 1.0);
        }
    }

    for (int i = 0; i < 32; i++) {
        if (i >= u.bullet_count) break;
        if (in_rect(in.uv, u.bullets[i], half_bullet)) {
            color = float3(1.0, 0.0, 0.0);
        }
    }

    return float4(color, 1.0);
}
"#;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![UniformDesc::new("time", UniformType::Float1)],
        },
    }
}

#[repr(C)]
pub struct Uniforms {
    pub time: f32,

    pub player_count: i16,
    pub players: [(f32, f32); 3],
    pub player_size: (f32, f32),

    pub bullet_count: i16,
    pub bullets: [(f32, f32); 3],
    pub bullet_size: (f32, f32),
}
