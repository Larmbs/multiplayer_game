use anyhow::Result;
use clap::Parser;
use common::message::{ClientMessage, ServerMessage};
use common::world::{Player, World};
use miniquad::conf::Conf;
use miniquad::*;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

mod cli;
mod client;
mod render;
use cli::Cli;
use client::Client;
use render::Render;

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

    last_frame: f64,

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
        let time = miniquad::date::now();

        Ok(Self {
            _runtime: runtime,
            server_rx,
            server_tx,
            world,
            render,
            last_frame: time,
            player_id: id,
            username: cli.username,
        })
    }
}
impl EventHandler for GameRuntime {
    fn update(&mut self) {
        let time = miniquad::date::now();
        let dt = (time - self.last_frame) as f32;
        self.last_frame = time;

        self.world.entities.update(dt);

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
        self.render.draw(&self.world);
    }
    fn key_up_event(&mut self, _keycode: KeyCode, _keymods: KeyMods) {
        let self_player = self.world.entities.players.get(&self.player_id).unwrap();
        let player = Player {
            x: self_player.x, // Client doesn't know its true position yet
            y: self_player.y,
            vx: 0.0,
            vy: 0.0,
            username: self.username.clone(),
        };

        let _ = self
            .server_tx
            .send(ClientMessage::NotifyUpdatePlayer(player));
    }
    fn key_down_event(&mut self, keycode: KeyCode, _mods: KeyMods, _repeat: bool) {
        use common::message::ClientMessage;
        use common::world::Player;

        // Simulate movement based on key input
        let mut vx = 0.0;
        let mut vy = 0.0;

        match keycode {
            KeyCode::W => vy = -1.0,
            KeyCode::S => vy = 1.0,
            KeyCode::A => vx = -1.0,
            KeyCode::D => vx = 1.0,
            _ => return,
        }

        let self_player = self.world.entities.players.get(&self.player_id).unwrap();
        let player = Player {
            x: self_player.x, // Client doesn't know its true position yet
            y: self_player.y,
            vx,
            vy,
            username: self.username.clone(),
        };

        let _ = self
            .server_tx
            .send(ClientMessage::NotifyUpdatePlayer(player));
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "My Game".to_string(),
        window_width: 800,
        window_height: 600,
        window_resizable: true, // Enable window resizing
        ..Default::default()
    }
}

fn main() {
    let cli = Cli::parse();

    let mut conf = window_conf();

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
