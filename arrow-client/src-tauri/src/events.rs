use crate::db_manager::PostureLog;
use crate::postures::Posture;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostureUpdate {
    pub posture: Posture,
    pub message: String,
    pub metrics: Option<PostureMetrics>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLogsUpdate {
    pub logs: Vec<PostureLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEvent {
    pub posture: String,
    pub message: String,
    pub is_good_posture: bool,
}