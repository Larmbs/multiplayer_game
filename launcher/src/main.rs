//! This binary is part of the multiplayer game project.
//! It defines the main entry point for the game launcher, which provides a user interface for
//! launching the game server and client, as well as options for single-player and multiplayer modes.
use anyhow::Result;
use common::details;
use common::version::Version;
use eframe::egui::{self, Align, CentralPanel, Context, Layout, RichText};
use local_ip_address::local_ip;
use std::{path::PathBuf, process::Stdio};
use tokio::process::{Child, Command};

/// Current launchers version number
const LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION_SERVERS: [&str; 1] =
    ["https://drive.google.com/drive/folders/1IezcI1yue--XeAM4vQUGaJLL8iwJaxH-?usp=drive_link"];
#[derive(Default)]
enum LauncherState {
    #[default]
    Ready,
    Failed,
    DownloadGame,
    DownloadingUpdate,
}

struct LauncherApp {
    version: Version,
    state: LauncherState,

    addr_input: String,

    root_path: PathBuf,
    version_file: PathBuf,
    game_zip: PathBuf,
    game_exe: PathBuf,

    server_process: Option<Child>,
    client_process: Option<Child>,
}
impl LauncherApp {
    fn new() -> Result<Self> {
        // Current file path
        let root_path = std::env::current_dir().unwrap();
        let version_file = root_path.join("version.txt");
        let game_zip = root_path.join("game.zip");
        let game_exe = root_path.join("target/release/client");

        Ok(LauncherApp {
            version: Version::try_from(LAUNCHER_VERSION)?,
            state: LauncherState::Ready,
            addr_input: String::new(),
            root_path,
            version_file,
            game_zip,
            game_exe,
            server_process: None,
            client_process: None,
        })
    }
    fn check_for_updates(&mut self) -> Result<(), String> {
        todo!()
    }
    fn install_game_files(&mut self, is_update: bool) -> Result<(), String> {
        todo!()
    }
}
impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                ui.add_space(20.0);
                ui.label(
                    RichText::new(&format!("{} Launcher", details::GAME_NAME))
                        .heading()
                        .size(28.0),
                );

                ui.add_space(20.0);

                match self.state {
                    LauncherState::MainMenu => {
                        ui.label("Choose your game mode:");

                        ui.add_space(10.0);
                        if ui
                            .add(
                                egui::Button::new("🎮 Single Player")
                                    .min_size([200.0, 40.0].into()),
                            )
                            .clicked()
                        {
                            let addr = &format!("127.0.0.1:{}", details::DEFAULT_PORT);
                            self.server_process = launch_server(addr);
                            self.client_process = launch_client(addr);
                            self.state = LauncherState::Launching;
                        }

                        ui.add_space(5.0);
                        if ui
                            .add(egui::Button::new("🌐 Multiplayer").min_size([200.0, 40.0].into()))
                            .clicked()
                        {
                            self.state = LauncherState::MultiplayerMenu;
                        }
                    }

                    LauncherState::MultiplayerMenu => {
                        ui.label(RichText::new("Multiplayer Options").strong().size(20.0));
                        ui.add_space(10.0);

                        if let Ok(local_addr) = local_ip() {
                            ui.label(format!("Your Local IP: {}", local_addr));
                        }

                        ui.add_space(10.0);
                        if ui
                            .add(egui::Button::new("🛠 Host Game").min_size([200.0, 40.0].into()))
                            .clicked()
                        {
                            let addr = format!("{}:{}", local_ip().unwrap(), details::DEFAULT_PORT);
                            self.server_process = launch_server(&addr);
                            self.client_process = launch_client(&addr);
                            self.state = LauncherState::Launching;
                        }

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label("Join Address:");
                            ui.text_edit_singleline(&mut self.addr_input);
                        });

                        ui.add_space(5.0);
                        if ui
                            .add(egui::Button::new("🔗 Join Game").min_size([200.0, 40.0].into()))
                            .clicked()
                        {
                            self.client_process = launch_client(&self.addr_input);
                            self.state = LauncherState::Launching;
                        }

                        ui.add_space(20.0);
                        if ui.button("⬅ Back").clicked() {
                            self.state = LauncherState::MainMenu;
                        }
                    }

                    LauncherState::Launching => {
                        ui.label(RichText::new("🚀 Game is launching...").size(20.0));
                        ui.add_space(10.0);
                        ui.label("You can close this launcher or stop the server/client below.");

                        ui.add_space(20.0);
                        if ui
                            .add(egui::Button::new("⛔ Stop Game").min_size([150.0, 40.0].into()))
                            .clicked()
                        {
                            stop_processes(self);
                            self.state = LauncherState::MainMenu;
                        }
                    }
                }
            });
        });
    }
}

impl Drop for LauncherApp {
    fn drop(&mut self) {
        stop_processes(self);
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        &format!("{} Launcher", details::GAME_NAME),
        options,
        Box::new(|_cc| Ok(Box::new(LauncherApp::new()?))),
    )
}

fn stop_processes(app: &mut LauncherApp) {
    if let Some(child) = &mut app.client_process {
        let _ = child.start_kill();
    }
    if let Some(child) = &mut app.server_process {
        let _ = child.start_kill();
    }
    app.client_process = None;
    app.server_process = None;
}

fn launch_server(addr: &str) -> Option<Child> {
    Command::new("target/release/server")
        .args([addr])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}

fn launch_client(addr: &str) -> Option<Child> {
    Command::new("target/release/client")
        .args([addr])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()
}
