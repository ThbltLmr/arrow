use crate::db_manager::DbManager;
use crate::events::{
    ConnectionStatus, NotificationEvent, PostureMetrics, PostureUpdate, SessionLogsUpdate,
};
use crate::notification_service::NotificationService;
use crate::postures::Posture;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

pub struct TcpClient {
    app_handle: AppHandle,
    connection_status: Arc<Mutex<bool>>,
    db_manager: Arc<Mutex<Option<DbManager>>>,
    notification_service: Arc<NotificationService>,
    current_posture: Arc<Mutex<Posture>>,
}

impl TcpClient {
    pub fn new(app_handle: AppHandle, db_manager: Arc<Mutex<Option<DbManager>>>) -> Self {
        Self {
            app_handle,
            connection_status: Arc::new(Mutex::new(false)),
            db_manager,
            notification_service: Arc::new(NotificationService::new()),
            current_posture: Arc::new(Mutex::new(Posture::Unknown)),
        }
    }

    pub async fn initialize_notifications(&self) -> Result<(), String> {
        self.notification_service.initialize().await
    }

    pub async fn start(&self) {
        let app_handle = self.app_handle.clone();
        let connection_status = self.connection_status.clone();
        let db_manager = self.db_manager.clone();
        let notification_service = self.notification_service.clone();
        let current_posture = self.current_posture.clone();

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

                        if let Err(e) = Self::handle_connection(
                            stream,
                            &app_handle,
                            &db_manager,
                            &notification_service,
                            &current_posture,
                        )
                        .await
                        {
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
        db_manager: &Arc<Mutex<Option<DbManager>>>,
        notification_service: &Arc<NotificationService>,
        current_posture: &Arc<Mutex<Posture>>,
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
                        // Check for posture change and handle logging/notifications
                        let previous_posture = {
                            let mut current = current_posture.lock().await;
                            let previous = current.clone();
                            *current = posture_update.posture.clone();
                            previous
                        };

                        let posture_changed = posture_update.posture.get_posture_value()
                            != previous_posture.get_posture_value();

                        if posture_changed {
                            // Log posture change to database
                            if let Some(db) = db_manager.lock().await.as_ref() {
                                let _ = db.log_posture_change(
                                    &posture_update.posture.get_posture_value(),
                                    &previous_posture.get_posture_value(),
                                );
                            }

                            // Send notification
                            let is_good_posture =
                                posture_update.posture.get_posture_value() == "STRAIGHT";
                            notification_service
                                .notify_posture_change(&posture_update.posture, is_good_posture)
                                .await;

                            // Emit session logs update event
                            if let Some(db) = db_manager.lock().await.as_ref() {
                                if let Ok(Some(logs)) = db.get_session_logs() {
                                    let _ = app_handle
                                        .emit("session-logs-updated", SessionLogsUpdate { logs });
                                }
                            }

                            // Emit notification event
                            let _ = app_handle.emit(
                                "notification-triggered",
                                NotificationEvent {
                                    posture: posture_update.posture.get_posture_value(),
                                    message: posture_update.posture.get_posture_message(),
                                    is_good_posture,
                                },
                            );
                        }

                        // Always emit posture update
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
                left_ear: crate::events::Point3D {
                    x: parts[0].parse::<f32>().unwrap_or(0.0),
                    y: parts[1].parse::<f32>().unwrap_or(0.0),
                    z: parts[2].parse::<f32>().unwrap_or(0.0),
                    visibility: parts[3].parse::<f32>().unwrap_or(0.0),
                },
                right_ear: crate::events::Point3D {
                    x: parts[4].parse::<f32>().unwrap_or(0.0),
                    y: parts[5].parse::<f32>().unwrap_or(0.0),
                    z: parts[6].parse::<f32>().unwrap_or(0.0),
                    visibility: parts[7].parse::<f32>().unwrap_or(0.0),
                },
                left_shoulder: crate::events::Point3D {
                    x: parts[8].parse::<f32>().unwrap_or(0.0),
                    y: parts[9].parse::<f32>().unwrap_or(0.0),
                    z: parts[10].parse::<f32>().unwrap_or(0.0),
                    visibility: parts[11].parse::<f32>().unwrap_or(0.0),
                },
                right_shoulder: crate::events::Point3D {
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
        let shoulder_slope =
            (left_shoulder.y - right_shoulder.y) / (left_shoulder.x - right_shoulder.x);
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

