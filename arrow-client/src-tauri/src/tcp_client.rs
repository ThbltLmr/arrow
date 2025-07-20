use crate::postures::Posture;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

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
pub struct PostureUpdate {
    pub posture: Posture,
    pub message: String,
    pub metrics: Option<PostureMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub message: String,
}

pub struct TcpClient {
    app_handle: AppHandle,
    connection_status: Arc<Mutex<bool>>,
}

impl TcpClient {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            connection_status: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(&self) {
        let app_handle = self.app_handle.clone();
        let connection_status = self.connection_status.clone();

        tokio::spawn(async move {
            loop {
                match TcpStream::connect("127.0.0.1:9876").await {
                    Ok(stream) => {
                        {
                            let mut status = connection_status.lock().await;
                            *status = true;
                        }

                        let _ = app_handle.emit(
                            "connection-status",
                            ConnectionStatus {
                                connected: true,
                                message: "Connected to posture server".to_string(),
                            },
                        );

                        if let Err(e) = Self::handle_connection(stream, &app_handle).await {
                            eprintln!("Connection error: {}", e);
                        }

                        {
                            let mut status = connection_status.lock().await;
                            *status = false;
                        }

                        let _ = app_handle.emit(
                            "connection-status",
                            ConnectionStatus {
                                connected: false,
                                message: "Disconnected from server. Retrying...".to_string(),
                            },
                        );
                    }
                    Err(e) => {
                        let _ = app_handle.emit(
                            "connection-status",
                            ConnectionStatus {
                                connected: false,
                                message: format!("Connection failed: {}. Retrying...", e),
                            },
                        );
                    }
                }

                // Wait before retrying
                sleep(Duration::from_secs(3)).await;
            }
        });
    }

    async fn handle_connection(
        stream: TcpStream,
        app_handle: &AppHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF - server closed connection
                    break;
                }
                Ok(_) => {
                    // Remove trailing newline
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
                    }

                    if let Some(posture_update) = Self::parse_metrics(&line) {
                        let _ = app_handle.emit("posture-update", posture_update);
                    }
                }
                Err(e) => {
                    return Err(Box::new(e));
                }
            }
        }

        Ok(())
    }

    fn parse_metrics(metrics_str: &str) -> Option<PostureUpdate> {
        let parts: Vec<&str> = metrics_str.split('|').collect();
        if parts.len() == 16 {
            let metrics = PostureMetrics {
                left_ear: Point3D {
                    x: parts[0].parse::<f32>().unwrap_or(0.0),
                    y: parts[1].parse::<f32>().unwrap_or(0.0),
                    z: parts[2].parse::<f32>().unwrap_or(0.0),
                    visibility: parts[3].parse::<f32>().unwrap_or(0.0),
                },
                right_ear: Point3D {
                    x: parts[4].parse::<f32>().unwrap_or(0.0),
                    y: parts[5].parse::<f32>().unwrap_or(0.0),
                    z: parts[6].parse::<f32>().unwrap_or(0.0),
                    visibility: parts[7].parse::<f32>().unwrap_or(0.0),
                },
                left_shoulder: Point3D {
                    x: parts[8].parse::<f32>().unwrap_or(0.0),
                    y: parts[9].parse::<f32>().unwrap_or(0.0),
                    z: parts[10].parse::<f32>().unwrap_or(0.0),
                    visibility: parts[11].parse::<f32>().unwrap_or(0.0),
                },
                right_shoulder: Point3D {
                    x: parts[12].parse::<f32>().unwrap_or(0.0),
                    y: parts[13].parse::<f32>().unwrap_or(0.0),
                    z: parts[14].parse::<f32>().unwrap_or(0.0),
                    visibility: parts[15].parse::<f32>().unwrap_or(0.0),
                },
            };

            let posture = Self::determine_posture(&metrics);
            let message = posture.get_posture_message();

            Some(PostureUpdate {
                posture,
                message,
                metrics: Some(metrics),
            })
        } else {
            None
        }
    }

    fn determine_posture(metrics: &PostureMetrics) -> Posture {
        let PostureMetrics {
            left_ear,
            right_ear,
            left_shoulder,
            right_shoulder,
        } = metrics;

        // Check visibility
        if left_shoulder.visibility < 0.9 || right_shoulder.visibility < 0.9 {
            return Posture::ShouldersNotVisible;
        }

        if left_ear.visibility < 0.9 || right_ear.visibility < 0.9 {
            return Posture::HeadNotVisible;
        }

        // Calculate avg depths
        let avg_ear_depth = (left_ear.z + right_ear.z) / 2.0;
        let avg_shoulder_depth = (left_shoulder.z + right_shoulder.z) / 2.0;

        // Check slouching
        if avg_ear_depth + 0.2 < avg_shoulder_depth && avg_shoulder_depth > -0.33 {
            return Posture::SlouchingBack;
        }
        if avg_ear_depth + 0.33 < avg_shoulder_depth {
            return Posture::LeaningIn;
        }

        // Calculate ear slope for head tilt
        let ear_slope = (left_ear.y - right_ear.y) / (left_ear.x - right_ear.x);
        if ear_slope > 0.10 {
            return Posture::HeadTiltRight;
        }
        if ear_slope < -0.10 {
            return Posture::HeadTiltLeft;
        }

        // Calculate shoulder slope for body tilt
        let shoulder_slope = (left_shoulder.y - right_shoulder.y) / (left_shoulder.x - right_shoulder.x);
        if shoulder_slope > 0.10 {
            return Posture::BodyTiltRight;
        }
        if shoulder_slope < -0.10 {
            return Posture::BodyTiltLeft;
        }

        // Default to STRAIGHT
        Posture::Straight
    }

    pub async fn is_connected(&self) -> bool {
        *self.connection_status.lock().await
    }
}