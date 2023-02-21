use futures::future;
use futures::stream::{self, Stream, StreamExt};
use semver::Version;
use serde::{Deserialize, Serialize};
use tokio::io::{self, AsyncBufReadExt, BufStream};
use tokio::net;

pub const SOCKET_ADDRESS: &'static str = "127.0.0.1:9167";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    Connected { title: String, version: Version },
}

#[derive(Debug, Clone)]
pub enum Message {
    Connected { title: String },
    Disconnected,
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
    loop {
        let mut input = String::new();

        loop {
            match stream.read_line(&mut input).await? {
                0 => return Ok((stream, Message::Disconnected)),
                _ => {
                    match serde_json::from_str(&input) {
                        Ok(input) => match input {
                            Input::Connected { title, version } => {
                                dbg!(&title, version);

                                return Ok((stream, Message::Connected { title }));
                            }
                        },
                        Err(_) => {
                            // TODO: Log decoding error
                        }
                    }
                }
            }
        }
    }
}
