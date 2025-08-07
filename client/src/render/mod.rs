use miniquad::*;

use crate::render::shader::Uniforms;
mod shader;

#[repr(C)]
struct Vec2 {
    x: f32,
    y: f32,
}
#[repr(C)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

pub struct Render {
    ctx: Box<dyn RenderingBackend>,
    pipeline: Pipeline,
    bindings: Bindings,
    uniforms: Uniforms,
    start_time: f64,
}
impl Render {
    pub fn init(ctx: &mut dyn RenderingBackend) -> Self {
        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex { pos : Vec2 { x: -1.0, y: -1.0 }, uv: Vec2 { x: 0., y: 0. } },
            Vertex { pos : Vec2 { x:  1.0, y: -1.0 }, uv: Vec2 { x: 1., y: 0. } },
            Vertex { pos : Vec2 { x:  1.0, y:  1.0 }, uv: Vec2 { x: 1., y: 1. } },
            Vertex { pos : Vec2 { x: -1.0, y:  1.0 }, uv: Vec2 { x: 0., y: 1. } },
        ];
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![],
        };

        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: shader::VERTEX,
                        fragment: shader::FRAGMENT,
                    },
                    Backend::Metal => ShaderSource::Msl {
                        program: shader::METAL,
                    },
                },
                shader::meta(),
            )
            .unwrap();

        let uniforms = shader::Uniforms {
            time: 0.,
            player_count: 3,
            players: [(0.0, 0.0), (10.0, 0.0), (0.0, 5.0)],
            player_size: (30.0, 30.0),
            bullet_count: 3,
            bullets: [(7.0, 0.0), (-10.0, 0.0), (0.0, -5.0)],
            bullet_size: (10.0, 10.0),
        };

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams::default(),
        );

        let start_time = miniquad::date::now();

        let ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        Self {
            ctx,
            pipeline,
            bindings,
            uniforms,
            start_time,
        }
    }
    pub fn draw(&mut self) {
        self.uniforms.time = (miniquad::date::now() - self.start_time) as f32;

        self.ctx
            .begin_default_pass(PassAction::clear_color(0.7, 0.1, 0.1, 1.0));
        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        self.ctx
            .apply_uniforms(UniformsSource::table(&self.uniforms));
        self.ctx.draw(0, 6, 1); // 3 vertices = 1 triangle
        self.ctx.end_render_pass();
        self.ctx.commit_frame();
    }
}
