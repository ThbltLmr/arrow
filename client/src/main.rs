use futures::stream::StreamExt;
use iced::{executor, Application, Command, Element, Settings, Subscription, Text};
use tokio::io::{AsyncBufReadExt, BufReader};
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
}

impl Application for PostureApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            PostureApp {
                posture: "Waiting for data...".into(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Posture Monitor".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::PostureUpdate(p) => {
                self.posture = p;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        Text::new(&self.posture).size(40).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced::subscription::unfold("tcp-reader", (), |_| async {
            let stream = TcpStream::connect("127.0.0.1:65432").await.unwrap();
            let reader = BufReader::new(stream);
            let mut lines = reader.lines();

            loop {
                if let Some(Ok(line)) = lines.next_line().await {
                    return (Some(Message::PostureUpdate(line)), ());
                }
            }
        })
    }
}
