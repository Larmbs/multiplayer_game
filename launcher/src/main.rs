//! This binary is part of the multiplayer game project.
//! It defines the main entry point for the game launcher, which provides a user interface for
//! launching the game server and client, as well as options for single-player and multiplayer modes.
//!
//! Application structure
//! This defines what the final structure of the application will look like once compiled.
//!
//! ├── build    // Will contain the binary files for the game and server as well as version files for each
//! │   ├── client
//! │   ├── server
//! │   ├── client_version.txt
//! │   └── server_version.txt
//! ├── launcher
//! ├── version.txt
//!

use anyhow::Result;
use common::details;
use common::version::Version;
use eframe::egui::{self, Align, CentralPanel, Context, Layout, RichText};
use local_ip_address::local_ip;
use reqwest::Client;
use std::{path::PathBuf, process::Stdio};
use tokio::process::{Child, Command};

/// Different servers that serve game and server binaries
const VERSION_SERVERS: [&str; 1] =
    ["https://raw.githubusercontent.com/Larmbs/multiplayer_game/refs/heads/master/"];

#[derive(Default)]
enum LauncherState {
    #[default]
    Ready,
    Failed,
    DownloadGame,
    DownloadingUpdate,
}

struct LauncherApp {
    state: LauncherState,

    addr_input: String,

    server_process: Option<Child>,
    client_process: Option<Child>,

    http: Client,
}
impl LauncherApp {
    const CLIENT_EXE: &'static str = "build/client/client";
    const CLIENT_VERSION: &'static str = "build/client/version.txt";

    const SERVER_EXE: &'static str = "build/server/server";
    const SERVER_VERSION: &'static str = "build/server/version.txt";

    const LAUNCHER_VERSION: &'static str = "build/launcher/version.txt";
    
    fn new() -> Result<Self> {
        Ok(LauncherApp {
            state: LauncherState::Ready,
            addr_input: String::new(),
            server_process: None,
            client_process: None,
            http: Client::new(),
        })
    }
    async fn fetch_remote_version(&self, relative_path: &str) -> Result<Version> {
        let url = format!("{}{}", VERSION_SERVERS[0], relative_path);
        let text = self.http.get(&url).send().await?.text().await?;
        Version::try_from(text.trim()).map_err(anyhow::Error::msg)
    }

    async fn read_local_version(&self, relative_path: &str) -> Result<Version> {
        let text = tokio::fs::read_to_string(relative_path).await?;
        Version::try_from(text.trim()).map_err(anyhow::Error::msg)
    }

    async fn check_for_client_updates(&mut self) -> Result<()> {
        let version_url = format!("{}/client/version.txt", VERSION_SERVERS[0]);

        let remote_version = self.fetch_remote_version("client/version.txt").await?;

        if remote_version > Version::from(Self::LAUNCHER_VERSION) {
            self.state = LauncherState::DownloadingUpdate;
            self.install_game_files(true).await?;
            self.version = remote_version;
            fs::write(&self.version_file, remote_version.to_string())
                .await
                .map_err(|e| format!("Failed to update local version file: {}", e))?;
            self.state = LauncherState::Ready;
            Ok(())
        } else {
            Ok(())
        }
    }
    fn install_game_files(&mut self, is_update: bool) -> Result<(), String> {
        todo!()
    }
}
impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        todo!()
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
