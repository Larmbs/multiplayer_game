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
use eframe::egui::{self, Context};
use local_ip_address::local_ip;
use reqwest::Client;
use std::{path::PathBuf, process::Stdio};
use tokio::process::{Child, Command};

/// Different servers that serve game and server binaries
const VERSION_SERVERS: [&str; 1] =
    ["https://raw.githubusercontent.com/Larmbs/multiplayer_game/refs/heads/master/"];

/// Server and Client sources are parallel
struct Source {
    pub binary: &'static str,
    pub zip: &'static str,
    pub version: &'static str,
}

#[derive(Default, Clone)]
enum LauncherState {
    #[default]
    Ready,
    Failed,
    DownloadNeeded,
    DownloadingUpdate,
    CheckingForUpdates,
}

struct LauncherApp {
    state: LauncherState,

    server_process: Option<Child>,
    client_process: Option<Child>,

    http: Client,

    addr_input: String,
    update_available: bool,
}
impl LauncherApp {
    const CLIENT_SRC: Source = Source {
        binary: "build/client/client",
        zip: "build/client/client.zip",
        version: "build/client/version.txt",
    };
    const SERVER_SRC: Source = Source {
        binary: "build/server/server",
        zip: "build/server/server.zip",
        version: "build/server/version.txt",
    };
    const LAUNCHER_SRC: Source = Source {
        binary: "build/launcher/launcher",
        zip: "build/launcher/launcher.zip",
        version: "build/launcher/version.txt",
    };

    async fn new() -> Result<Self> {
        Ok(Self {
            state: LauncherState::Ready,
            addr_input: String::new(),
            server_process: None,
            client_process: None,
            http: Client::new(),
            update_available: false,
        })
    }
    async fn check_for_updates(&mut self) -> Result<Vec<String>> {
        self.state = LauncherState::CheckingForUpdates;
        let mut updates = Vec::new();
        if self.check_for_file_updates(&Self::CLIENT_SRC).await? {
            updates.push(String::from("Client needs an update"));
        }
        if self.check_for_file_updates(&Self::SERVER_SRC).await? {
            updates.push(String::from("Server needs an update"));
        }
        Ok(updates)
    }
    async fn update(&mut self) -> Result<()> {
        self.state = LauncherState::CheckingForUpdates;
        if self.check_for_file_updates(&Self::CLIENT_SRC).await? {
            self.state = LauncherState::DownloadingUpdate;
            self.update_file(&Self::CLIENT_SRC).await?;
        }
        if self.check_for_file_updates(&Self::SERVER_SRC).await? {
            self.state = LauncherState::DownloadingUpdate;
            self.update_file(&Self::SERVER_SRC).await?;
        }
        self.state = LauncherState::Ready;
        Ok(())
    }
    async fn check_for_file_updates(&self, src: &Source) -> Result<bool> {
        let local_version = self.read_local_version(src).await?;
        let remote_version = self.fetch_remote_version(src).await?;
        match (local_version, remote_version) {
            (Some(local), Some(remote)) if remote > local => Ok(true),
            _ => Ok(false),
        }
    }
    async fn update_file(&self, src: &Source) -> Result<()> {
        let local_version = self.read_local_version(src).await?;
        let remote_version = self.fetch_remote_version(src).await?;

        // Update the client version
        if let (Some(local), Some(remote)) = (local_version, remote_version) {
            if remote > local {
                self.download_remote_file(src.zip, src.zip).await?;
                self.download_remote_file(src.version, src.version).await?;
                self.unzip_file(src.zip).await?;
            }
        }
        Ok(())
    }
}
/// File management
impl LauncherApp {
    async fn fetch_remote_version(&self, src: &Source) -> Result<Option<Version>> {
        let url = format!("{}{}", VERSION_SERVERS[0], src.version);
        let text = self.http.get(&url).send().await?.text().await?;
        Ok(Version::try_from(text.trim()).ok())
    }
    async fn read_local_version(&self, src: &Source) -> Result<Option<Version>> {
        let text = tokio::fs::read_to_string(src.version).await?;
        Ok(Version::try_from(text.trim()).ok())
    }
    async fn download_remote_file(
        &self,
        relative_path: &str,
        output_path: &str,
    ) -> Result<PathBuf> {
        let url = format!("{}{}", VERSION_SERVERS[0], relative_path);
        let response = self.http.get(&url).send().await?;
        if response.status().is_success() {
            let bytes = response.bytes().await?;
            let path = PathBuf::from(output_path);
            tokio::fs::write(&path, bytes).await?;
            Ok(path)
        } else {
            Err(anyhow::anyhow!("Failed to download file: {}", url))
        }
    }
    async fn unzip_file(&self, zip_path: &str) -> Result<()> {
        let output = Command::new("unzip")
            .arg(zip_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to unzip file: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}
/// Launching game processes
impl LauncherApp {
    fn launch_client(&mut self, addr: &str) -> Result<()> {
        if addr.is_empty() {
            return Err(anyhow::anyhow!("Address cannot be empty"));
        }
        if let Some(child) = Command::new(Self::CLIENT_SRC.binary)
            .args([addr])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok()
        {
            self.client_process = Some(child);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to launch client"))
        }
    }

    fn launch_server(&mut self, addr: &str) -> Result<()> {
        if addr.is_empty() {
            return Err(anyhow::anyhow!("Address cannot be empty"));
        }
        if let Some(child) = Command::new(Self::SERVER_SRC.binary)
            .args([addr])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok()
        {
            self.client_process = Some(child);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to launch client"))
        }
    }
    fn process_terminate(&mut self) {
        if let Some(child) = &mut self.client_process {
            let _ = child.start_kill();
        }
        if let Some(child) = &mut self.server_process {
            let _ = child.start_kill();
        }
        self.client_process = None;
        self.server_process = None;
    }
}
/// Rendering the UI
impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        use egui::{Align, Button, Layout, RichText, Separator};

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.add_space(10.0);
                ui.label(
                    RichText::new(format!("{} Launcher", details::GAME_NAME))
                        .heading()
                        .strong(),
                );

                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    ui.label("Server Address:");
                    ui.text_edit_singleline(&mut self.addr_input)
                        .on_hover_text("127.0.0.1");
                });

                ui.add_space(10.0);
                ui.add(Separator::default());

                // Launch buttons
                if ui
                    .add(Button::new("🎮 Join").min_size([150.0, 30.0].into()))
                    .clicked()
                {
                    if self.client_process.is_none() || true {
                        if let Err(e) = self.launch_client(&self.addr_input.clone()) {
                            self.state = LauncherState::Failed;
                            eprintln!("{e}");
                        }
                    }
                }
                if ui
                    .add(Button::new("🖥 Host").min_size([150.0, 30.0].into()))
                    .clicked()
                {
                    if self.server_process.is_none() {
                        let ip = &format!("{}:8000", local_ip().unwrap().to_string());
                        if let Err(e) = self.launch_server(ip) {
                            self.state = LauncherState::Failed;
                            eprintln!("{e}");
                        }
                        if let Err(e) = self.launch_client(ip) {
                            self.state = LauncherState::Failed;
                            eprintln!("{e}");
                        }
                    }
                }
                if ui
                    .add(Button::new("👤 Single Player").min_size([150.0, 30.0].into()))
                    .clicked()
                {
                    if self.server_process.is_none() {
                        if let Err(e) = self.launch_server("127.0.0.1:8000") {
                            self.state = LauncherState::Failed;
                            eprintln!("{e}");
                        }
                        if let Err(e) = self.launch_client("127.0.0.1:8000") {
                            self.state = LauncherState::Failed;
                            eprintln!("{e}");
                        }
                    }
                }

                ui.add_space(20.0);
                ui.add(Separator::default());
                ui.add_space(10.0);

                // Check for Updates
                if ui
                    .add(Button::new("🔍 Check for Updates").min_size([180.0, 30.0].into()))
                    .clicked()
                {
                    self.state = LauncherState::CheckingForUpdates;
                    let ctx_clone = ctx.clone();
                    // Clone only the fields needed for the async call
                    let http = self.http.clone();
                    let client_src = Self::CLIENT_SRC;
                    let server_src = Self::SERVER_SRC;

                    tokio::spawn(async move {
                        // Perform the update check asynchronously
                        let mut updates = Vec::new();
                        // Check client
                        if let Ok(local_version) =
                            tokio::fs::read_to_string(client_src.version).await
                        {
                            let url = format!("{}{}", VERSION_SERVERS[0], client_src.version);
                            if let Ok(response) = http.get(&url).send().await {
                                if let Ok(text) = response.text().await {
                                    if let (Ok(local), Ok(remote)) = (
                                        Version::try_from(local_version.trim()),
                                        Version::try_from(text.trim()),
                                    ) {
                                        if remote > local {
                                            updates.push(String::from("Client needs an update"));
                                        }
                                    }
                                }
                            }
                        }
                        // Check server
                        if let Ok(local_version) =
                            tokio::fs::read_to_string(server_src.version).await
                        {
                            let url = format!("{}{}", VERSION_SERVERS[0], server_src.version);
                            if let Ok(response) = http.get(&url).send().await {
                                if let Ok(text) = response.text().await {
                                    if let (Ok(local), Ok(remote)) = (
                                        Version::try_from(local_version.trim()),
                                        Version::try_from(text.trim()),
                                    ) {
                                        if remote > local {
                                            updates.push(String::from("Server needs an update"));
                                        }
                                    }
                                }
                            }
                        }
                        // Print updates found
                        if !updates.is_empty() {
                            println!("Updates found: {:?}", updates);
                        }
                        ctx_clone.request_repaint();
                    });
                    self.update_available = true; // Set flag so Download button appears
                    self.state = LauncherState::Ready;
                }

                // If update found, show Download button
                if self.update_available {
                    if ui
                        .add(Button::new("⬇ Download Updates").min_size([180.0, 30.0].into()))
                        .clicked()
                    {
                        self.state = LauncherState::DownloadingUpdate;
                        let ctx_clone = ctx.clone();
                        // Clone only the fields needed for the async call
                        let mut app_clone = LauncherApp {
                            state: self.state.clone(),
                            server_process: None,
                            client_process: None,
                            http: self.http.clone(),
                            addr_input: self.addr_input.clone(),
                            update_available: self.update_available,
                        };
                        // Spawn the update task
                        tokio::spawn(async move {
                            if let Err(e) = app_clone.update().await {
                                eprintln!("Update failed: {e}");
                                app_clone.state = LauncherState::Failed;
                            } else {
                                app_clone.update_available = false;
                                app_clone.state = LauncherState::Ready;
                            }
                            ctx_clone.request_repaint();
                        });
                    }
                }

                ui.add_space(15.0);

                // Status
                let status_text = match self.state {
                    LauncherState::Ready => "✅ Ready",
                    LauncherState::Failed => "❌ Failed",
                    LauncherState::DownloadNeeded => "📦 Update Available",
                    LauncherState::DownloadingUpdate => "⬇ Downloading Update...",
                    LauncherState::CheckingForUpdates => "🔍 Checking for Updates...",
                };
                ui.label(RichText::new(status_text).strong());
            });
        });
    }
}
impl Drop for LauncherApp {
    fn drop(&mut self) {
        self.process_terminate();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = eframe::NativeOptions::default();
    let launcher = LauncherApp::new().await?;
    if let Err(e) = eframe::run_native(
        &format!("{} Launcher", details::GAME_NAME),
        options,
        Box::new(|_cc| Ok(Box::new(launcher))),
    ) {
        eprintln!("Failed to launch GUI: {e}");
    }
    Ok(())
}
