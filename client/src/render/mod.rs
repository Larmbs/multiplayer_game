use common::world::World;
use miniquad::*;

use crate::{
    camera::Camera,
    render::{
        shader::Uniforms,
        shapes::{Mesh, Tri},
    },
};
mod shader;
mod shapes;

pub struct Render {
    ctx: Box<dyn RenderingBackend>,
    pipeline: Pipeline,
    bindings: Bindings,
    uniforms: Uniforms,
    start_time: f64,

    player_buffer: BufferId,
}
impl Render {
    pub fn init() -> Self {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        let player_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Dynamic,
            BufferSource::empty::<shapes::Vertex>(5 * 3),
        );

        let indices: Vec<u16> = (0..25 as u16).collect();

        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let bindings = Bindings {
            vertex_buffers: vec![player_buffer],
            index_buffer, // or remove entirely
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
            offset: (0.0, 0.0),
        };

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_color", VertexFormat::Float3),
            ],
            shader,
            PipelineParams {
                primitive_type: PrimitiveType::Triangles,
                ..PipelineParams::default()
            },
        );

        let start_time = miniquad::date::now();

        Self {
            ctx,
            pipeline,
            bindings,
            uniforms,
            start_time,
            player_buffer,
        }
    }
    pub fn draw(&mut self, camera: &Camera, world: &World) {
        self.uniforms.time = (miniquad::date::now() - self.start_time) as f32;
        self.uniforms.offset = (camera.pos.x, camera.pos.y);

        let mut triangle_vertices = Vec::new();

        for (_, player) in world.entities.players.iter() {
            triangle_vertices.append(
                &mut Tri::point(player.pos, 0.05, player.color.clone())
                    .mesh_vertices(),
            );
        }

        // Update the player buffer with all triangle vertices
        self.ctx
            .buffer_update(self.player_buffer, BufferSource::slice(&triangle_vertices));

        self.ctx
            .begin_default_pass(PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        self.ctx
            .apply_uniforms(UniformsSource::table(&self.uniforms));
        self.ctx.draw(0, triangle_vertices.len() as i32, 1);
        self.ctx.end_render_pass();
        self.ctx.commit_frame();
    }
}
