//! This binary is part of the multiplayer game project.
//! It defines the main entry point for the game client, which connects to the server,
//! handles user input, and renders the game world. The client uses Tokio for asynchronous
//! networking and Miniquad for rendering. It supports player movement, updates the game state,
//! and communicates with the server to synchronize the game world.
use anyhow::Result;
use clap::Parser;

use common::vec::Vec2;
use miniquad::{conf::Conf, *};

use common::message::{ClientMessage, ServerMessage};
use common::world::{World, entities::Player};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};

mod camera;
mod cli;
mod client;
mod render;

use camera::Camera;
use cli::Cli;
use client::Client;
use render::Render;

/// GameRuntime manages the game loop, rendering, and client-server communication.
pub struct GameRuntime {
    /* Inbox of messages from the server */
    /// Async context
    _runtime: tokio::runtime::Runtime,
    server_rx: UnboundedReceiver<ServerMessage>,
    server_tx: UnboundedSender<ClientMessage>,

    /// World data
    world: World,

    /* Rendering related */
    render: Render,
    camera: Camera,

    last_frame: f32,
    time_accumulator: f32,

    player_id: u64,
    username: String,
}
impl GameRuntime {
    pub fn init(runtime: tokio::runtime::Runtime, cli: Cli) -> Result<Self> {
        let (runtime_tx, server_rx) = unbounded_channel();
        let (server_tx, runtime_rx) = unbounded_channel();

        let handle = runtime.handle().clone();

        let (id, mut client) = runtime.block_on(Client::connect(
            cli.address,
            cli.username.clone(),
            cli.password.unwrap_or_default(),
            runtime_tx,
            runtime_rx,
        ))?;

        // Spawn the network listener inside the given runtime
        handle.spawn(async move {
            // Create a client
            let _ = client.listen().await;
        });

        let world = World::new();
        let render = Render::init();
        let time = miniquad::date::now() as f32;

        Ok(Self {
            _runtime: runtime,
            server_rx,
            server_tx,
            world,
            render,
            last_frame: time,
            time_accumulator: 0.0,
            player_id: id,
            username: cli.username,
            camera: Camera { pos: Vec2::ZERO },
        })
    }
}
impl EventHandler for GameRuntime {
    fn update(&mut self) {
        const FIXED_TIMESTEP: f32 = 1.0 / 60.0;
        let time = miniquad::date::now() as f32;
        let dt = (time - self.last_frame) as f32;
        self.last_frame = time;

        self.time_accumulator += dt;

        while self.time_accumulator >= FIXED_TIMESTEP {
            self.world.entities.update(dt);

            self.time_accumulator -= FIXED_TIMESTEP;
        }

        if let Some(self_player) = self.world.entities.players.get(&self.player_id) {
            self.camera.pos = self_player.pos;
        }

        // Receive world updates from server
        while let Ok(msg) = self.server_rx.try_recv() {
            match msg {
                ServerMessage::UpdateEntities(players) => {
                    self.world.entities = players; // You'll need to implement this
                }
                _ => {}
            }
        }
    }

    fn draw(&mut self) {
        self.render.draw(&self.camera, &self.world);
    }
    fn key_up_event(&mut self, _keycode: KeyCode, _keymods: KeyMods) {
        let self_player = self.world.entities.players.get(&self.player_id).unwrap();
        let player = Player {
            color: self_player.color,
            pos: self_player.pos,
            vel: Vec2::ZERO,
            username: self.username.clone(),
        };

        let _ = self
            .server_tx
            .send(ClientMessage::NotifyUpdatePlayer(player));
    }
    fn key_down_event(&mut self, keycode: KeyCode, _mods: KeyMods, _repeat: bool) {
        // Simulate movement based on key input
        let mut vx = 0.0;
        let mut vy = 0.0;

        match keycode {
            KeyCode::W => vy = 1.0,
            KeyCode::S => vy = -1.0,
            KeyCode::A => vx = -1.0,
            KeyCode::D => vx = 1.0,
            _ => return,
        }

        let self_player = self.world.entities.players.get(&self.player_id).unwrap();
        let player = Player {
            color: self_player.color,
            pos: self_player.pos,
            vel: Vec2 { x: vx, y: vy },
            username: self.username.clone(),
        };

        let _ = self
            .server_tx
            .send(ClientMessage::NotifyUpdatePlayer(player));
    }
}

fn main() {
    let cli = Cli::parse();

    let mut conf = Conf {
        window_title: "My Game".to_string(),
        window_width: 800,
        window_height: 600,
        window_resizable: true, // Enable window resizing
        ..Default::default()
    };

    let metal = cli.metal;
    conf.platform.apple_gfx_api = if metal {
        panic!("Client does not support Mac");
    } else {
        conf::AppleGfxApi::OpenGl
    };

    let runtime = Runtime::new().unwrap();

    miniquad::start(conf, move || {
        Box::new(GameRuntime::init(runtime, cli).unwrap())
    });
}
