use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::sleep;

use crate::{ConnectionStatus, AppState};

pub async fn start_tcp_service(app_handle: AppHandle) {
    tokio::spawn(async move {
        loop {
            match connect_and_listen(&app_handle).await {
                Ok(_) => {
                    println!("TCP connection ended normally");
                }
                Err(e) => {
                    eprintln!("TCP connection error: {}", e);
                    let status = ConnectionStatus {
                        connected: false,
                        message: format!("Connection failed: {}. Retrying...", e),
                    };
                    let _ = app_handle.emit("connection-status", &status);
                }
            }
            
            // Wait before retrying
            sleep(Duration::from_secs(2)).await;
        }
    });
}

async fn connect_and_listen(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Attempting to connect to server...");
    
    let stream = TcpStream::connect("127.0.0.1:9876").await?;
    println!("Connected to server!");
    
    let status = ConnectionStatus {
        connected: true,
        message: "Connected. Waiting for data...".to_string(),
    };
    let _ = app_handle.emit("connection-status", &status);
    
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF reached (server closed connection)
                println!("Server closed connection");
                let status = ConnectionStatus {
                    connected: false,
                    message: "Disconnected. Retrying...".to_string(),
                };
                let _ = app_handle.emit("connection-status", &status);
                return Ok(());
            }
            Ok(_) => {
                // Successfully read a line
                let data = line.trim().to_string();
                
                // Process the posture metrics
                if let Some(state) = app_handle.try_state::<AppState>() {
                    if let Err(e) = crate::process_posture_metrics(data, state, app_handle.clone()) {
                        eprintln!("Error processing posture metrics: {}", e);
                    }
                } else {
                    eprintln!("Failed to get app state");
                }
            }
            Err(e) => {
                eprintln!("Error reading from TCP stream: {}", e);
                let status = ConnectionStatus {
                    connected: false,
                    message: "Connection lost. Retrying...".to_string(),
                };
                let _ = app_handle.emit("connection-status", &status);
                return Err(e.into());
            }
        }
    }
}