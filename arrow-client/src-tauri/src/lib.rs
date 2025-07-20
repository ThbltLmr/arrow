mod db_manager;
mod postures;
mod tcp_client;

use db_manager::{DbManager, PostureLog};
use postures::Posture;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tcp_client::{ConnectionStatus, PostureUpdate, TcpClient};
use tokio::sync::Mutex;

pub struct AppState {
    pub db_manager: Arc<Mutex<Option<DbManager>>>,
    pub tcp_client: Arc<Mutex<Option<TcpClient>>>,
    pub current_posture: Arc<Mutex<Posture>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db_manager: Arc::new(Mutex::new(None)),
            tcp_client: Arc::new(Mutex::new(None)),
            current_posture: Arc::new(Mutex::new(Posture::Unknown)),
        }
    }
}

#[tauri::command]
async fn initialize_app(app_handle: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    // Initialize database
    let db_manager = match DbManager::new() {
        Ok(manager) => {
            if let Err(e) = manager.log_session_start() {
                eprintln!("Failed to log session start: {}", e);
            }
            manager
        }
        Err(e) => {
            return Err(format!("Failed to initialize database: {}", e));
        }
    };

    {
        let mut db_lock = state.db_manager.lock().await;
        *db_lock = Some(db_manager);
    }

    // Initialize TCP client
    let tcp_client = TcpClient::new(app_handle.clone());
    tcp_client.start().await;

    {
        let mut tcp_lock = state.tcp_client.lock().await;
        *tcp_lock = Some(tcp_client);
    }

    Ok("Application initialized successfully".to_string())
}

#[tauri::command]
async fn get_session_logs(state: State<'_, AppState>) -> Result<Option<Vec<PostureLog>>, String> {
    let db_lock = state.db_manager.lock().await;
    if let Some(db_manager) = db_lock.as_ref() {
        match db_manager.get_session_logs() {
            Ok(logs) => Ok(logs),
            Err(e) => Err(format!("Failed to get session logs: {}", e)),
        }
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
async fn get_connection_status(state: State<'_, AppState>) -> Result<ConnectionStatus, String> {
    let tcp_lock = state.tcp_client.lock().await;
    if let Some(tcp_client) = tcp_lock.as_ref() {
        let connected = tcp_client.is_connected().await;
        Ok(ConnectionStatus {
            connected,
            message: if connected {
                "Connected to posture server".to_string()
            } else {
                "Not connected to server".to_string()
            },
        })
    } else {
        Ok(ConnectionStatus {
            connected: false,
            message: "TCP client not initialized".to_string(),
        })
    }
}

#[tauri::command]
async fn log_posture_change(
    current_posture: String,
    previous_posture: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db_lock = state.db_manager.lock().await;
    if let Some(db_manager) = db_lock.as_ref() {
        match db_manager.log_posture_change(&current_posture, &previous_posture) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Failed to log posture change: {}", e)),
        }
    } else {
        Err("Database not initialized".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            initialize_app,
            get_session_logs,
            get_connection_status,
            log_posture_change
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
