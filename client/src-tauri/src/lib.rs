mod db_manager;
mod postures;

use std::sync::Mutex;
use db_manager::{DbManager, PostureLog};
use postures::Posture;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostureMetrics {
    pub left_ear: Point3D,
    pub right_ear: Point3D,
    pub left_shoulder: Point3D,
    pub right_shoulder: Point3D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub visibility: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostureUpdate {
    pub posture: String,
    pub message: String,
    pub previous_posture: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub message: String,
}

pub struct AppState {
    pub db_manager: Mutex<Option<DbManager>>,
    pub current_posture: Mutex<Posture>,
}

#[tauri::command]
fn get_session_logs(state: State<AppState>) -> Result<Option<Vec<PostureLog>>, String> {
    let db_manager = state.db_manager.lock().unwrap();
    match db_manager.as_ref() {
        Some(manager) => manager.get_session_logs().map_err(|e| e.to_string()),
        None => Ok(None),
    }
}

#[tauri::command]
fn get_current_posture(state: State<AppState>) -> String {
    let posture = state.current_posture.lock().unwrap();
    posture.get_posture_message()
}

fn determine_posture(metrics: &PostureMetrics) -> Posture {
    let PostureMetrics {
        left_ear,
        right_ear,
        left_shoulder,
        right_shoulder,
    } = metrics;

    if left_shoulder.visibility < 0.9 || right_shoulder.visibility < 0.9 {
        return Posture::ShouldersNotVisible;
    }

    if left_ear.visibility < 0.9 || right_ear.visibility < 0.9 {
        return Posture::HeadNotVisible;
    }

    let avg_ear_depth = (left_ear.z + right_ear.z) / 2.0;
    let avg_shoulder_depth = (left_shoulder.z + right_shoulder.z) / 2.0;

    if avg_ear_depth + 0.2 < avg_shoulder_depth && avg_shoulder_depth > -0.33 {
        return Posture::SlouchingBack;
    }
    if avg_ear_depth + 0.33 < avg_shoulder_depth {
        return Posture::LeaningIn;
    }

    let ear_slope = (left_ear.y - right_ear.y) / (left_ear.x - right_ear.x);
    if ear_slope > 0.10 {
        return Posture::HeadTiltRight;
    }
    if ear_slope < -0.10 {
        return Posture::HeadTiltLeft;
    }

    let shoulder_slope = (left_shoulder.y - right_shoulder.y) / (left_shoulder.x - right_shoulder.x);
    if shoulder_slope > 0.10 {
        return Posture::BodyTiltRight;
    }
    if shoulder_slope < -0.10 {
        return Posture::BodyTiltLeft;
    }

    Posture::Straight
}

#[tauri::command]
fn process_posture_metrics(
    metrics_str: String,
    state: State<AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let parts: Vec<&str> = metrics_str.split('|').collect();
    if parts.len() != 16 {
        return Err("Invalid metrics format".to_string());
    }

    let metrics = PostureMetrics {
        left_ear: Point3D {
            x: parts[0].parse().map_err(|_| "Invalid left_ear.x")?,
            y: parts[1].parse().map_err(|_| "Invalid left_ear.y")?,
            z: parts[2].parse().map_err(|_| "Invalid left_ear.z")?,
            visibility: parts[3].parse().map_err(|_| "Invalid left_ear.visibility")?,
        },
        right_ear: Point3D {
            x: parts[4].parse().map_err(|_| "Invalid right_ear.x")?,
            y: parts[5].parse().map_err(|_| "Invalid right_ear.y")?,
            z: parts[6].parse().map_err(|_| "Invalid right_ear.z")?,
            visibility: parts[7].parse().map_err(|_| "Invalid right_ear.visibility")?,
        },
        left_shoulder: Point3D {
            x: parts[8].parse().map_err(|_| "Invalid left_shoulder.x")?,
            y: parts[9].parse().map_err(|_| "Invalid left_shoulder.y")?,
            z: parts[10].parse().map_err(|_| "Invalid left_shoulder.z")?,
            visibility: parts[11].parse().map_err(|_| "Invalid left_shoulder.visibility")?,
        },
        right_shoulder: Point3D {
            x: parts[12].parse().map_err(|_| "Invalid right_shoulder.x")?,
            y: parts[13].parse().map_err(|_| "Invalid right_shoulder.y")?,
            z: parts[14].parse().map_err(|_| "Invalid right_shoulder.z")?,
            visibility: parts[15].parse().map_err(|_| "Invalid right_shoulder.visibility")?,
        },
    };

    let new_posture = determine_posture(&metrics);
    let previous_posture = {
        let mut current_posture = state.current_posture.lock().unwrap();
        let previous = current_posture.clone();
        *current_posture = new_posture.clone();
        previous
    };

    if new_posture.get_posture_value() != previous_posture.get_posture_value() {
        if let Some(manager) = state.db_manager.lock().unwrap().as_ref() {
            let _ = manager.log_posture_change(
                &new_posture.get_posture_value(),
                &previous_posture.get_posture_value(),
            );
        }

        let update = PostureUpdate {
            posture: new_posture.get_posture_value(),
            message: new_posture.get_posture_message(),
            previous_posture: previous_posture.get_posture_value(),
        };

        let _ = app_handle.emit("posture-changed", &update);
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_manager = match DbManager::new() {
        Ok(manager) => {
            if let Err(e) = manager.log_session_start() {
                eprintln!("Failed to log session start: {}", e);
            }
            Some(manager)
        }
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            None
        }
    };

    let app_state = AppState {
        db_manager: Mutex::new(db_manager),
        current_posture: Mutex::new(Posture::Unknown),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_session_logs,
            get_current_posture,
            process_posture_metrics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}