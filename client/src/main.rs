mod db_manager;

use std::time::Duration;

use db_manager::{DbManager, EventLog};
use iced::{
    executor,
    widget::{column, svg, Container, Text},
    Alignment::Center,
    Application, Command, Element, Length, Settings, Subscription, Theme,
};
use notify_rust::{Notification, NotificationHandle};
use tokio::io::BufReader;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> iced::Result {
    PostureApp::run(Settings::default())
}

struct PostureMetrics {
    left_ear: Point3D,
    right_ear: Point3D,
    left_shoulder: Point3D,
    right_shoulder: Point3D,
}

struct Point3D {
    x: f32,
    y: f32,
    z: f32,
    visibility: f32,
}

struct PostureApp {
    posture: String,
    raw_metrics: Option<PostureMetrics>,
    notification: Option<NotificationHandle>,
    db_manager: Option<DbManager>,
    last_logs: Option<Vec<EventLog>>,
}

#[derive(Debug, Clone)]
enum Message {
    PostureUpdate(String),
    Connected(Result<(), String>),
    Disconnected,
}

// State for the subscription lifecycle
enum State {
    Disconnected,
    Connected(BufReader<TcpStream>),
}

impl PostureApp {
    fn determine_posture(&self) -> String {
        if let Some(metrics) = &self.raw_metrics {
            // Check visibility
            let PostureMetrics {
                left_ear,
                right_ear,
                left_shoulder,
                right_shoulder,
            } = metrics;

            if left_shoulder.visibility < 0.9 || right_shoulder.visibility < 0.9 {
                return "SHOULDERS_NOT_VISIBLE".to_string();
            }

            if left_ear.visibility < 0.9 || right_ear.visibility < 0.9 {
                return "HEAD_NOT_VISIBLE".to_string();
            }

            // Calculate avg depths
            let avg_ear_depth = (left_ear.z + right_ear.z) / 2.0;
            let avg_shoulder_depth = (left_shoulder.z + right_shoulder.z) / 2.0;

            // Check slouching
            if avg_ear_depth + 0.2 < avg_shoulder_depth && avg_shoulder_depth > -0.33 {
                return "SLOUCHING_BACK".to_string();
            }
            if avg_ear_depth + 0.33 < avg_shoulder_depth {
                return "LEANING_IN".to_string();
            }

            // Calculate ear slope for head tilt
            let ear_slope = (left_ear.y - right_ear.y) / (left_ear.x - right_ear.x);
            if ear_slope > 0.10 {
                return "HEAD_TILT_RIGHT".to_string();
            }
            if ear_slope < -0.10 {
                return "HEAD_TILT_LEFT".to_string();
            }

            // Calculate shoulder slope for body tilt
            let shoulder_slope =
                (left_shoulder.y - right_shoulder.y) / (left_shoulder.x - right_shoulder.x);
            if shoulder_slope > 0.10 {
                return "BODY_TILT_RIGHT".to_string();
            }
            if shoulder_slope < -0.10 {
                return "BODY_TILT_LEFT".to_string();
            }

            // Default to STRAIGHT
            return "STRAIGHT".to_string();
        }

        // If no metrics available
        "UNKNOWN".to_string()
    }
}

impl Application for PostureApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme; // Use Theme directly
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
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
        (
            PostureApp {
                posture: "Connecting...".into(), // Initial state message
                raw_metrics: None,
                notification: Some(Notification::new().show().unwrap()),
                db_manager,
                last_logs: None,
            },
            Command::none(), // Subscription will initiate connection attempt
        )
    }

    fn title(&self) -> String {
        "Posture Monitor".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::PostureUpdate(metrics_str) => {
                // Parse the raw metrics
                if let Some(manager) = self.db_manager.take() {
                    self.last_logs = Some(manager.get_last_logs(6).unwrap());
                    self.db_manager = Some(manager);
                }
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

                    let previous_posture = self.posture.clone();

                    self.raw_metrics = Some(metrics);
                    self.posture = self.determine_posture();

                    if self.posture != previous_posture {
                        if let Some(manager) = self.db_manager.take() {
                            let _ = manager.log_posture_change(&self.posture, &previous_posture);
                            self.db_manager = Some(manager);
                        }

                        if self.posture == "STRAIGHT" {
                            if let Some(mut handle) = self.notification.take() {
                                handle
                                    .summary("Well done!")
                                    .body("Back to sitting straight, good job!")
                                    .timeout(Duration::from_secs(1));
                                handle.update();
                                self.notification = Some(handle);
                            }
                        } else {
                            if let Some(mut handle) = self.notification.take() {
                                handle
                                    .summary("Bad posture!")
                                    .body(&format!(
                                    "You should correct your posture. Current posture detected: {}",
                                    self.posture
                                ))
                                    .timeout(0);

                                handle.update();
                                self.notification = Some(handle);
                            }
                        }
                    }
                }
            }

            Message::Connected(Ok(())) => {
                self.posture = "Connected. Waiting for data...".into(); // Update UI on successful connection
            }

            Message::Connected(Err(e)) => {
                // Update UI on connection failure, include error message
                self.posture = format!("Connection failed: {}. Retrying...", e);
                // The subscription logic itself handles the retry by staying in Disconnected state
            }

            Message::Disconnected => {
                // Update UI when disconnected (e.g., server closed connection, read error)
                self.posture = "Disconnected. Retrying...".into();
                // The subscription logic handles the retry
            }
        }

        Command::none() // No further commands needed from update logic
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let svg_path: &str = match self.posture.as_str() {
            "STRAIGHT" => "./src/assets/good_posture.svg",
            _ => "./src/assets/bad_posture.svg",
        };

        let svg_widget = svg(svg::Handle::from_path(svg_path)).height(100).width(100);

        let mut content = column![svg_widget, Text::new(&self.posture).size(40)]
            .align_items(Center)
            .spacing(20);

        if let Some(events) = self.last_logs.clone() {
            let event_iter = events.clone().into_iter();
            let logs = event_iter
                .map(|event| {
                    Text::new(event.previous_posture.unwrap_or(String::from("UNKNOWN"))).size(10)
                })
                .collect::<Vec<Text>>();

            for log in logs {
                content = content.push(log.clone());
            }
        }

        return Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Initiate and manage the TCP connection subscription
        subscription::connect("tcp-reader").map(|result| result.unwrap())
    }
}

// Separate module encapsulating the subscription logic
mod subscription {
    use super::{BufReader, Message, State, TcpStream};
    use iced::subscription::{self, Subscription};
    use tokio::io::AsyncBufReadExt;

    // Function to create the subscription
    pub fn connect(id: &'static str) -> Subscription<Option<Message>> {
        // Use unfold to manage state across connection attempts and reads
        subscription::unfold(id, State::Disconnected, move |state| async move {
            match state {
                // If currently disconnected, attempt to connect
                State::Disconnected => {
                    // Optional: Introduce a delay before retrying connection
                    // sleep(Duration::from_secs(5)).await;
                    match TcpStream::connect("127.0.0.1:9876").await {
                        Ok(stream) => {
                            let reader = BufReader::new(stream);
                            // On success, send Connected message and change state
                            (Some(Message::Connected(Ok(()))), State::Connected(reader))
                        }
                        Err(e) => {
                            // On failure, send Connected message with error and remain Disconnected
                            (
                                Some(Message::Connected(Err(e.to_string()))),
                                State::Disconnected,
                            )
                        }
                    }
                }
                // If currently connected, attempt to read a line
                State::Connected(mut reader) => {
                    let mut line = String::new();
                    match reader.read_line(&mut line).await {
                        Ok(0) => {
                            // EOF reached (server closed connection)
                            // Send Disconnected message and change state
                            (Some(Message::Disconnected), State::Disconnected)
                        }
                        Ok(_) => {
                            // Successfully read a line
                            // Remove potential trailing newline characters (\n or \r\n)
                            if line.ends_with('\n') {
                                line.pop();
                                if line.ends_with('\r') {
                                    line.pop();
                                }
                            }
                            // Send PostureUpdate message with the line and keep Connected state
                            (Some(Message::PostureUpdate(line)), State::Connected(reader))
                        }
                        Err(e) => {
                            // Error during read (connection likely lost)
                            eprintln!("Error reading from TCP stream: {}", e);
                            // Send Disconnected message and change state
                            (Some(Message::Disconnected), State::Disconnected)
                        }
                    }
                }
            }
        })
    }
}

impl Drop for PostureApp {
    fn drop(&mut self) {
        if let Some(handle) = self.notification.take() {
            handle.close();
        }
        if let Some(manager) = self.db_manager.take() {
            manager.log_session_end(&self.posture).unwrap();
        }
    }
}
