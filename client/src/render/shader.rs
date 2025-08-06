use miniquad::*;

pub const VERTEX: &str = r#"#version 100
    attribute vec2 in_pos;
    attribute vec2 in_uv;

    varying highp vec2 uv;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
        uv = in_uv;
    }"#;

pub const FRAGMENT: &str = r#"#version 100
    precision highp float;

    varying vec2 uv;

    uniform float time;
    uniform int blobs_count;
    uniform vec2 blobs_positions[32];

    float k = 20.0;
    float field = 0.0;
    vec2 coord;
        
    void circle ( float r , vec3 col , vec2 offset) {
        vec2 pos = coord.xy;
        vec2 c = offset;
        float d = distance ( pos , c );
        field += ( k * r ) / ( d*d );
    }
        
    vec3 band ( float shade, float low, float high, vec3 col1, vec3 col2 ) {
        if ( (shade >= low) && (shade <= high) ) {
            float delta = (shade - low) / (high - low);
            vec3 colDiff = col2 - col1;
            return col1 + (delta * colDiff);
        }
        else
            return vec3(0.0,0.0,0.0);
    }
    
    vec3 gradient ( float shade ) {
        vec3 colour = vec3( (sin(time/2.0)*0.25)+0.25,0.0,(cos(time/2.0)*0.25)+0.25);
        
        vec3 col1 = vec3(0.01, 0.0, 1.0-0.01);
        vec3 col2 = vec3(1.0-0.01, 0.0, 0.01);
        vec3 col3 = vec3(0.02, 1.0-0.02, 0.02);
        vec3 col4 = vec3((0.01+0.02)/2.0, (0.01+0.02)/2.0, 1.0 - (0.01+0.02)/2.0);
        vec3 col5 = vec3(0.02, 0.02, 0.02);
        
        colour += band ( shade, 0.0, 0.3, colour, col1 );
        colour += band ( shade, 0.3, 0.6, col1, col2 );
        colour += band ( shade, 0.6, 0.8, col2, col3 );
        colour += band ( shade, 0.8, 0.9, col3, col4 );
        colour += band ( shade, 0.9, 1.0, col4, col5 );
        
        return colour;
    }
    
    void main() {
        coord = uv;
        
        for (int i = 0; i < 32; i++) {
            if (i >= blobs_count) { break; } // workaround for webgl error: Loop index cannot be compared with non-constant expression
            circle(.03 , vec3(0.7 ,0.2, 0.8), blobs_positions[i]);
        }
        
        float shade = min ( 1.0, max ( field/256.0, 0.0 ) );
        
        gl_FragColor = vec4( gradient(shade), 1.0 );
    }"#;

pub const METAL: &str = r#"
    #include <metal_stdlib>

    using namespace metal;

    struct Uniforms
    {
        float time;
        int16_t blobs_count;
        float2 blobs[32];
    };

    struct Vertex
    {
        float2 in_pos   [[attribute(0)]];
        float2 in_uv    [[attribute(1)]];
    };

    struct RasterizerData
    {
        float4 position [[position]];
        float2 uv       [[user(locn0)]];
    };

    vertex RasterizerData vertexShader(
      Vertex v [[stage_in]])
    {
        RasterizerData out;

        out.position = float4(v.in_pos.xy, 0.0, 1.0);
        out.uv = v.in_uv;

        return out;
    }

    constant float k = 20.0;
        
    float circle(float2 coord, float r , float3 col , float2 offset) {
        float2 pos = coord.xy;
        float2 c = offset;
        float d = distance ( pos , c );
        return ( k * r ) / ( d*d );
    }
        
    float3 band ( float shade, float low, float high, float3 col1, float3 col2 ) {
        if ( (shade >= low) && (shade <= high) ) {
            float delta = (shade - low) / (high - low);
            float3 colDiff = col2 - col1;
            return col1 + (delta * colDiff);
        }
        else
            return float3(0.0,0.0,0.0);
    }
    
    float3 gradient (float shade, float time) {
        float3 colour = float3( (sin(time/2.0)*0.25)+0.25,0.0,(cos(time/2.0)*0.25)+0.25);
        
        float3 col1 = float3(0.01, 0.0, 1.0-0.01);
        float3 col2 = float3(1.0-0.01, 0.0, 0.01);
        float3 col3 = float3(0.02, 1.0-0.02, 0.02);
        float3 col4 = float3((0.01+0.02)/2.0, (0.01+0.02)/2.0, 1.0 - (0.01+0.02)/2.0);
        float3 col5 = float3(0.02, 0.02, 0.02);
        
        colour += band ( shade, 0.0, 0.3, colour, col1 );
        colour += band ( shade, 0.3, 0.6, col1, col2 );
        colour += band ( shade, 0.6, 0.8, col2, col3 );
        colour += band ( shade, 0.8, 0.9, col3, col4 );
        colour += band ( shade, 0.9, 1.0, col4, col5 );
        
        return colour;
    }

    fragment float4 fragmentShader(RasterizerData in [[stage_in]], constant Uniforms& uniforms [[buffer(0)]])
    {
        float field = 0.0;
        for (int i = 0; i < 32; i++) {
            if (i >= uniforms.blobs_count) { break; } // workaround for webgl error: Loop index cannot be compared with non-constant expression
            field += circle(in.uv, .03 , float3(0.7 ,0.2, 0.8), uniforms.blobs[i]);
        }
        
        float shade = min ( 1.0, max(field/256.0, 0.0 ) );
        
        return float4(gradient(shade, uniforms.time), 1.0 );
    }"#;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("time", UniformType::Float1),
                UniformDesc::new("blobs_count", UniformType::Int1),
                UniformDesc::new("blobs_positions", UniformType::Float2).array(32),
            ],
        },
    }
}

#[repr(C)]
pub struct Uniforms {
    pub time: f32,
    pub blobs_count: i32,
    pub blobs_positions: [(f32, f32); 32],
}
