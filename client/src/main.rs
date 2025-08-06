use macroquad::prelude::*;
use tokio::runtime::Runtime; // Import Tokio Runtime
use std::sync::{Arc, Mutex}; // For shared state if needed

// Assuming your client struct is in a 'client.rs' file
mod client;
use client::Client;

// This function will connect and listen for server messages
async fn connect_and_listen() {
    let addr = "127.0.0.1:8000";
    match Client::connect(addr).await {
        Ok(mut client) => {
            println!("Connected to server at {}", addr);
            if let Err(e) = client.listen().await {
                eprintln!("Client listen error: {:?}", e);
            }
        }
        Err(e) => {
            eprintln!("Client connection error: {:?}", e);
        }
    }
}

#[macroquad::main("Battle Game")]
async fn main() {
    // Create a Tokio runtime
    let rt = Arc::new(Runtime::new().unwrap());

    // Spawn the client logic as a background task on the Tokio runtime.
    // This will not block the main game loop.
    let rt_clone = rt.clone(); // Clone Arc for use in the spawned task
    rt_clone.spawn(connect_and_listen()); // Use the runtime's spawn method

    // Game loop using macroquad
    loop {
        clear_background(BLACK);

        draw_text("Battle Game", 20.0, 40.0, 40.0, WHITE);
        draw_text("Press ESC to quit", 20.0, 80.0, 30.0, GRAY);

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;
    }
}
