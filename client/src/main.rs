use anyhow::Result;
use clap::Parser;
use common::message::{ClientMessage, ServerMessage};
use common::world::World;
use miniquad::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

mod cli;
mod client;
mod render;
use cli::Cli;
use client::Client;
use render::Render;

pub struct GameRuntime {
    /// Inbox of messages from the server
    server_rx: UnboundedReceiver<ServerMessage>,
    server_tx: UnboundedSender<ClientMessage>,

    /// World data
    world: World,

    /* Rendering related */
    ctx: Box<dyn RenderingBackend>,
    render: Render,
    last_frame: f64,
}
impl GameRuntime {
    fn init() -> Result<Self> {
        // For communication of world updates from the server to the client
        let (runtime_tx, server_rx) = unbounded_channel();

        // For communication of world updates from the client to the server
        let (server_tx, runtime_rx) = unbounded_channel();

        // The game requires argument to start
        let cli = Cli::parse();

        let client = Client::connect(cli.address, runtime_tx, runtime_rx);
        tokio::spawn(async move {
            match client.await {
                Ok(mut client) => {
                    let _ = client.listen().await;
                }
                Err(e) => {
                    eprintln!("Connection error: {:?}", e);
                }
            }
        });

        let world = World::new();

        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        let render = Render::init(&mut *ctx);

        let time = miniquad::date::now();

        Ok(Self {
            server_rx,
            server_tx,
            world,
            ctx,
            render,
            last_frame: time,
        })
    }
}
impl EventHandler for GameRuntime {
    fn update(&mut self) {
        let time = miniquad::date::now();
        let dt = (time - self.last_frame) as f32;
        self.last_frame = time;

        self.world.update(dt);
    }

    fn draw(&mut self) {
        self.render.draw(&mut *self.ctx);
        self.ctx.commit_frame();
    }
}

fn main() {
    let mut conf = conf::Conf::default();
    let metal = std::env::args().nth(1).as_deref() == Some("metal");
    conf.platform.apple_gfx_api = if metal {
        conf::AppleGfxApi::Metal
    } else {
        conf::AppleGfxApi::OpenGl
    };

    miniquad::start(conf, move || Box::new(GameRuntime::init().unwrap()));
}
