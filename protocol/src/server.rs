use futures::future;
use futures::stream::{self, Stream, StreamExt};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt, BufStream};
use tokio::net;

pub const SOCKET_ADDRESS: &str = "127.0.0.1:9167";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    Connected { version: Version },
    PerformanceReported(Performance),
}

#[derive(Debug, Clone)]
pub enum Message {
    Connected,
    Disconnected,
    PerformanceReported(Performance),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Performance {
    Startup(Duration),
    Update(Duration),
    View(Duration),
    Layout(Duration),
    OnEvent(Duration),
    Draw(Duration),
    Render(Duration),
}

pub fn run() -> impl Stream<Item = Message> {
    enum State {
        Disconnected,
        Connected(BufStream<net::TcpStream>),
    }

    stream::unfold(State::Disconnected, |state| async {
        match state {
            State::Disconnected => match connect().await {
                Ok(stream) => {
                    let stream = BufStream::new(stream);

                    Some((None, State::Connected(stream)))
                }
                Err(_error) => Some((None, State::Disconnected)),
            },
            State::Connected(stream) => match receive(stream).await {
                Ok((_, Message::Disconnected)) | Err(_) => {
                    Some((Some(Message::Disconnected), State::Disconnected))
                }
                Ok((stream, message)) => Some((Some(message), State::Connected(stream))),
            },
        }
    })
    .filter_map(future::ready)
}

async fn connect() -> Result<net::TcpStream, io::Error> {
    let listener = net::TcpListener::bind(SOCKET_ADDRESS).await?;

    let (stream, _) = listener.accept().await?;

    stream.set_nodelay(true)?;
    stream.readable().await?;

    Ok(stream)
}

async fn receive(
    mut stream: BufStream<net::TcpStream>,
) -> Result<(BufStream<net::TcpStream>, Message), io::Error> {
    let mut input = String::new();

    loop {
        match stream.read_line(&mut input).await? {
            0 => return Ok((stream, Message::Disconnected)),
            n => {
                match serde_json::from_str(&input[..n]) {
                    Ok(input) => {
                        return Ok((
                            stream,
                            match input {
                                Input::Connected { version } => {
                                    dbg!(version);

                                    Message::Connected
                                }
                                Input::PerformanceReported(performance) => {
                                    dbg!(performance);

                                    Message::PerformanceReported(performance)
                                }
                            },
                        ))
                    }
                    Err(_) => {
                        // TODO: Log decoding error
                    }
                }
            }
        }
    }
}
