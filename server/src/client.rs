use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

pub struct Client {
    write_stream: mpsc::Sender<Message>,
}

impl Client {
    pub async fn new(ws_stream: WebSocketStream<TcpStream>) -> Result<Self, String> {

        Err("".to_string())
    }
}