use crate::postures::Posture;
use notify_rust::{Notification, NotificationHandle};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;

pub struct NotificationService {
    current_handle: Arc<Mutex<Option<NotificationHandle>>>,
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            current_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn initialize(&self) -> Result<(), String> {
        // Create initial notification to test permissions
        match Notification::new()
            .summary("Arrow Posture Monitor")
            .body("Posture monitoring started")
            .timeout(Duration::from_secs(2))
            .show()
        {
            Ok(handle) => {
                let mut current_handle = self.current_handle.lock().await;
                *current_handle = Some(handle);
                Ok(())
            }
            Err(e) => Err(format!("Failed to initialize notifications: {}", e)),
        }
    }

    pub async fn notify_posture_change(&self, current_posture: &Posture, is_good_posture: bool) {
        let mut handle_guard = self.current_handle.lock().await;

        if is_good_posture {
            // Good posture notification
            match Notification::new()
                .summary("Well done!")
                .body("Back to sitting straight, good job!")
                .timeout(Duration::from_secs(3))
                .show()
            {
                Ok(handle) => {
                    if let Some(old_handle) = handle_guard.take() {
                        old_handle.close();
                    }
                    *handle_guard = Some(handle);
                }
                Err(e) => {
                    eprintln!("Failed to show good posture notification: {}", e);
                }
            }
        } else {
            // Bad posture notification - persistent until corrected
            match Notification::new()
                .summary("Bad posture!")
                .body(&format!(
                    "You should correct your posture. Current posture detected: {}",
                    current_posture.get_posture_value()
                ))
                .timeout(Duration::from_secs(0)) // Persistent notification
                .show()
            {
                Ok(handle) => {
                    if let Some(old_handle) = handle_guard.take() {
                        old_handle.close();
                    }
                    *handle_guard = Some(handle);
                }
                Err(e) => {
                    eprintln!("Failed to show bad posture notification: {}", e);
                }
            }
        }
    }

    pub async fn close_notification(&self) {
        let mut handle_guard = self.current_handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.close();
        }
    }
}

impl Drop for NotificationService {
    fn drop(&mut self) {
        // Note: We can't properly clean up notifications in Drop because:
        // 1. We can't await in Drop
        // 2. NotificationHandle::close() takes ownership
        // The OS will clean up notifications when the process exits
    }
}