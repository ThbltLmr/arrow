use iced::{executor, widget::Text, Application, Command, Element, Settings, Subscription, Theme};
use tokio::io::BufReader;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> iced::Result {
    PostureApp::run(Settings::default())
}

struct PostureApp {
    posture: String,
}

#[derive(Debug, Clone)]
enum Message {
    PostureUpdate(String),
    Connected(Result<(), String>), // Use String to pass error message if any
    Disconnected,
}

// State for the subscription lifecycle
enum State {
    Disconnected,
    Connected(BufReader<TcpStream>),
}

impl Application for PostureApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme; // Use Theme directly
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            PostureApp {
                posture: "Connecting...".into(), // Initial state message
            },
            Command::none(), // Subscription will initiate connection attempt
        )
    }

    fn title(&self) -> String {
        "Posture Monitor".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::PostureUpdate(p) => {
                self.posture = p; // Update posture text from received data
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

    fn view(&self) -> Element<Self::Message> {
        // Display the current posture string
        Text::new(&self.posture).size(40).into()
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
